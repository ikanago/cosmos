use common::{FrameBufferConfig, PixelFormat};

pub const BYTES_PER_PIXEL: usize = 4;

#[derive(Clone, Copy, Debug)]
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
}

/// This struct is responsible for drawing pixels via frame buffer.
pub struct Screen {
    frame_buffer: *mut u8,
    stride: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
    r_offset: usize,
    g_offset: usize,
    b_offset: usize,
}

impl Screen {}

impl From<FrameBufferConfig> for Screen {
    fn from(config: FrameBufferConfig) -> Self {
        let (r_offset, g_offset, b_offset) = match config.format {
            PixelFormat::Rgb => (0, 1, 2),
            PixelFormat::Bgr => (2, 1, 0),
        };
        Self {
            frame_buffer: config.base,
            stride: config.stride,
            horizontal_resolution: config.horizontal_resolution,
            vertical_resolution: config.vertical_resolution,
            r_offset,
            g_offset,
            b_offset,
        }
    }
}

impl Screen {
    fn frame_buffer_size(&self) -> usize {
        BYTES_PER_PIXEL * self.stride * self.vertical_resolution
    }

    /// Draw `color` at specified position (x, y).
    /// (x, y) is a coordinate in the form (horizontal, vertical).
    pub fn draw_pixel(&self, x: usize, y: usize, color: Color) {
        let frame_buffer_slice =
            unsafe { core::slice::from_raw_parts_mut(self.frame_buffer, self.frame_buffer_size()) };
        let position = self.stride * y + x;
        let base = BYTES_PER_PIXEL * position;
        let Color { r, g, b } = color;
        frame_buffer_slice[base + self.r_offset] = r;
        frame_buffer_slice[base + self.g_offset] = g;
        frame_buffer_slice[base + self.b_offset] = b;
    }

    /// Draw all the screen with `color`.
    pub fn draw_all(&self, color: Color) {
        for x in 0..self.horizontal_resolution {
            for y in 0..self.vertical_resolution {
                self.draw_pixel(x, y, color);
            }
        }
    }
}

pub struct Font;

impl Font {
    const FONT_DATA: &'static [u8] = include_bytes!("../assets/hankaku.bin");
    const BYTES_PER_CHAR: usize = 16;
    const CHAR_WIDTH: usize = 8;
    const CHAR_MARGIN: usize = 1;

    pub fn draw_char(&self, screen: &Screen, x: usize, y: usize, ch: char, fg_color: Color) {
        let ch = if ch as usize >= Self::FONT_DATA.len() {
            b'?' as usize
        } else {
            ch as usize
        };

        let ch_pos = Self::BYTES_PER_CHAR * ch;
        for dy in 0..Self::BYTES_PER_CHAR {
            let row_in_bitmap = Self::FONT_DATA[ch_pos + dy];
            for dx in 0..Self::CHAR_WIDTH {
                if row_in_bitmap & (0x80 >> dx) != 0 {
                    screen.draw_pixel(x + dx, y + dy, fg_color);
                }
            }
        }
    }

    pub fn draw_string(&self, screen: &Screen, x: usize, y: usize, s: &str, fg_color: Color) {
        for (i, ch) in s.chars().enumerate() {
            let x = x + i * (Self::CHAR_WIDTH + Self::CHAR_MARGIN * 2);
            self.draw_char(screen, x, y, ch, fg_color);
        }
    }
}