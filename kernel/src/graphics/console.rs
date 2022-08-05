use core::fmt::Write;

use crate::{
    global::ScreenLock,
    graphics::{Attribute, Font, Point, Render},
};

const CONSOLE_BUFFER_SIZE: usize = 10_000;

#[derive(Default)]
struct Cursor {
    row: usize,
    column: usize,
}

pub struct Console {
    num_rows: usize,
    num_columns: usize,
    buffer: [Option<char>; CONSOLE_BUFFER_SIZE],
    cursor: Cursor,
    attribute: Attribute,
    font: Font,
}

impl Console {
    pub fn new(num_lines: usize, num_columns: usize, font: Font) -> Self {
        Self {
            num_rows: num_lines,
            num_columns,
            buffer: [None; CONSOLE_BUFFER_SIZE],
            cursor: Cursor::default(),
            attribute: Attribute::default(),
            font,
        }
    }

    /// Feed a character `ch` to the console.
    /// This method also handles cursor movement.
    pub fn put_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.move_cursor_down();
            }
            ch => {
                let index = self.cursor.row * self.num_columns + self.cursor.column;
                self.buffer[index] = Some(ch);
                self.move_cursor_forward();
            }
        }
    }

    /// Feed a string `s` to the console.
    /// Equivalent to calling `Console::put_char()` for each character in the string.
    pub fn put_string(&mut self, s: &str) {
        for ch in s.chars() {
            self.put_char(ch);
        }
    }

    /// Move cursor one character forward.
    /// Intended to be called in `Console::put_char()`
    fn move_cursor_forward(&mut self) {
        if self.cursor.column < self.num_columns - 1 {
            self.cursor.column += 1;
        } else {
            self.move_cursor_down();
        }
    }

    /// Move cursor one row down.
    /// Also scrolls console contents if needed.
    fn move_cursor_down(&mut self) {
        self.cursor.column = 0;
        if self.cursor.row < self.num_rows - 1 {
            self.cursor.row += 1;
        } else {
            self.scroll_up();
        }
    }

    fn scroll_up(&mut self) {
        let end = self.num_rows * self.num_columns;
        let copy_range = self.num_columns..end;
        self.buffer.copy_within(copy_range, 0);

        let fill_range = (end - self.num_columns)..end;
        self.buffer[fill_range].fill(None);
    }
}

impl Render for Console {
    fn render(&self, screen: &mut ScreenLock) {
        for row in 0..self.num_rows {
            for column in 0..self.num_columns {
                let index = row * self.num_columns + column;
                if let Some(ch) = self.buffer[index] {
                    let x = column * Font::CHAR_WIDTH;
                    let y = row * Font::CHAR_HEIGHT;
                    self.font
                        .draw_char(screen, Point::new(x, y), ch, self.attribute);
                }
            }
        }
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.put_char(c);
        }
        Ok(())
    }
}
