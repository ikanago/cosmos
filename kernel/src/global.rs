use core::fmt::Write;

use crate::graphics::{
    console::Console,
    screen::Screen,
    Color, Font, Render,
};
use common::FrameBufferConfig;
use spin::{Mutex, MutexGuard, Once};

static CONSOLE: Once<Mutex<Console>> = Once::new();

pub fn initialize_console(num_lines: usize, num_columns: usize, font: Font) {
    CONSOLE.call_once(|| Mutex::new(Console::new(num_lines, num_columns, font)));
}

pub fn write_console(args: core::fmt::Arguments) {
    let mut console = CONSOLE.get().unwrap().lock();
    console.write_fmt(args).unwrap();
}

pub fn flush_console() {
    let mut screen = ScreenLock::lock();
    let console = CONSOLE.get().unwrap().lock();
    console.render(&mut screen);
}

static SCREEN: Once<Mutex<Screen>> = Once::new();

pub fn initialize_screen(config: FrameBufferConfig) {
    SCREEN.call_once(|| Mutex::new(Screen::from(config)));
}

pub struct ScreenLock<'l> {
    pub lock: MutexGuard<'l, Screen>,
}

impl<'l> ScreenLock<'l> {
    pub fn lock() -> Self {
        let lock = SCREEN.get().unwrap().lock();
        Self { lock }
    }

    // pub fn draw<R: Render>(&mut self, drawing: &R) {
    //     self.lock.draw(drawing);
    // }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.lock.draw_pixel(x, y, color);
    }

    pub fn draw_all(&mut self, color: Color) {
        self.lock.draw_all(color);
    }
}
