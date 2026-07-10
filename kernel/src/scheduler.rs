//! Scheduler task lifecycle and conservative timer staging.
//!
//! The state machine is real and timer-driven wakeups are active. Cross-stack
//! switching remains deliberately disabled until per-task kernel stacks and the
//! complete x86_64 extended context are saved and restored together.

use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};

const TASK_COUNT: usize = 2;
const NO_WAKE_DEADLINE: u64 = u64::MAX;

static PREEMPTION_TICKS: AtomicU64 = AtomicU64::new(0);
static STATES: [AtomicU8; TASK_COUNT] = [
 AtomicU8::new(TaskState::Running as u8),
 AtomicU8::new(TaskState::Blocked as u8),
];
static WAKE_TICKS: [AtomicU64; TASK_COUNT] = [
 AtomicU64::new(NO_WAKE_DEADLINE),
 AtomicU64::new(NO_WAKE_DEADLINE),
];
static SWITCHES: [AtomicU64; TASK_COUNT] = [AtomicU64::new(0), AtomicU64::new(0)];

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
 STATES[0].store(TaskState::Running as u8, Ordering::Release);
 STATES[1].store(TaskState::Blocked as u8, Ordering::Release);
 for task in 0..TASK_COUNT {
  WAKE_TICKS[task].store(NO_WAKE_DEADLINE, Ordering::Relaxed);
  SWITCHES[task].store(0, Ordering::Relaxed);
 }
}

pub fn state(task: usize) -> Option<TaskState> {
 STATES.get(task).map(|value| TaskState::from_raw(value.load(Ordering::Acquire)))
}

/// Move a blocked task back to the ready queue.
pub fn wake(task: usize) -> bool {
 transition(task, TaskState::Blocked, TaskState::Ready)
  || transition(task, TaskState::Sleeping, TaskState::Ready)
}

/// Block a ready or running task until an explicit wakeup.
pub fn block(task: usize) -> bool {
 WAKE_TICKS.get(task).map(|value| value.store(NO_WAKE_DEADLINE, Ordering::Relaxed));
 transition(task, TaskState::Ready, TaskState::Blocked)
  || transition(task, TaskState::Running, TaskState::Blocked)
}

/// Sleep a ready or running task until the absolute PIT tick is reached.
pub fn sleep_until(task: usize, wake_tick: u64) -> bool {
 let Some(deadline) = WAKE_TICKS.get(task) else { return false };
 let changed = transition(task, TaskState::Ready, TaskState::Sleeping)
  || transition(task, TaskState::Running, TaskState::Sleeping);
 if changed {
  deadline.store(wake_tick, Ordering::Release);
 }
 changed
}

/// Mark a task permanently dead. Dead tasks cannot be transitioned again.
pub fn exit(task: usize) -> bool {
 let Some(slot) = STATES.get(task) else { return false };
 loop {
  let current = TaskState::from_raw(slot.load(Ordering::Acquire));
  if current == TaskState::Dead { return false; }
  if slot.compare_exchange_weak(
   current as u8,
   TaskState::Dead as u8,
   Ordering::AcqRel,
   Ordering::Acquire,
  ).is_ok() {
   WAKE_TICKS[task].store(NO_WAKE_DEADLINE, Ordering::Release);
   return true;
  }
 }
}

fn transition(task: usize, from: TaskState, to: TaskState) -> bool {
 let Some(slot) = STATES.get(task) else { return false };
 slot.compare_exchange(from as u8, to as u8, Ordering::AcqRel, Ordering::Acquire).is_ok()
}

/// Timer scheduling hook. Sleeping tasks are woken at their deadline, while the
/// interrupted stack is returned unchanged until full context switching lands.
#[no_mangle]
pub extern "C" fn scheduler_on_timer_tick(current_stack: u64, tick: u64) -> u64 {
 PREEMPTION_TICKS.fetch_add(1, Ordering::Relaxed);
 SWITCHES[0].fetch_add(1, Ordering::Relaxed);

 for task in 0..TASK_COUNT {
  if state(task) == Some(TaskState::Sleeping)
   && WAKE_TICKS[task].load(Ordering::Acquire) <= tick
  {
   let _ = transition(task, TaskState::Sleeping, TaskState::Ready);
   WAKE_TICKS[task].store(NO_WAKE_DEADLINE, Ordering::Release);
  }
 }
 current_stack
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
