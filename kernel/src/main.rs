#![no_main]
#![no_std]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
use core::arch::asm;

#[no_mangle]
pub extern "C" fn kernel_main(frame_buffer_base: *mut u8, frame_buffer_size: usize) -> ! {
    let frame_buffer =
        unsafe { core::slice::from_raw_parts_mut(frame_buffer_base, frame_buffer_size) };
    for (i, pixel) in frame_buffer.iter_mut().enumerate() {
        *pixel = (i % 0xff) as u8;
    }

    #[allow(clippy::empty_loop)]
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    #[allow(clippy::empty_loop)]
    loop {
        unsafe { asm!("hlt") }
    }
}
