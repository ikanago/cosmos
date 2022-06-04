#![no_std]

#[repr(u8)]
pub enum PixelFormat {
    Rgb,
    Bgr,
}

#[repr(C)]
pub struct FrameBufferConfig {
    pub base: *mut u8,
    // Horizontal length of frame buffer, which might be longer than `horizontal_resolution`.
    pub stride: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
    pub format: PixelFormat,
}
