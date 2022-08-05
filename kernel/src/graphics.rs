pub mod console;
pub mod mouse;
pub mod screen;

use core::ops::AddAssign;
use screen::Screen;

use crate::global::ScreenLock;

/// Trait to abstruct objects that are rendered on `Screen`.
pub trait Render {
    fn render(&self, screen: &mut ScreenLock);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[allow(unused)]
impl Color {
    pub const WHITE: Self = Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    };
    pub const BLACK: Self = Color {
        r: 0x0,
        g: 0x0,
        b: 0x0,
    };
    pub const RED: Self = Color {
        r: 0xff,
        g: 0x0,
        b: 0x0,
    };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point<T>
where
    T: AddAssign,
{
    x: T,
    y: T,
}

impl<T> Point<T>
where
    T: AddAssign,
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> AddAssign for Point<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x = rhs.x;
        self.y = rhs.y;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Attribute {
    fg_color: Color,
    bg_color: Color,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            fg_color: Color::WHITE,
            bg_color: Color::BLACK,
        }
    }
}

pub struct Font;

impl Font {
    const FONT_DATA: &'static [u8] = include_bytes!("../assets/hankaku.bin");
    pub const CHAR_WIDTH: usize = 8;
    pub const CHAR_HEIGHT: usize = 16;

    /// Draw character `ch` at the specific position.
    /// (x, y) is the coordinate of top left pixel of the bounding rectangle.
    pub fn draw_char(
        &self,
        screen: &mut ScreenLock,
        pos: Point<usize>,
        ch: char,
        attribute: Attribute,
    ) {
        let ch = if ch as usize >= Self::FONT_DATA.len() {
            b'?' as usize
        } else {
            ch as usize
        };

        let ch_pos = Self::CHAR_HEIGHT * ch;
        for dy in 0..Self::CHAR_HEIGHT {
            let row_in_bitmap = Self::FONT_DATA[ch_pos + dy];
            for dx in 0..Self::CHAR_WIDTH {
                if row_in_bitmap & (0x80 >> dx) != 0 {
                    screen.draw_pixel(pos.x + dx, pos.y + dy, attribute.fg_color);
                } else {
                    screen.draw_pixel(pos.x + dx, pos.y + dy, attribute.bg_color);
                }
            }
        }
    }

    pub fn draw_string(
        &self,
        screen: &mut ScreenLock,
        pos: Point<usize>,
        s: &str,
        attribute: Attribute,
    ) {
        for (i, ch) in s.chars().enumerate() {
            let pos = Point::new(pos.x + i * Self::CHAR_WIDTH, pos.y);
            self.draw_char(screen, pos, ch, attribute);
        }
    }
}
