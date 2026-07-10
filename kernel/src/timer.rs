//! PIT channel 0 clock running at 100 Hz.

use core::sync::atomic::{AtomicU64, Ordering};
use crate::pic;

pub const HZ: u64 = 100;
static TICKS: AtomicU64 = AtomicU64::new(0);

pub unsafe fn init() {
    let divisor = (1_193_182u32 / HZ as u32) as u16;
    pic::outb(0x43, 0x36);
    pic::outb(0x40, divisor as u8);
    pic::outb(0x40, (divisor >> 8) as u8);
}

pub fn interrupt() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    unsafe { pic::end_of_interrupt(pic::TIMER_VECTOR) };
}

pub fn ticks() -> u64 { TICKS.load(Ordering::Relaxed) }
pub fn seconds() -> u64 { ticks() / HZ }
pub fn centiseconds() -> u64 { ticks() % HZ }
