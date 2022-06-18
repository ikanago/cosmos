#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![feature(int_roundings)]

use anyhow::{anyhow, Result};
use common::{FrameBufferConfig, PixelFormat};
use core::fmt::Write;
use object::{Object, ObjectSegment};
use uefi::{
    prelude::entry,
    proto::{
        console::gop::{GraphicsOutput, PixelFormat as UefiPixelFormat},
        media::file::{Directory, File, FileAttribute, FileInfo, FileMode, RegularFile},
    },
    table::{
        boot::{AllocateType, BootServices, MemoryType},
        Boot, SystemTable,
    },
    CStr16, {Handle, Status},
};

type KernelEntryPoint = extern "sysv64" fn(FrameBufferConfig);

const KERNEL_FILE_NAME_MAX_LEN: usize = 32;
const KERNEL_FILE_INFO_BUF_SIZE: usize = 8192;
const PAGE_SIZE: usize = 0x1000;

#[entry]
fn efi_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    system_table.stdout().reset(false).unwrap();
    writeln!(system_table.stdout(), "Hello, UEFI!").unwrap();

    boot(handle, system_table);

    #[allow(unreachable_code)]
    Status::SUCCESS
}

fn boot(handle: Handle, mut system_table: SystemTable<Boot>) -> ! {
    let boot_services = system_table.boot_services();
    let mut memmap_buf = [0; 4 * PAGE_SIZE];
    assert!(boot_services.memory_map_size().map_size < memmap_buf.len());
    if let Err(err) = boot_services.memory_map(&mut memmap_buf) {
        // Error must be printed in this function because `system_table` is taken by `exit_boot_service`.
        writeln!(
            system_table.stdout(),
            "Failed to get memory map; status: {:?}, data: {:?}",
            err.status(),
            err.data()
        )
        .unwrap();
        panic!();
    }

    let kernel_main = match load_kernel(handle, boot_services, "\\kernel.elf") {
        Ok(kernel_main) => kernel_main,
        Err(err) => {
            writeln!(system_table.stdout(), "Failed to load kernel: {:?}", err).unwrap();
            panic!();
        }
    };

    let frame_buffer_config = get_frame_buffer_config(boot_services);

    writeln!(system_table.stdout(), "Kernel loaded").unwrap();

    system_table
        .exit_boot_services(handle, &mut memmap_buf)
        .unwrap();

    kernel_main(frame_buffer_config);

    #[allow(clippy::empty_loop)]
    loop {}
}

fn get_frame_buffer_config(boot_services: &BootServices) -> FrameBufferConfig {
    let gop = boot_services.locate_protocol::<GraphicsOutput>().unwrap();
    let gop = unsafe { &mut *gop.get() };
    let mut frame_buffer = gop.frame_buffer();
    let buffer_base = frame_buffer.as_mut_ptr();
    let buffer_size = frame_buffer.size();

    let mode_info = gop.current_mode_info();
    let stride = mode_info.stride();
    let (horizontal_resolution, vertical_resolution) = mode_info.resolution();

    FrameBufferConfig {
        buffer_base,
        buffer_size,
        stride,
        horizontal_resolution,
        vertical_resolution,
        format: match mode_info.pixel_format() {
            UefiPixelFormat::Rgb => PixelFormat::Rgb,
            UefiPixelFormat::Bgr => PixelFormat::Bgr,
            _ => unimplemented!(),
        },
    }
}

fn load_kernel(
    handle: Handle,
    boot_services: &BootServices,
    file_name: &str,
) -> Result<KernelEntryPoint> {
    let mut root_dir = open_root_dir(handle, boot_services)
        .map_err(|_| anyhow!("Failed to open root directory"))?;
    let kernel_file = open_kernel_file(&mut root_dir, file_name)?;
    let kernel_content = load_kernel_in_memory(boot_services, kernel_file)?;

    let binary = object::File::parse(kernel_content as &[u8])
        .map_err(|_| anyhow!("Failed to parse the kernel file"))?;
    let (kernel_base_address, kernel_end) = calculate_load_address_range(&binary);

    let num_pages = (kernel_end - kernel_base_address).div_ceil(PAGE_SIZE);
    // `kernel_segments_memory` points to memory region that segments of the kernel should be loaded.
    let kernel_segments_memory = {
        boot_services
            .allocate_pages(
                AllocateType::Address(kernel_base_address),
                MemoryType::LOADER_DATA,
                num_pages,
            )
            .map_err(|_| anyhow!("Failed to allocate pages for the kernel"))?;
        unsafe {
            core::slice::from_raw_parts_mut(kernel_base_address as *mut u8, num_pages * PAGE_SIZE)
        }
    };

    // Area that a segment is not loaded should be 0-cleared.
    unsafe {
        core::ptr::write_bytes(
            kernel_segments_memory.as_mut_ptr(),
            0,
            kernel_segments_memory.len(),
        )
    };

    for segment in binary.segments() {
        let data = segment
            .data()
            .map_err(|_| anyhow!("Failed to read kernel data at {:x}", segment.address()))?;
        let p_offset = segment.address() as usize - kernel_base_address;
        let end_address = p_offset + data.len();
        // The head address of `kernel_segments_memory` is the address of the first segment of the kernel.
        // So load the segment data at `p_offset`, which is equivalent to the one in the program header.
        kernel_segments_memory[p_offset..end_address].copy_from_slice(data);
    }

    let entry_point_address = binary.entry() as usize;
    let entry_point: KernelEntryPoint = unsafe { core::mem::transmute(entry_point_address) };

    boot_services
        .free_pool(kernel_content.as_mut_ptr())
        .map_err(|_| anyhow!("Failed to free allocated pool"))?;

    Ok(entry_point)
}

fn open_root_dir(handle: Handle, boot_services: &BootServices) -> uefi::Result<Directory> {
    let fs = boot_services.get_image_file_system(handle)?;
    let fs = unsafe { &mut *fs.interface.get() };
    fs.open_volume()
}

fn open_kernel_file(root_dir: &mut Directory, file_name: &str) -> Result<RegularFile> {
    let mut file_name_buf = [0; KERNEL_FILE_NAME_MAX_LEN + 1];
    let file_name = CStr16::from_str_with_buf(file_name, &mut file_name_buf)
        .map_err(|_| anyhow!("Failed to get CStr16 representation of the kernel file name"))?;
    let kernel_file = root_dir
        .open(file_name, FileMode::Read, FileAttribute::empty())
        .map_err(|_| anyhow!("Failed to open the kernel file"))?;
    let kernel_file = kernel_file
        .into_regular_file()
        .ok_or_else(|| anyhow!("The kernel file is not a regular one"))?;
    Ok(kernel_file)
}

/// Load the kernel to the memory from `kernel_file` for temporary use.
/// Returned array must be freed by `BootServeces::free_pool()`.
fn load_kernel_in_memory(
    boot_services: &BootServices,
    mut kernel_file: RegularFile,
) -> Result<&mut [u8]> {
    let kernel_file_size = {
        // TODO: research an exact size of this buffer.
        let mut kernel_file_info_buf = [0; KERNEL_FILE_INFO_BUF_SIZE];
        let kernel_file_info = kernel_file
            .get_info::<FileInfo>(&mut kernel_file_info_buf)
            .map_err(|_| anyhow!("Failed to get the kernel file information"))?;
        kernel_file_info.file_size() as usize
    };

    let kernel_content = {
        let tmp = boot_services
            .allocate_pool(MemoryType::LOADER_DATA, kernel_file_size)
            .map_err(|_| anyhow!("Failed to allocate memories for temporary kernel analysis"))?;

        unsafe { core::slice::from_raw_parts_mut(tmp, kernel_file_size) }
    };
    let num_read = kernel_file
        .read(kernel_content)
        .map_err(|_| anyhow!("Failed to load the kernel file to temporary allocated memories"))?;
    assert_eq!(num_read, kernel_file_size);

    Ok(kernel_content)
}

/// Calculate an address range of segments to be loaded.
fn calculate_load_address_range(binary: &object::read::File) -> (usize, usize) {
    let mut start = u64::MAX;
    let mut end = u64::MIN;
    for segment in binary.segments() {
        start = start.min(segment.address());
        end = end.max(segment.address() + segment.size());
    }
    (start as usize, end as usize)
}
