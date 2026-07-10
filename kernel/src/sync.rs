//! Minimal single-core IRQ-safe synchronization primitives.

use core::arch::asm;
use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

const RFLAGS_INTERRUPT_ENABLE: u64 = 1 << 9;

pub struct IrqSpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for IrqSpinLock<T> {}

impl<T> IrqSpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self { locked: AtomicBool::new(false), value: UnsafeCell::new(value) }
    }

    pub fn lock(&self) -> IrqSpinLockGuard<'_, T> {
        let flags = disable_interrupts();
        while self.locked.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_err() {
            spin_loop();
        }
        IrqSpinLockGuard { lock: self, restore_interrupts: flags & RFLAGS_INTERRUPT_ENABLE != 0 }
    }
}

pub struct IrqSpinLockGuard<'a, T> {
    lock: &'a IrqSpinLock<T>,
    restore_interrupts: bool,
}

impl<T> Deref for IrqSpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T { unsafe { &*self.lock.value.get() } }
}

impl<T> DerefMut for IrqSpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T { unsafe { &mut *self.lock.value.get() } }
}

impl<T> Drop for IrqSpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
        if self.restore_interrupts {
            unsafe { asm!("sti", options(nomem, nostack, preserves_flags)) };
        }
    }
}

#[inline]
fn disable_interrupts() -> u64 {
    let flags: u64;
    unsafe {
        asm!(
            "pushfq",
            "pop {}",
            "cli",
            out(reg) flags,
            options(nomem, preserves_flags),
        );
    }
    flags
}
