//! Minimal COM1 kernel logger with severity levels.
//!
//! The logger intentionally has no allocator dependency, so it is available
//! from the first instructions of the kernel entry point and from panic paths.

use core::arch::asm;
use core::fmt::{self, Write};
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, Ordering};

const COM1: u16 = 0x3f8;
const TX_READY: u8 = 1 << 5;
const MAX_TX_SPINS: usize = 100_000;

static LOCKED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    const fn label(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

/// Configure COM1 for 115200 baud, 8 data bits, no parity, one stop bit.
///
/// # Safety
/// Must only be called while the caller owns early platform initialization.
pub unsafe fn init() {
    outb(COM1 + 1, 0x00); // Disable UART interrupts.
    outb(COM1 + 3, 0x80); // Enable divisor latch access.
    outb(COM1, 0x01); // Divisor 1: 115200 baud.
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x03); // 8N1.
    outb(COM1 + 2, 0xc7); // Enable and clear FIFO, 14-byte threshold.
    outb(COM1 + 4, 0x0b); // DTR, RTS and OUT2.
}

/// Write one structured kernel log record.
pub fn log(level: Level, args: fmt::Arguments<'_>) {
    while LOCKED
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        spin_loop();
    }

    let mut writer = SerialWriter;
    let _ = writeln!(writer, "[{}] {}", level.label(), args);
    LOCKED.store(false, Ordering::Release);
}

/// Panic-safe best-effort output that bypasses the logger lock.
/// This avoids deadlocking if a panic interrupts an active log record.
pub fn emergency_log(level: Level, args: fmt::Arguments<'_>) {
    let mut writer = SerialWriter;
    let _ = writeln!(writer, "[{}] {}", level.label(), args);
}

struct SerialWriter;

impl Write for SerialWriter {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            if byte == b'\n' {
                write_byte(b'\r');
            }
            write_byte(byte);
        }
        Ok(())
    }
}

fn write_byte(byte: u8) {
    for _ in 0..MAX_TX_SPINS {
        if unsafe { inb(COM1 + 5) } & TX_READY != 0 {
            unsafe { outb(COM1, byte) };
            return;
        }
        spin_loop();
    }
}

#[inline]
unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags),
    );
}

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags),
    );
    value
}
