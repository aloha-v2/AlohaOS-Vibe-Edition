//! Shared contract between AlohaBoot and the kernel.
//!
//! Kept intentionally tiny and `#[repr(C)]` so both sides agree on the exact
//! memory layout across the boot handoff.
#![no_std]

/// Byte order of each 32-bit pixel in the framebuffer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PixelFormat {
    /// Red is the least-significant byte.
    Rgb = 0,
    /// Blue is the least-significant byte (the common UEFI case).
    Bgr = 1,
    /// Anything we do not explicitly handle.
    Unknown = 2,
}

/// Everything the kernel needs to draw to the screen.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct FrameBufferInfo {
    /// Physical base address of the linear framebuffer.
    pub addr: u64,
    /// Size of the framebuffer in bytes.
    pub size: usize,
    /// Visible width in pixels.
    pub width: usize,
    /// Visible height in pixels.
    pub height: usize,
    /// Pixels per scanline (may exceed `width` due to padding).
    pub stride: usize,
    /// Pixel byte order.
    pub pixel_format: PixelFormat,
}

/// Handed from the bootloader to the kernel entry point.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct BootInfo {
    pub framebuffer: FrameBufferInfo,
}

impl BootInfo {
    /// A zeroed placeholder used before the real values are filled in.
    pub const EMPTY: BootInfo = BootInfo {
        framebuffer: FrameBufferInfo {
            addr: 0,
            size: 0,
            width: 0,
            height: 0,
            stride: 0,
            pixel_format: PixelFormat::Unknown,
        },
    };
}
