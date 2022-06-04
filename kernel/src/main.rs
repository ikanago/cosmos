#![no_main]
#![no_std]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod console;
mod graphics;

use common::FrameBufferConfig;
use console::Console;
use core::arch::asm;
use graphics::{Color, Font, Screen};

#[no_mangle]
extern "C" fn kernel_main(config: FrameBufferConfig) -> ! {
    let screen = Screen::from(config);
    screen.draw_all(Color::BLACK);

    let font = Font;
    let mut console = Console::new(&screen, 25, 80);
    for ch in "Hello, kernel!".chars() {
        console.insert_char(ch);
    }
    console.render(&font);

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
