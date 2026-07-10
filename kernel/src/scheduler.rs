//! Minimal preemptive round-robin scheduler for kernel threads.
//!
//! PIT IRQ0 saves the complete general-purpose context. The assembly ISR gives
//! that stack pointer to `on_timer_tick`, which returns the stack of the task
//! that should resume through `iretq`.

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

const TASK_COUNT: usize = 2;
const STACK_SIZE: usize = 64 * 1024;
const QUANTUM_TICKS: u64 = 5;
const KERNEL_CODE_SELECTOR: u64 = 0x08;
const INITIAL_RFLAGS: u64 = 0x202;
const CONTEXT_WORDS: usize = 18;

#[repr(C, align(16))]
struct KernelStack([u8; STACK_SIZE]);

#[derive(Clone, Copy)]
struct Task {
    stack_pointer: u64,
    switches: u64,
    state: TaskState,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Ready,
}

impl Task {
    const EMPTY: Self = Self {
        stack_pointer: 0,
        switches: 0,
        state: TaskState::Ready,
    };
}

pub struct TaskSnapshot {
    pub id: usize,
    pub state: TaskState,
    pub switches: u64,
}

static mut WORKER_STACK: KernelStack = KernelStack([0; STACK_SIZE]);
static mut TASKS: [Task; TASK_COUNT] = [Task::EMPTY; TASK_COUNT];
static mut CURRENT_TASK: usize = 0;
static mut INITIALIZED: bool = false;
static BACKGROUND_HEARTBEAT: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    unsafe {
        TASKS[0] = Task {
            stack_pointer: 0,
            switches: 0,
            state: TaskState::Running,
        };
        TASKS[1] = Task {
            stack_pointer: build_initial_context(
                core::ptr::addr_of_mut!(WORKER_STACK.0).cast::<u8>(),
                background_worker,
            ),
            switches: 0,
            state: TaskState::Ready,
        };
        CURRENT_TASK = 0;
        INITIALIZED = true;
    }
}

/// Called on every timer interrupt with the stack produced by the IRQ stub.
/// It returns the stack that the stub must restore before `iretq`.
#[no_mangle]
pub extern "C" fn scheduler_on_timer_tick(current_stack: u64, tick: u64) -> u64 {
    unsafe {
        if !INITIALIZED {
            return current_stack;
        }

        TASKS[CURRENT_TASK].stack_pointer = current_stack;
        if tick % QUANTUM_TICKS != 0 {
            return current_stack;
        }

        let previous = CURRENT_TASK;
        let next = (CURRENT_TASK + 1) % TASK_COUNT;
        TASKS[previous].state = TaskState::Ready;
        TASKS[next].state = TaskState::Running;
        TASKS[next].switches = TASKS[next].switches.wrapping_add(1);
        CURRENT_TASK = next;
        TASKS[next].stack_pointer
    }
}

pub fn snapshots() -> [TaskSnapshot; TASK_COUNT] {
    unsafe {
        [
            TaskSnapshot { id: 0, state: TASKS[0].state, switches: TASKS[0].switches },
            TaskSnapshot { id: 1, state: TASKS[1].state, switches: TASKS[1].switches },
        ]
    }
}

pub fn heartbeat() -> u64 {
    BACKGROUND_HEARTBEAT.load(Ordering::Relaxed)
}

extern "C" fn background_worker() -> ! {
    loop {
        BACKGROUND_HEARTBEAT.fetch_add(1, Ordering::Relaxed);
        // Sleep until IRQ0 wakes this task. The next timer interrupt can then
        // preempt it and resume the shell task.
        unsafe { asm!("sti", "hlt", options(nomem, nostack)) };
    }
}

unsafe fn build_initial_context(stack_start: *mut u8, entry: extern "C" fn() -> !) -> u64 {
    let stack_top = stack_start.add(STACK_SIZE) as usize;
    let context_start = (stack_top - CONTEXT_WORDS * core::mem::size_of::<u64>()) & !0xf;
    let context = context_start as *mut u64;

    // General registers in the exact order consumed by the timer ISR pops:
    // r15..rax. Zero is a valid initial value for all of them.
    for index in 0..15 {
        context.add(index).write(0);
    }
    context.add(15).write(entry as *const () as u64); // RIP
    context.add(16).write(KERNEL_CODE_SELECTOR);      // CS
    context.add(17).write(INITIAL_RFLAGS);            // RFLAGS, IF set
    context_start as u64
}
