//! Tiny allocation-free framebuffer console used by boot, panic and shell paths.

use common::{FrameBufferInfo, PixelFormat};

use crate::{font, sync::IrqSpinLock};

const SCALE: usize = 2;
const GLYPH_WIDTH: usize = 16;
const GLYPH_HEIGHT: usize = 16;
const LINE_HEIGHT: usize = 22;
const LEFT: usize = 32;
const TOP: usize = 32;

const EMPTY_FRAMEBUFFER: FrameBufferInfo = FrameBufferInfo {
    addr: 0,
    size: 0,
    width: 0,
    height: 0,
    stride: 0,
    pixel_format: PixelFormat::Unknown,
};

struct Console {
    framebuffer: FrameBufferInfo,
    cursor_x: usize,
    cursor_y: usize,
    color: u32,
}

impl Console {
    const EMPTY: Self = Self {
        framebuffer: EMPTY_FRAMEBUFFER,
        cursor_x: 0,
        cursor_y: TOP,
        color: 0x00ff_ffff,
    };

    fn initialized(&self) -> bool {
        self.framebuffer.addr != 0
            && self.framebuffer.width != 0
            && self.framebuffer.height != 0
    }

    fn clear(&mut self, r: u8, g: u8, b: u8) {
        if !self.initialized() {
            return;
        }
        let fb = self.framebuffer;
        let color = compose(fb.pixel_format, r, g, b);
        let buffer = fb.addr as *mut u32;
        for y in 0..fb.height {
            for x in 0..fb.width {
                unsafe { core::ptr::write_volatile(buffer.add(y * fb.stride + x), color) };
            }
        }
        self.cursor_x = 0;
        self.cursor_y = TOP;
    }

    fn set_color(&mut self, r: u8, g: u8, b: u8) {
        self.color = compose(self.framebuffer.pixel_format, r, g, b);
    }

    fn write_text(&mut self, text: &str) {
        for byte in text.bytes() {
            self.write_byte(byte);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            8 => self.backspace(),
            byte => self.draw_char(byte.to_ascii_uppercase()),
        }
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += LINE_HEIGHT;
        if self.cursor_y + GLYPH_HEIGHT >= self.framebuffer.height {
            self.cursor_y = TOP;
        }
    }

    fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.erase_cell(self.cursor_x, self.cursor_y);
        }
    }

    fn erase_cell(&mut self, column: usize, y0: usize) {
        if !self.initialized() {
            return;
        }
        let fb = self.framebuffer;
        let buffer = fb.addr as *mut u32;
        let background = compose(fb.pixel_format, 0x0f, 0x17, 0x2a);
        let x0 = LEFT + column * GLYPH_WIDTH;
        let y_end = (y0 + GLYPH_HEIGHT).min(fb.height);
        let x_end = (x0 + GLYPH_WIDTH).min(fb.width);
        for y in y0..y_end {
            for x in x0..x_end {
                unsafe {
                    core::ptr::write_volatile(buffer.add(y * fb.stride + x), background)
                };
            }
        }
    }

    fn draw_char(&mut self, character: u8) {
        if !self.initialized() {
            return;
        }
        let fb = self.framebuffer;
        let mut x0 = LEFT + self.cursor_x * GLYPH_WIDTH;
        if x0 + GLYPH_WIDTH >= fb.width {
            self.newline();
            x0 = LEFT;
        }
        let y0 = self.cursor_y;
        if y0 + GLYPH_HEIGHT >= fb.height {
            return;
        }

        let buffer = fb.addr as *mut u32;
        for (row, bits) in font::glyph(character).iter().enumerate() {
            for column in 0..8 {
                if bits & (0x80 >> column) == 0 {
                    continue;
                }
                for scale_y in 0..SCALE {
                    for scale_x in 0..SCALE {
                        let y = y0 + row * SCALE + scale_y;
                        let x = x0 + column * SCALE + scale_x;
                        unsafe {
                            core::ptr::write_volatile(
                                buffer.add(y * fb.stride + x),
                                self.color,
                            )
                        };
                    }
                }
            }
        }
        self.cursor_x += 1;
    }
}

static CONSOLE: IrqSpinLock<Console> = IrqSpinLock::new(Console::EMPTY);

pub fn init(info: FrameBufferInfo) {
    let mut console = CONSOLE.lock();
    console.framebuffer = info;
    console.cursor_x = 0;
    console.cursor_y = TOP;
    console.color = compose(info.pixel_format, 255, 255, 255);
}

pub fn clear(r: u8, g: u8, b: u8) {
    CONSOLE.lock().clear(r, g, b);
}

pub fn clear_console() {
    clear(0, 0, 0);
}

pub fn set_color(r: u8, g: u8, b: u8) {
    CONSOLE.lock().set_color(r, g, b);
}

pub fn panic_header(reason: &str) {
    let mut console = CONSOLE.lock();
    console.clear(0x38, 8, 0x12);
    console.set_color(255, 0xd1, 0xd9);
    console.write_text("ALOHAOS KERNEL PANIC\n\n");
    console.set_color(255, 255, 255);
    console.write_text(reason);
    console.write_text("\n\n");
}

pub fn write_label_hex(label: &str, value: u64) {
    let mut console = CONSOLE.lock();
    console.write_text(label);
    console.write_text("0X");
    for shift in (0..16).rev() {
        let nibble = ((value >> (shift * 4)) & 15) as u8;
        console.draw_char(if nibble < 10 {
            b'0' + nibble
        } else {
            b'A' + nibble - 10
        });
    }
    console.newline();
}

pub fn write_label_dec(label: &str, mut value: u64) {
    let mut console = CONSOLE.lock();
    console.write_text(label);
    if value == 0 {
        console.draw_char(b'0');
        console.newline();
        return;
    }
    let mut digits = [0u8; 20];
    let mut length = 0;
    while value != 0 {
        digits[length] = b'0' + (value % 10) as u8;
        length += 1;
        value /= 10;
    }
    for &digit in digits[..length].iter().rev() {
        console.draw_char(digit);
    }
    console.newline();
}

pub fn write_line(text: &str) {
    let mut console = CONSOLE.lock();
    console.write_text(text);
    console.newline();
}

pub fn write_text(text: &str) {
    CONSOLE.lock().write_text(text);
}

pub fn write_byte(byte: u8) {
    CONSOLE.lock().write_byte(byte);
}

fn compose(format: PixelFormat, r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    match format {
        PixelFormat::Rgb => (b << 16) | (g << 8) | r,
        _ => (r << 16) | (g << 8) | b,
    }
}
