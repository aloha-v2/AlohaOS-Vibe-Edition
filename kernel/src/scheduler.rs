//! Scheduler staging layer.
//!
//! PIT preemption remains active, but cross-stack context switching is disabled
//! until the interrupt frame, FPU state and per-task kernel stacks are modeled
//! together. Returning the interrupted stack keeps the kernel stable instead of
//! pretending that a partial context switch is production-safe.

use core::sync::atomic::{AtomicU64, Ordering};

const TASK_COUNT: usize = 2;
static PREEMPTION_TICKS: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Blocked,
}

pub struct TaskSnapshot {
    pub id: usize,
    pub state: TaskState,
    pub switches: u64,
}

pub fn init() {
    PREEMPTION_TICKS.store(0, Ordering::Relaxed);
}

/// Timer scheduling hook. For now it records scheduling opportunities and
/// resumes the exact interrupted context. This is intentionally conservative:
/// the previous implementation switched only GPRs and RSP, which is not a
/// complete x86_64 task context and repeatedly escalated into #DF.
#[no_mangle]
pub extern "C" fn scheduler_on_timer_tick(current_stack: u64, _tick: u64) -> u64 {
    PREEMPTION_TICKS.fetch_add(1, Ordering::Relaxed);
    current_stack
}

pub fn snapshots() -> [TaskSnapshot; TASK_COUNT] {
    [
        TaskSnapshot {
            id: 0,
            state: TaskState::Running,
            switches: PREEMPTION_TICKS.load(Ordering::Relaxed),
        },
        TaskSnapshot {
            id: 1,
            state: TaskState::Blocked,
            switches: 0,
        },
    ]
}

pub fn heartbeat() -> u64 {
    PREEMPTION_TICKS.load(Ordering::Relaxed)
}
