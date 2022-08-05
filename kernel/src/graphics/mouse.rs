use crate::global::ScreenLock;

use super::{screen::Screen, Color, Point, Render};

pub struct MouseCursor {
    pos: Point<usize>,
    edge_color: Color,
    fill_color: Color,
}

impl MouseCursor {
    const MOUSE_CURSOR_SHAPE: [[u8; 15]; 24] = [
        *b"@              ",
        *b"@@             ",
        *b"@.@            ",
        *b"@..@           ",
        *b"@...@          ",
        *b"@....@         ",
        *b"@.....@        ",
        *b"@......@       ",
        *b"@.......@      ",
        *b"@........@     ",
        *b"@.........@    ",
        *b"@..........@   ",
        *b"@...........@  ",
        *b"@............@ ",
        *b"@......@@@@@@@@",
        *b"@......@       ",
        *b"@....@@.@      ",
        *b"@...@ @.@      ",
        *b"@..@   @.@     ",
        *b"@.@    @.@     ",
        *b"@@      @.@    ",
        *b"@       @.@    ",
        *b"         @.@   ",
        *b"         @@@   ",
    ];

    pub fn new(pos: Point<usize>, edge_color: Color, fill_color: Color) -> Self {
        Self {
            pos,
            edge_color,
            fill_color,
        }
    }
}

impl Render for MouseCursor {
    fn render(&self, screen: &mut ScreenLock) {
        for (y, row) in MouseCursor::MOUSE_CURSOR_SHAPE.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                let color = match pixel {
                    b'@' => self.edge_color,
                    b'.' => self.fill_color,
                    _ => continue,
                };
                screen.draw_pixel(self.pos.x + x, self.pos.y + y, color);
            }
        }
    }
}
