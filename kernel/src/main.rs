//! The AlohaOS kernel.
//!
//! For now it does exactly one thing: paint the "AlohaOS" splash onto the
//! framebuffer handed over by AlohaBoot, then halt forever.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use common::{BootInfo, PixelFormat};

mod font;

/// Kernel entry point. AlohaBoot jumps here with a pointer to `BootInfo`
/// in `rdi` (System V AMD64 calling convention).
#[no_mangle]
pub extern "sysv64" fn _start(boot_info: *const BootInfo) -> ! {
    let info: &BootInfo = unsafe { &*boot_info };
    let fb = &info.framebuffer;

    let buffer = fb.addr as *mut u32;
    let stride = fb.stride;
    let width = fb.width;
    let height = fb.height;
    let format = fb.pixel_format;

    // Deep ocean-blue background.
    let bg = compose(format, 0x0f, 0x17, 0x2a);
    for y in 0..height {
        for x in 0..width {
            unsafe { put_pixel(buffer, stride, x, y, bg) };
        }
    }

    // "AlohaOS", centered and scaled to the screen.
    let text = b"AlohaOS";
    let scale = pick_scale(width);
    let glyph_w = 8 * scale;
    let text_w = glyph_w * text.len();
    let text_h = 8 * scale;

    let start_x = center(width, text_w);
    let start_y = center(height, text_h);

    let fg = compose(format, 0xf5, 0xa6, 0x23); // warm aloha orange

    for (i, &ch) in text.iter().enumerate() {
        draw_glyph(buffer, stride, start_x + i * glyph_w, start_y, ch, scale, fg);
    }

    halt();
}

/// Center `used` within `total`, clamped to 0.
fn center(total: usize, used: usize) -> usize {
    if total > used {
        (total - used) / 2
    } else {
        0
    }
}

/// Pick an integer scale so the title spans roughly a third of the screen.
fn pick_scale(width: usize) -> usize {
    let target = width / 3;
    let natural = 8 * 7; // 7 glyphs * 8px
    (target / natural).max(1)
}

/// Compose a 32-bit pixel for the given framebuffer byte order.
fn compose(format: PixelFormat, r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    match format {
        PixelFormat::Rgb => (b << 16) | (g << 8) | r,
        _ => (r << 16) | (g << 8) | b, // Bgr and Unknown
    }
}

#[inline]
unsafe fn put_pixel(buffer: *mut u32, stride: usize, x: usize, y: usize, color: u32) {
    core::ptr::write_volatile(buffer.add(y * stride + x), color);
}

/// Blit a single 8x8 glyph, magnified by `scale`.
fn draw_glyph(
    buffer: *mut u32,
    stride: usize,
    x0: usize,
    y0: usize,
    ch: u8,
    scale: usize,
    color: u32,
) {
    let glyph = font::glyph(ch);
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..8 {
            if bits & (0x80 >> col) != 0 {
                for sy in 0..scale {
                    for sx in 0..scale {
                        unsafe {
                            put_pixel(
                                buffer,
                                stride,
                                x0 + col * scale + sx,
                                y0 + row * scale + sy,
                                color,
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Park the CPU. `hlt` in a loop keeps it cool while doing nothing.
fn halt() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt()
}
