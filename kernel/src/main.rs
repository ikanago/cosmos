#![no_main]
#![no_std]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod console;
mod graphics;

use common::FrameBufferConfig;
use console::Console;
use core::arch::asm;
use graphics::{Color, Font, Screen, Vector2D};

#[no_mangle]
extern "C" fn kernel_main(config: FrameBufferConfig) -> ! {
    let frame_width = config.horizontal_resolution;
    let frame_height = config.vertical_resolution;
    let screen = Screen::from(config);
    screen.draw_all(Color::BLACK);
    screen.draw_filled_rectangle(
        Vector2D::new(0, frame_height - 50),
        Vector2D::new(frame_width, 50),
        Color {
            r: 0x32,
            g: 0x35,
            b: 0xeb,
        },
    );

    let font = Font;
    let mut console = Console::new(&screen, 5, 20);
    console.put_string("Hello, kernel!\n");
    console.put_string("line2\n");
    console.put_string("line3\n");
    console.put_string("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n");
    console.put_string("line5");
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
