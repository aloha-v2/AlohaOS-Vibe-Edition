//! Small IRQ-safe synchronization primitives for the single-core kernel.
//!
//! Lock acquisition saves RFLAGS and disables interrupts before spinning. This
//! prevents a same-CPU interrupt handler from deadlocking on a lock held by the
//! code it interrupted. Dropping the guard restores the caller's IF state.

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
unsafe impl<T: Send> Send for IrqSpinLock<T> {}

impl<T> IrqSpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self { locked: AtomicBool::new(false), value: UnsafeCell::new(value) }
    }

    pub fn lock(&self) -> IrqSpinLockGuard<'_, T> {
        let interrupts_were_enabled = save_and_disable_interrupts();
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            spin_loop();
        }
        IrqSpinLockGuard { lock: self, interrupts_were_enabled }
    }
}

pub struct IrqSpinLockGuard<'a, T> {
    lock: &'a IrqSpinLock<T>,
    interrupts_were_enabled: bool,
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
        if self.interrupts_were_enabled {
            unsafe { asm!("sti", options(nomem, nostack, preserves_flags)) };
        }
    }
}

#[inline]
fn save_and_disable_interrupts() -> bool {
    let flags: u64;
    unsafe {
        asm!("pushfq", "pop {}", out(reg) flags, options(nomem, preserves_flags));
        asm!("cli", options(nomem, nostack, preserves_flags));
    }
    flags & RFLAGS_INTERRUPT_ENABLE != 0
}
