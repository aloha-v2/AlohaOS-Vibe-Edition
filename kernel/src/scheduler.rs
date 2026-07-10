//! Minimal preemptive round-robin scheduler for kernel threads.
//!
//! PIT IRQ0 saves the general-purpose context. The timer ISR gives that stack
//! pointer to the scheduler and restores the selected task through `iretq`.

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

const TASK_COUNT: usize = 2;
const STACK_SIZE: usize = 64 * 1024;
const QUANTUM_TICKS: u64 = 5;
const KERNEL_CODE_SELECTOR: u64 = 0x08;
const INITIAL_RFLAGS: u64 = 0x202;
const SAVED_REGISTER_WORDS: usize = 15;
const IRET_FRAME_WORDS: usize = 3;
const CONTEXT_WORDS: usize = SAVED_REGISTER_WORDS + IRET_FRAME_WORDS;

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
        let next_stack = TASKS[next].stack_pointer;
        if next_stack == 0 {
            return current_stack;
        }

        TASKS[previous].state = TaskState::Ready;
        TASKS[next].state = TaskState::Running;
        TASKS[next].switches = TASKS[next].switches.wrapping_add(1);
        CURRENT_TASK = next;
        next_stack
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
        unsafe { asm!("sti", "hlt", options(nomem, nostack)) };
    }
}

unsafe fn build_initial_context(stack_start: *mut u8, entry: extern "C" fn() -> !) -> u64 {
    let aligned_top = (stack_start.add(STACK_SIZE) as usize) & !0xf;

    // A normal SysV function starts with RSP % 16 == 8 because CALL pushed a
    // return address. IRET does not do that, so reserve a fake return slot and
    // make the post-IRET stack match the ABI exactly. The old version entered
    // the Rust worker with RSP % 16 == 0, which could trigger an alignment #GP;
    // that fault during the timer switch escalated into the observed #DF.
    let initial_rsp = aligned_top - core::mem::size_of::<u64>();
    (initial_rsp as *mut u64).write(0);

    let context_start = initial_rsp - CONTEXT_WORDS * core::mem::size_of::<u64>();
    let context = context_start as *mut u64;

    // Layout consumed by the timer ISR: r15..rax, then RIP, CS and RFLAGS.
    for index in 0..SAVED_REGISTER_WORDS {
        context.add(index).write(0);
    }
    context.add(15).write(entry as *const () as u64);
    context.add(16).write(KERNEL_CODE_SELECTOR);
    context.add(17).write(INITIAL_RFLAGS);
    context_start as u64
}
