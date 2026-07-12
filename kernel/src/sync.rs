//! IRQ-safe and scheduler-aware synchronization primitives.
//!
//! `IrqSpinLock` is for tiny critical sections that may be reached from an
//! interrupt handler. `Mutex`, `Semaphore` and `WaitQueue` are task-context
//! primitives: they keep interrupts enabled and park the current task instead
//! of burning a CPU while another task owns a resource.

use core::arch::asm;
use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use crate::scheduler;

const RFLAGS_INTERRUPT_ENABLE: u64 = 1 << 9;

pub struct IrqSpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for IrqSpinLock<T> {}
unsafe impl<T: Send> Send for IrqSpinLock<T> {}

impl<T> IrqSpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
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
        IrqSpinLockGuard {
            lock: self,
            interrupts_were_enabled,
        }
    }
}

pub struct IrqSpinLockGuard<'a, T> {
    lock: &'a IrqSpinLock<T>,
    interrupts_were_enabled: bool,
}

impl<T> Deref for IrqSpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for IrqSpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for IrqSpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
        if self.interrupts_were_enabled {
            unsafe { asm!("sti", options(nomem, nostack, preserves_flags)) };
        }
    }
}

/// A scheduler wait queue represented as a bit per kernel task.
///
/// Registration happens before the condition is checked a second time, which
/// closes the classic lost-wakeup window between observing a busy resource and
/// parking the current task.
pub struct WaitQueue {
    waiters: AtomicU64,
}

impl WaitQueue {
    pub const fn new() -> Self {
        Self {
            waiters: AtomicU64::new(0),
        }
    }

    pub fn wait_until<F>(&self, mut condition: F)
    where
        F: FnMut() -> bool,
    {
        loop {
            if condition() {
                return;
            }

            // Blocking while IF=0 cannot make progress. Callers in interrupt
            // context degrade to a spin rather than corrupting task state.
            if !interrupts_enabled() {
                spin_loop();
                continue;
            }

            let task = scheduler::current_task();
            let bit = 1u64 << task;
            self.waiters.fetch_or(bit, Ordering::AcqRel);

            if condition() {
                self.waiters.fetch_and(!bit, Ordering::AcqRel);
                return;
            }

            if !scheduler::park_current() {
                self.waiters.fetch_and(!bit, Ordering::AcqRel);
                spin_loop();
                continue;
            }

            self.waiters.fetch_and(!bit, Ordering::AcqRel);
        }
    }

    pub fn wake_one(&self) -> bool {
        loop {
            let waiters = self.waiters.load(Ordering::Acquire);
            if waiters == 0 {
                return false;
            }
            let task = waiters.trailing_zeros() as usize;
            let bit = 1u64 << task;
            if self
                .waiters
                .compare_exchange_weak(
                    waiters,
                    waiters & !bit,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                let _ = scheduler::wake(task);
                return true;
            }
        }
    }

    pub fn wake_all(&self) -> usize {
        let mut waiters = self.waiters.swap(0, Ordering::AcqRel);
        let mut woken = 0;
        while waiters != 0 {
            let task = waiters.trailing_zeros() as usize;
            waiters &= !(1u64 << task);
            if scheduler::wake(task) {
                woken += 1;
            }
        }
        woken
    }
}

impl Default for WaitQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Mutex<T> {
    locked: AtomicBool,
    waiters: WaitQueue,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            waiters: WaitQueue::new(),
            value: UnsafeCell::new(value),
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| MutexGuard { mutex: self })
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        loop {
            if let Some(guard) = self.try_lock() {
                return guard;
            }
            self.waiters
                .wait_until(|| !self.locked.load(Ordering::Acquire));
        }
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
        self.mutex.waiters.wake_one();
    }
}

pub struct Semaphore {
    permits: AtomicUsize,
    waiters: WaitQueue,
}

impl Semaphore {
    pub const fn new(permits: usize) -> Self {
        Self {
            permits: AtomicUsize::new(permits),
            waiters: WaitQueue::new(),
        }
    }

    pub fn try_acquire(&self) -> bool {
        self.permits
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |permits| {
                permits.checked_sub(1)
            })
            .is_ok()
    }

    pub fn acquire(&self) {
        loop {
            if self.try_acquire() {
                return;
            }
            self.waiters
                .wait_until(|| self.permits.load(Ordering::Acquire) != 0);
        }
    }

    /// Add permits, returning false without changing state on overflow.
    pub fn release(&self, permits: usize) -> bool {
        if permits == 0 {
            return true;
        }
        if self
            .permits
            .fetch_update(Ordering::Release, Ordering::Relaxed, |current| {
                current.checked_add(permits)
            })
            .is_err()
        {
            return false;
        }
        self.waiters.wake_all();
        true
    }

    pub fn available(&self) -> usize {
        self.permits.load(Ordering::Acquire)
    }
}

#[inline]
fn interrupts_enabled() -> bool {
    read_rflags() & RFLAGS_INTERRUPT_ENABLE != 0
}

#[inline]
fn save_and_disable_interrupts() -> bool {
    let enabled = interrupts_enabled();
    unsafe { asm!("cli", options(nomem, nostack, preserves_flags)) };
    enabled
}

#[inline]
fn read_rflags() -> u64 {
    let flags: u64;
    unsafe {
        asm!("pushfq", "pop {}", out(reg) flags, options(nomem, preserves_flags));
    }
    flags
}
