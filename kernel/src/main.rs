#![no_main]
#![no_std]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod graphics;

use common::FrameBufferConfig;
use core::arch::asm;
use graphics::{Color, Screen};

#[no_mangle]
extern "C" fn kernel_main(config: FrameBufferConfig) -> ! {
    let screen = Screen::from(config);
    screen.draw_all(Color::WHITE);

    for x in 0..200 {
        for y in 0..200 {
            screen.draw_pixel(x, y, Color::RED);
        }
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
