//! Tiny allocation-free framebuffer console used by boot and panic paths.

use common::{FrameBufferInfo, PixelFormat};
use crate::font;

const SCALE: usize = 2;
const GLYPH_WIDTH: usize = 8 * SCALE;
const GLYPH_HEIGHT: usize = 8 * SCALE;
const LINE_HEIGHT: usize = GLYPH_HEIGHT + 6;
const LEFT: usize = 32;
const TOP: usize = 32;

static mut FRAMEBUFFER: FrameBufferInfo = FrameBufferInfo {
    addr: 0,
    size: 0,
    width: 0,
    height: 0,
    stride: 0,
    pixel_format: PixelFormat::Unknown,
};
static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = TOP;
static mut COLOR: u32 = 0x00ff_ffff;

pub fn init(info: FrameBufferInfo) {
    unsafe {
        FRAMEBUFFER = info;
        CURSOR_X = 0;
        CURSOR_Y = TOP;
        COLOR = compose(info.pixel_format, 0xff, 0xff, 0xff);
    }
}

pub fn clear(r: u8, g: u8, b: u8) {
    unsafe {
        let fb = FRAMEBUFFER;
        let color = compose(fb.pixel_format, r, g, b);
        let buffer = fb.addr as *mut u32;
        for y in 0..fb.height {
            for x in 0..fb.width {
                core::ptr::write_volatile(buffer.add(y * fb.stride + x), color);
            }
        }
        CURSOR_X = 0;
        CURSOR_Y = TOP;
    }
}

pub fn set_color(r: u8, g: u8, b: u8) {
    unsafe { COLOR = compose(FRAMEBUFFER.pixel_format, r, g, b) }
}

pub fn panic_header(reason: &str) {
    clear(0x38, 0x08, 0x12);
    set_color(0xff, 0xd1, 0xd9);
    write_line("ALOHAOS KERNEL PANIC");
    write_line("");
    set_color(0xff, 0xff, 0xff);
    write_line(reason);
    write_line("");
}

pub fn write_label_hex(label: &str, value: u64) {
    write_text(label);
    write_text("0X");
    for shift in (0..16).rev() {
        let nibble = ((value >> (shift * 4)) & 0xf) as u8;
        draw_char(if nibble < 10 { b'0' + nibble } else { b'A' + nibble - 10 });
    }
    newline();
}

pub fn write_line(text: &str) {
    write_text(text);
    newline();
}

fn write_text(text: &str) {
    for byte in text.bytes() {
        draw_char(byte.to_ascii_uppercase());
    }
}

fn newline() {
    unsafe {
        CURSOR_X = 0;
        CURSOR_Y += LINE_HEIGHT;
    }
}

fn draw_char(ch: u8) {
    unsafe {
        let fb = FRAMEBUFFER;
        let x0 = LEFT + CURSOR_X * GLYPH_WIDTH;
        let y0 = CURSOR_Y;
        if x0 + GLYPH_WIDTH >= fb.width || y0 + GLYPH_HEIGHT >= fb.height {
            return;
        }

        let buffer = fb.addr as *mut u32;
        for (row, bits) in font::glyph(ch).iter().enumerate() {
            for col in 0..8 {
                if bits & (0x80 >> col) != 0 {
                    for sy in 0..SCALE {
                        for sx in 0..SCALE {
                            core::ptr::write_volatile(
                                buffer.add(
                                    (y0 + row * SCALE + sy) * fb.stride
                                        + x0
                                        + col * SCALE
                                        + sx,
                                ),
                                COLOR,
                            );
                        }
                    }
                }
            }
        }
        CURSOR_X += 1;
    }
}

fn compose(format: PixelFormat, r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    match format {
        PixelFormat::Rgb => (b << 16) | (g << 8) | r,
        _ => (r << 16) | (g << 8) | b,
    }
}
