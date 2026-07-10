//! Allocation-free COM1 logger for early boot and panic diagnostics.

use core::arch::asm;
use core::fmt::{self, Write};
use core::hint::spin_loop;
use crate::sync::IrqSpinLock;

const COM1: u16 = 0x3f8;
const TX_READY: u8 = 1 << 5;
const MAX_TX_SPINS: usize = 100_000;
static WRITER: IrqSpinLock<SerialWriter> = IrqSpinLock::new(SerialWriter);

pub unsafe fn init() {
 outb(COM1 + 1, 0x00);outb(COM1 + 3, 0x80);outb(COM1, 0x01);outb(COM1 + 1, 0x00);
 outb(COM1 + 3, 0x03);outb(COM1 + 2, 0xc7);outb(COM1 + 4, 0x0b);
}

pub fn debug(args: fmt::Arguments<'_>) { log("DEBUG", args) }
pub fn info(args: fmt::Arguments<'_>) { log("INFO", args) }
pub fn error(args: fmt::Arguments<'_>) { log("ERROR", args) }

fn log(level: &str, args: fmt::Arguments<'_>) {
 let mut writer = WRITER.lock();
 let _ = writeln!(writer, "[{}] {}", level, args);
}

/// Best-effort panic output bypasses the lock because panic may interrupt a
/// writer while interrupts are already disabled.
pub fn emergency(args: fmt::Arguments<'_>) {
 let mut writer = SerialWriter;
 let _ = writeln!(writer, "[ERROR] {}", args);
}

struct SerialWriter;
impl Write for SerialWriter {
 fn write_str(&mut self, text: &str) -> fmt::Result {
  for byte in text.bytes() { if byte == b'\n' { write_byte(b'\r'); } write_byte(byte); }
  Ok(())
 }
}
fn write_byte(byte: u8) { for _ in 0..MAX_TX_SPINS { if unsafe { inb(COM1 + 5) } & TX_READY != 0 { unsafe { outb(COM1, byte) };return; }spin_loop(); } }
#[inline]unsafe fn outb(port:u16,value:u8){asm!("out dx, al",in("dx")port,in("al")value,options(nomem,nostack,preserves_flags));}
#[inline]unsafe fn inb(port:u16)->u8{let value:u8;asm!("in al, dx",in("dx")port,out("al")value,options(nomem,nostack,preserves_flags));value}
