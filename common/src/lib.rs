#![no_std]

#[repr(C)]
pub struct FrameBufferConfig {
    pub base: *mut u8,
    pub size: usize,
}
