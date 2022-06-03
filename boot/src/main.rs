#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![feature(int_roundings)]

use anyhow::{anyhow, Result};
use core::fmt::Write;
use object::{Object, ObjectSegment};
use uefi::{
    prelude::entry,
    proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode, RegularFile},
    table::{
        boot::{AllocateType, BootServices, MemoryType},
        Boot, SystemTable,
    },
    CStr16, {Handle, Status},
};

const KERNEL_FILE_NAME_MAX_LEN: usize = 32;
const KERNEL_FILE_INFO_BUF_SIZE: usize = 8192;
const PAGE_SIZE: usize = 0x1000;

#[entry]
fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    boot(handle, system_table).unwrap();
    Status::SUCCESS
}

fn boot(handle: Handle, mut system_table: SystemTable<Boot>) -> Result<()> {
    uefi_services::init(&mut system_table).expect("Failed to initialize utilities");
    system_table.stdout().reset(false).unwrap();
    writeln!(system_table.stdout(), "Hello, UEFI!").unwrap();

    let boot_services = system_table.boot_services();
    let mut memmap_buf = [0; 4 * PAGE_SIZE];
    assert!(boot_services.memory_map_size().map_size < memmap_buf.len());
    boot_services
        .memory_map(&mut memmap_buf)
        .map_err(|_| anyhow!("Failed to get memory map"))?;

    let mut root_dir = open_root_dir(handle, boot_services)
        .map_err(|_| anyhow!("Failed to open root directory"))?;
    let kernel_main = load_kernel(boot_services, &mut root_dir, "\\kernel.elf")?;

    writeln!(system_table.stdout(), "Kernel loaded").unwrap();

    system_table
        .exit_boot_services(handle, &mut memmap_buf)
        .map_err(|_| anyhow!("Failed to exit boot services"))?;

    kernel_main();

    #[allow(clippy::empty_loop)]
    loop {}
}

fn open_root_dir(handle: Handle, boot_services: &BootServices) -> uefi::Result<Directory> {
    let fs = boot_services.get_image_file_system(handle)?;
    let fs = unsafe { &mut *fs.interface.get() };
    fs.open_volume()
}

fn load_kernel(
    boot_services: &BootServices,
    root_dir: &mut Directory,
    file_name: &str,
) -> Result<extern "sysv64" fn()> {
    let mut kernel_file = open_kernel_file(root_dir, file_name)?;
    let mut kernel_file_info_buf = [0; KERNEL_FILE_INFO_BUF_SIZE];
    let kernel_file_info = kernel_file
        .get_info::<FileInfo>(&mut kernel_file_info_buf)
        .map_err(|_| anyhow!("Failed to get the kernel file information"))?;
    let kernel_file_size = kernel_file_info.file_size() as usize;

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

    let binary = object::File::parse(kernel_content as &[u8])
        .map_err(|_| anyhow!("Failed to parse the kernel file"))?;
    let (kernel_base_address, kernel_end) = calculate_load_address_range(&binary);

    let num_pages = (kernel_end - kernel_base_address).div_ceil(PAGE_SIZE);
    let allocated_pages = {
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

    for segment in binary.segments() {
        let data = segment
            .data()
            .map_err(|_| anyhow!("Failed to read kernel data at {:x}", segment.address()))?;
        let p_offset = segment.address() as usize - kernel_base_address;
        let end_address = p_offset + data.len();
        allocated_pages[p_offset..end_address].copy_from_slice(data);
    }

    let entry_point_address = binary.entry() as usize;
    let entry_point: extern "sysv64" fn() = unsafe { core::mem::transmute(entry_point_address) };

    boot_services
        .free_pool(kernel_content.as_mut_ptr())
        .map_err(|_| anyhow!("Failed to free allocated pool"))?;

    Ok(entry_point)
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

fn calculate_load_address_range(binary: &object::read::File) -> (usize, usize) {
    let mut start = u64::MAX;
    let mut end = u64::MIN;
    for segment in binary.segments() {
        start = start.min(segment.address());
        end = end.max(segment.address() + segment.size());
    }
    (start as usize, end as usize)
}
