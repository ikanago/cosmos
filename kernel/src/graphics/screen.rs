use crate::{Color, Point, Render};
use common::{FrameBufferConfig, PixelFormat};

const BYTES_PER_PIXEL: usize = 4;

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

    pub fn draw<R: Render>(&self, drawing: &R) {
        drawing.render(self);
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
        for y in 0..self.vertical_resolution {
            for x in 0..self.horizontal_resolution {
                self.draw_pixel(x, y, color);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FilledRectangle {
    pos: Point<usize>,
    size: Point<usize>,
    color: Color,
}

impl FilledRectangle {
    pub fn new(pos: Point<usize>, size: Point<usize>, color: Color) -> Self {
        Self { pos, size, color }
    }
}

impl Render for FilledRectangle {
    fn render(&self, screen: &Screen) {
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                screen.draw_pixel(self.pos.x + x, self.pos.y + y, self.color);
            }
        }
    }
}
