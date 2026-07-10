//! Two-task round-robin scheduler with an explicit runtime safety gate.
//!
//! The stable boot path does not touch XSAVE or the worker frame. Context
//! infrastructure is initialized lazily when the operator runs `sched on`.

use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use crate::{serial, task_context, task_stacks};

const TASK_COUNT: usize = 2;
const NO_WAKE_DEADLINE: u64 = u64::MAX;
const KERNEL_CODE_SELECTOR: u64 = 0x08;
const KERNEL_DATA_SELECTOR: u64 = 0x10;

static PREEMPTION_TICKS: AtomicU64 = AtomicU64::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PREEMPTION_ENABLED: AtomicBool = AtomicBool::new(false);
static WORKER_PREPARED: AtomicBool = AtomicBool::new(false);
static STATES: [AtomicU8; TASK_COUNT] = [
    AtomicU8::new(TaskState::Running as u8),
    AtomicU8::new(TaskState::Blocked as u8),
];
static WAKE_TICKS: [AtomicU64; TASK_COUNT] = [
    AtomicU64::new(NO_WAKE_DEADLINE),
    AtomicU64::new(NO_WAKE_DEADLINE),
];
static SWITCHES: [AtomicU64; TASK_COUNT] = [AtomicU64::new(0), AtomicU64::new(0)];
static WORKER_HEARTBEAT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TaskState {
    Ready = 0,
    Running = 1,
    Blocked = 2,
    Sleeping = 3,
    Dead = 4,
}

impl TaskState {
    fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Ready,
            1 => Self::Running,
            2 => Self::Blocked,
            3 => Self::Sleeping,
            4 => Self::Dead,
            _ => Self::Dead,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TaskSnapshot {
    pub id: usize,
    pub state: TaskState,
    pub switches: u64,
    pub wake_tick: Option<u64>,
}

pub fn init() {
    PREEMPTION_TICKS.store(0, Ordering::Relaxed);
    CURRENT.store(0, Ordering::Relaxed);
    PREEMPTION_ENABLED.store(false, Ordering::Release);
    WORKER_PREPARED.store(false, Ordering::Release);
    WORKER_HEARTBEAT.store(0, Ordering::Relaxed);
    STATES[0].store(TaskState::Running as u8, Ordering::Release);
    STATES[1].store(TaskState::Blocked as u8, Ordering::Release);
    for task in 0..TASK_COUNT {
        WAKE_TICKS[task].store(NO_WAKE_DEADLINE, Ordering::Relaxed);
        SWITCHES[task].store(0, Ordering::Relaxed);
    }
    serial::info(format_args!("scheduler: stable path ready, preemption gated"));
}

fn prepare_worker() -> bool {
    if WORKER_PREPARED.load(Ordering::Acquire) {
        return true;
    }
    if !task_context::init() {
        serial::error(format_args!("scheduler: context initialization failed"));
        return false;
    }
    let Some(stack_top) = task_stacks::stack_top(1) else {
        serial::error(format_args!("scheduler: worker stack unavailable"));
        return false;
    };
    let prepared = unsafe {
        task_context::prepare_kernel_task(
            1,
            stack_top,
            worker_task as *const () as u64,
            task_returned as *const () as u64,
            KERNEL_CODE_SELECTOR,
            KERNEL_DATA_SELECTOR,
        )
    };
    if !prepared {
        serial::error(format_args!("scheduler: worker frame unavailable"));
        return false;
    }
    STATES[1].store(TaskState::Ready as u8, Ordering::Release);
    WORKER_PREPARED.store(true, Ordering::Release);
    serial::info(format_args!("scheduler: worker prepared"));
    true
}

extern "C" fn worker_task() -> ! {
    loop {
        WORKER_HEARTBEAT.fetch_add(1, Ordering::Relaxed);
        unsafe { asm!("sti", "hlt", options(nomem, nostack)) };
    }
}

extern "C" fn task_returned() -> ! {
    PREEMPTION_ENABLED.store(false, Ordering::Release);
    let _ = exit(CURRENT.load(Ordering::Acquire));
    loop {
        unsafe { asm!("cli", "hlt", options(nomem, nostack)) };
    }
}

pub fn state(task: usize) -> Option<TaskState> {
    STATES
        .get(task)
        .map(|value| TaskState::from_raw(value.load(Ordering::Acquire)))
}

pub fn wake(task: usize) -> bool {
    transition(task, TaskState::Blocked, TaskState::Ready)
        || transition(task, TaskState::Sleeping, TaskState::Ready)
}

pub fn block(task: usize) -> bool {
    if let Some(value) = WAKE_TICKS.get(task) {
        value.store(NO_WAKE_DEADLINE, Ordering::Relaxed);
    }
    transition(task, TaskState::Ready, TaskState::Blocked)
        || transition(task, TaskState::Running, TaskState::Blocked)
}

pub fn sleep_until(task: usize, wake_tick: u64) -> bool {
    let Some(deadline) = WAKE_TICKS.get(task) else { return false };
    let changed = transition(task, TaskState::Ready, TaskState::Sleeping)
        || transition(task, TaskState::Running, TaskState::Sleeping);
    if changed {
        deadline.store(wake_tick, Ordering::Release);
    }
    changed
}

pub fn exit(task: usize) -> bool {
    let Some(slot) = STATES.get(task) else { return false };
    loop {
        let current = TaskState::from_raw(slot.load(Ordering::Acquire));
        if current == TaskState::Dead {
            return false;
        }
        if slot
            .compare_exchange_weak(
                current as u8,
                TaskState::Dead as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            WAKE_TICKS[task].store(NO_WAKE_DEADLINE, Ordering::Release);
            return true;
        }
    }
}

fn transition(task: usize, from: TaskState, to: TaskState) -> bool {
    let Some(slot) = STATES.get(task) else { return false };
    slot.compare_exchange(
        from as u8,
        to as u8,
        Ordering::AcqRel,
        Ordering::Acquire,
    )
    .is_ok()
}

/// The shell calls this from task 0. Disabling from another task would strand
/// the shell, so the gate deliberately rejects that transition.
pub fn set_preemption(enabled: bool) -> bool {
    if CURRENT.load(Ordering::Acquire) != 0 {
        return false;
    }
    if enabled {
        if !prepare_worker() || state(1) != Some(TaskState::Ready) {
            return false;
        }
        PREEMPTION_ENABLED.store(true, Ordering::Release);
    } else {
        PREEMPTION_ENABLED.store(false, Ordering::Release);
    }
    true
}

#[no_mangle]
pub extern "C" fn scheduler_on_timer_tick(stack: u64, tick: u64) -> u64 {
    PREEMPTION_TICKS.fetch_add(1, Ordering::Relaxed);
    for task in 0..TASK_COUNT {
        if state(task) == Some(TaskState::Sleeping)
            && WAKE_TICKS[task].load(Ordering::Acquire) <= tick
        {
            let _ = transition(task, TaskState::Sleeping, TaskState::Ready);
            WAKE_TICKS[task].store(NO_WAKE_DEADLINE, Ordering::Release);
        }
    }

    if !PREEMPTION_ENABLED.load(Ordering::Acquire) {
        return stack;
    }

    let current = CURRENT.load(Ordering::Relaxed);
    let mut next = current;
    for offset in 1..=TASK_COUNT {
        let candidate = (current + offset) % TASK_COUNT;
        if matches!(state(candidate), Some(TaskState::Ready) | Some(TaskState::Running)) {
            next = candidate;
            break;
        }
    }
    if next == current {
        return stack;
    }

    if state(current) == Some(TaskState::Running) {
        let _ = transition(current, TaskState::Running, TaskState::Ready);
    }
    if state(next) == Some(TaskState::Ready) {
        let _ = transition(next, TaskState::Ready, TaskState::Running);
    }
    CURRENT.store(next, Ordering::Release);
    SWITCHES[next].fetch_add(1, Ordering::Relaxed);

    unsafe { task_context::switch(current, next, stack) }
}

pub fn snapshots() -> [TaskSnapshot; TASK_COUNT] {
    core::array::from_fn(|id| {
        let deadline = WAKE_TICKS[id].load(Ordering::Acquire);
        TaskSnapshot {
            id,
            state: state(id).unwrap_or(TaskState::Dead),
            switches: SWITCHES[id].load(Ordering::Relaxed),
            wake_tick: (deadline != NO_WAKE_DEADLINE).then_some(deadline),
        }
    })
}

pub fn heartbeat() -> u64 {
    PREEMPTION_TICKS.load(Ordering::Relaxed)
}

pub fn worker_heartbeat() -> u64 {
    WORKER_HEARTBEAT.load(Ordering::Relaxed)
}

pub fn context_switching_ready() -> bool {
    PREEMPTION_ENABLED.load(Ordering::Acquire)
}
