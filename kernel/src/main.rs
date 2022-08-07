#![no_main]
#![no_std]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod global;
mod graphics;
mod pci;
mod x86;

use common::FrameBufferConfig;
use global::{initialize_console, initialize_screen, ScreenLock};
use graphics::{mouse::MouseCursor, screen::FilledRectangle, Color, Font, Point, Render};
use x86::hlt;

use crate::pci::scan_all_bus;

#[no_mangle]
extern "C" fn kernel_main(config: FrameBufferConfig) -> ! {
    let frame_width = config.horizontal_resolution;
    let frame_height = config.vertical_resolution;
    initialize_screen(config);
    {
        let mut screen = ScreenLock::lock();
        screen.draw_all(Color::BLACK);
        FilledRectangle::new(
            Point::new(0, frame_height - 50),
            Point::new(frame_width, 50),
            Color {
                r: 0x32,
                g: 0x35,
                b: 0xeb,
            },
        )
        .render(&mut screen);
        MouseCursor::new(Point::new(300, 300), Color::WHITE, Color::BLACK).render(&mut screen);
    }

    initialize_console(35, 90, Font);
    println!("Hello, kernel!");

    match scan_all_bus() {
        Err(err) => {
            println!("{}", err);
        }
        Ok(devices) => {
            println!("{} PCI devices found", devices.len());
            for device in devices {
                println!("{}", device);
            }
        }
    }

    hlt_loop();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop()
}

fn hlt_loop() -> ! {
    #[allow(clippy::empty_loop)]
    loop {
        hlt();
    }
}
