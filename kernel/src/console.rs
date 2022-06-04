use crate::graphics::{Attribute, Font, Screen};

const CONSOLE_BUFFER_SIZE: usize = 10_000;

struct Cursor {
    row: usize,
    column: usize,
}

impl Default for Cursor {
    fn default() -> Self {
        Self { row: 0, column: 0 }
    }
}

pub struct Console<'s> {
    screen: &'s Screen,
    num_rows: usize,
    num_columns: usize,
    buffer: [Option<char>; CONSOLE_BUFFER_SIZE],
    cursor: Cursor,
    attribute: Attribute,
}

impl<'s> Console<'s> {
    pub fn new(screen: &'s Screen, num_lines: usize, num_columns: usize) -> Self {
        Self {
            screen,
            num_rows: num_lines,
            num_columns,
            buffer: [None; CONSOLE_BUFFER_SIZE],
            cursor: Cursor::default(),
            attribute: Attribute::default(),
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let index = self.cursor.row * self.num_columns + self.cursor.column;
        self.buffer[index] = Some(ch);
        self.move_cursor_forward();
    }

    fn move_cursor_forward(&mut self) {
        if self.cursor.column == self.num_columns - 1 {
            self.cursor.row += 1;
            self.cursor.column = 0;
        } else {
            self.cursor.column += 1;
        }
    }

    pub fn render(&self, font: &Font) {
        for row in 0..self.num_rows {
            for column in 0..self.num_columns {
                let index = row * self.num_columns + column;
                if let Some(ch) = self.buffer[index] {
                    let x = column * Font::CHAR_WIDTH;
                    let y = row * Font::CHAR_HEIGHT;
                    font.draw_char(self.screen, x, y, ch, self.attribute);
                }
            }
        }
    }
}
