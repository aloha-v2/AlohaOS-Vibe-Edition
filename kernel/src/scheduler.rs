//! Preemptive two-task round-robin scheduler.

use core::arch::asm;
use core::mem::size_of;
use core::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering};

use crate::{task_context, task_stacks};

const TASK_COUNT: usize = 2;
const NO_WAKE_DEADLINE: u64 = u64::MAX;
const KERNEL_CODE_SELECTOR: u64 = 0x08;
const INTERRUPT_FLAGS: u64 = 0x202;

static PREEMPTION_TICKS: AtomicU64 = AtomicU64::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(0);
static SAVED_STACKS: [AtomicU64; TASK_COUNT] = [AtomicU64::new(0), AtomicU64::new(0)];
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
pub enum TaskState { Ready = 0, Running = 1, Blocked = 2, Sleeping = 3, Dead = 4 }
impl TaskState {
 fn from_raw(value: u8) -> Self { match value { 0=>Self::Ready,1=>Self::Running,2=>Self::Blocked,3=>Self::Sleeping,4=>Self::Dead,_=>Self::Dead } }
}

#[derive(Clone, Copy)]
pub struct TaskSnapshot { pub id: usize, pub state: TaskState, pub switches: u64, pub wake_tick: Option<u64> }

#[repr(C)]
struct InterruptFrame {
 r15:u64,r14:u64,r13:u64,r12:u64,rbp:u64,rbx:u64,r11:u64,r10:u64,r9:u64,r8:u64,
 rdi:u64,rsi:u64,rdx:u64,rcx:u64,rax:u64,rip:u64,cs:u64,rflags:u64,
}

pub fn init() {
 PREEMPTION_TICKS.store(0, Ordering::Relaxed);
 CURRENT.store(0, Ordering::Relaxed);
 STATES[0].store(TaskState::Running as u8, Ordering::Release);
 for task in 0..TASK_COUNT {
  WAKE_TICKS[task].store(NO_WAKE_DEADLINE, Ordering::Relaxed);
  SWITCHES[task].store(0, Ordering::Relaxed);
  SAVED_STACKS[task].store(0, Ordering::Relaxed);
 }

 if task_context::init() {
  let stack = unsafe { build_initial_frame(1, worker_task as usize as u64) };
  if let Some(stack) = stack {
   SAVED_STACKS[1].store(stack, Ordering::Release);
   STATES[1].store(TaskState::Ready as u8, Ordering::Release);
   return;
  }
 }
 STATES[1].store(TaskState::Blocked as u8, Ordering::Release);
}

unsafe fn build_initial_frame(task: usize, entry: u64) -> Option<u64> {
 let top = task_stacks::stack_top(task)?;
 let stack = top.checked_sub(size_of::<InterruptFrame>() as u64)?;
 let frame = &mut *(stack as *mut InterruptFrame);
 *frame = InterruptFrame {
  r15:0,r14:0,r13:0,r12:0,rbp:0,rbx:0,r11:0,r10:0,r9:0,r8:0,
  rdi:0,rsi:0,rdx:0,rcx:0,rax:0,
  rip:entry,cs:KERNEL_CODE_SELECTOR,rflags:INTERRUPT_FLAGS,
 };
 Some(stack)
}

extern "C" fn worker_task() -> ! {
 loop {
  WORKER_HEARTBEAT.fetch_add(1, Ordering::Relaxed);
  unsafe { asm!("sti", "hlt", options(nomem, nostack)) };
 }
}

pub fn state(task: usize) -> Option<TaskState> { STATES.get(task).map(|v| TaskState::from_raw(v.load(Ordering::Acquire))) }
pub fn wake(task: usize) -> bool { transition(task,TaskState::Blocked,TaskState::Ready)||transition(task,TaskState::Sleeping,TaskState::Ready) }
pub fn block(task: usize) -> bool {
 if let Some(value)=WAKE_TICKS.get(task){value.store(NO_WAKE_DEADLINE,Ordering::Relaxed)}
 transition(task,TaskState::Ready,TaskState::Blocked)||transition(task,TaskState::Running,TaskState::Blocked)
}
pub fn sleep_until(task:usize,wake_tick:u64)->bool{
 let Some(deadline)=WAKE_TICKS.get(task)else{return false};
 let changed=transition(task,TaskState::Ready,TaskState::Sleeping)||transition(task,TaskState::Running,TaskState::Sleeping);
 if changed{deadline.store(wake_tick,Ordering::Release)} changed
}
pub fn exit(task:usize)->bool{
 let Some(slot)=STATES.get(task)else{return false};
 loop{let current=TaskState::from_raw(slot.load(Ordering::Acquire));if current==TaskState::Dead{return false}
 if slot.compare_exchange_weak(current as u8,TaskState::Dead as u8,Ordering::AcqRel,Ordering::Acquire).is_ok(){WAKE_TICKS[task].store(NO_WAKE_DEADLINE,Ordering::Release);return true}}
}
fn transition(task:usize,from:TaskState,to:TaskState)->bool{let Some(slot)=STATES.get(task)else{return false};slot.compare_exchange(from as u8,to as u8,Ordering::AcqRel,Ordering::Acquire).is_ok()}

#[no_mangle]
pub extern "C" fn scheduler_on_timer_tick(current_stack:u64,tick:u64)->u64{
 PREEMPTION_TICKS.fetch_add(1,Ordering::Relaxed);
 for task in 0..TASK_COUNT{if state(task)==Some(TaskState::Sleeping)&&WAKE_TICKS[task].load(Ordering::Acquire)<=tick{let _=transition(task,TaskState::Sleeping,TaskState::Ready);WAKE_TICKS[task].store(NO_WAKE_DEADLINE,Ordering::Release)}}

 let current=CURRENT.load(Ordering::Relaxed);
 SAVED_STACKS[current].store(current_stack,Ordering::Release);
 let next=(1..=TASK_COUNT).map(|offset|(current+offset)%TASK_COUNT).find(|&task|matches!(state(task),Some(TaskState::Ready)|Some(TaskState::Running))).unwrap_or(current);
 if next==current{return current_stack}
 if state(current)==Some(TaskState::Running){let _=transition(current,TaskState::Running,TaskState::Ready)}
 if state(next)==Some(TaskState::Ready){let _=transition(next,TaskState::Ready,TaskState::Running)}
 unsafe{task_context::switch(current,next)};
 CURRENT.store(next,Ordering::Release);
 SWITCHES[next].fetch_add(1,Ordering::Relaxed);
 SAVED_STACKS[next].load(Ordering::Acquire)
}

pub fn snapshots()->[TaskSnapshot;TASK_COUNT]{core::array::from_fn(|id|{let deadline=WAKE_TICKS[id].load(Ordering::Acquire);TaskSnapshot{id,state:state(id).unwrap_or(TaskState::Dead),switches:SWITCHES[id].load(Ordering::Relaxed),wake_tick:(deadline!=NO_WAKE_DEADLINE).then_some(deadline)}})}
pub fn heartbeat()->u64{PREEMPTION_TICKS.load(Ordering::Relaxed)}
pub fn worker_heartbeat()->u64{WORKER_HEARTBEAT.load(Ordering::Relaxed)}
pub fn context_switching_ready()->bool{task_context::is_ready()}
