#![no_main]
#![no_std]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod graphics;

use common::FrameBufferConfig;
use core::arch::asm;
use graphics::{
    console::Console,
    mouse::MouseCursor,
    screen::{initialize_screen, FilledRectangle, ScreenLock},
    Color, Font, Point, Render,
};

#[no_mangle]
extern "C" fn kernel_main(config: FrameBufferConfig) -> ! {
    let frame_width = config.horizontal_resolution;
    let frame_height = config.vertical_resolution;
    initialize_screen(config);
    let mut screen = ScreenLock::lock();
    screen.draw_all(Color::BLACK);
    screen.draw(&FilledRectangle::new(
        Point::new(0, frame_height - 50),
        Point::new(frame_width, 50),
        Color {
            r: 0x32,
            g: 0x35,
            b: 0xeb,
        },
    ));

    let mut console = Console::new(5, 20, Font);
    console.put_string("Hello, kernel!\n");
    console.put_string("line2\n");
    console.put_string("line3\n");
    console.put_string("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n");
    console.put_string("line5");
    screen.draw(&console);

    let mouse_cursor = MouseCursor::new(Point::new(300, 300), Color::WHITE, Color::BLACK);
    screen.draw(&mouse_cursor);

    hlt_loop();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    hlt_loop()
}

fn hlt_loop() -> ! {
    #[allow(clippy::empty_loop)]
    loop {
        unsafe { asm!("hlt") }
    }
}
