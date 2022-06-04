#![no_main]
#![no_std]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use core::arch::asm;
use common::FrameBufferConfig;

#[no_mangle]
extern "C" fn kernel_main(config: FrameBufferConfig) -> ! {
    let frame_buffer = unsafe { core::slice::from_raw_parts_mut(config.base, config.size) };
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
