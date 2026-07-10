//! Extended x86_64 task context with an assembly-only switch trampoline.
//!
//! Timer interrupts arrive on a shared IST stack. Before switching tasks the
//! trampoline copies the complete GPR + IRET frame into a persistent per-task
//! slot, then saves/restores CR3, FS/GS and XSAVE state without re-entering Rust.

use core::arch::{asm, global_asm, x86_64::__cpuid_count};
use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::serial;

const TASK_COUNT: usize = 2;
const XSTATE_CAPACITY: usize = 16 * 1024;
const CONTEXT_XSTATE_OFFSET: usize = 64;
const FRAME_QWORDS: usize = 20;
const INTERRUPT_FLAGS: u64 = 0x202;

#[repr(C, align(64))]
struct Xstate([u8; XSTATE_CAPACITY]);

/// Keep these offsets in sync with the assembly below.
#[repr(C, align(64))]
struct TaskContext {
    saved_frame: u64, // 0
    cr3: u64,         // 8
    fs_base: u64,     // 16
    gs_base: u64,     // 24
    xcr0_mask: u64,   // 32
    _padding: [u8; 24],
    xstate: Xstate,   // 64
}

impl TaskContext {
    const EMPTY: Self = Self {
        saved_frame: 0,
        cr3: 0,
        fs_base: 0,
        gs_base: 0,
        xcr0_mask: 0,
        _padding: [0; 24],
        xstate: Xstate([0; XSTATE_CAPACITY]),
    };
}

/// Layout consumed by the timer ISR: 15 GPRs followed by the five qwords the
/// CPU pushes when an IST stack switch happens (RIP, CS, RFLAGS, RSP, SS).
#[repr(C, align(16))]
struct InterruptFrame([u64; FRAME_QWORDS]);

struct ContextSlots([UnsafeCell<TaskContext>; TASK_COUNT]);
unsafe impl Sync for ContextSlots {}
struct FrameSlots([UnsafeCell<InterruptFrame>; TASK_COUNT]);
unsafe impl Sync for FrameSlots {}

static CONTEXTS: ContextSlots = ContextSlots([
    UnsafeCell::new(TaskContext::EMPTY),
    UnsafeCell::new(TaskContext::EMPTY),
]);
static FRAMES: FrameSlots = FrameSlots([
    UnsafeCell::new(InterruptFrame([0; FRAME_QWORDS])),
    UnsafeCell::new(InterruptFrame([0; FRAME_QWORDS])),
]);
static READY: AtomicBool = AtomicBool::new(false);
static XCR0_MASK: AtomicU64 = AtomicU64::new(0);

unsafe extern "C" {
    fn aloha_context_capture(context: *mut TaskContext);
    fn aloha_context_switch(
        current: *mut TaskContext,
        next: *const TaskContext,
        ist_frame: u64,
        persistent_frame: *mut InterruptFrame,
    ) -> u64;
}

pub fn init() -> bool {
    serial::debug(format_args!("context: cpuid"));
    let basic = __cpuid_count(1, 0);
    if basic.ecx & (1 << 26) == 0 {
        serial::error(format_args!("context: XSAVE unsupported"));
        return false;
    }

    unsafe {
        let mut cr4: u64;
        asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
        cr4 |= (1 << 9) | (1 << 10) | (1 << 18);
        asm!("mov cr4, {}", in(reg) cr4, options(nostack, preserves_flags));
    }

    let supported = __cpuid_count(0x0d, 0);
    let supported_mask = supported.eax as u64 | ((supported.edx as u64) << 32);
    let mut enabled = supported_mask & 0b11;
    if enabled != 0b11 {
        serial::error(format_args!("context: x87/SSE state unavailable"));
        return false;
    }
    if basic.ecx & (1 << 28) != 0 && supported_mask & (1 << 2) != 0 {
        enabled |= 1 << 2;
    }

    unsafe { xsetbv(enabled) };
    let configured = __cpuid_count(0x0d, 0);
    if configured.ebx as usize > XSTATE_CAPACITY {
        serial::error(format_args!("context: xstate buffer too small"));
        return false;
    }

    debug_assert_eq!(core::mem::offset_of!(TaskContext, xstate), CONTEXT_XSTATE_OFFSET);
    unsafe {
        (*CONTEXTS.0[0].get()).xcr0_mask = enabled;
        aloha_context_capture(CONTEXTS.0[0].get());
        ptr::copy_nonoverlapping(CONTEXTS.0[0].get(), CONTEXTS.0[1].get(), 1);
        (*CONTEXTS.0[0].get()).saved_frame = 0;
        (*CONTEXTS.0[1].get()).saved_frame = 0;
    }

    XCR0_MASK.store(enabled, Ordering::Release);
    READY.store(true, Ordering::Release);
    serial::info(format_args!(
        "context: assembly trampoline ready, xstate bytes {}",
        configured.ebx
    ));
    true
}

pub fn is_ready() -> bool {
    READY.load(Ordering::Acquire)
}

pub fn xcr0_mask() -> u64 {
    XCR0_MASK.load(Ordering::Acquire)
}

/// Build the first persistent IST frame for a kernel task.
///
/// `stack_top` is the top of its mapped stack and `task_returned` is a fatal
/// fallback used if the task entry unexpectedly returns.
pub unsafe fn prepare_kernel_task(
    task: usize,
    stack_top: u64,
    entry: u64,
    task_returned: u64,
    code_selector: u64,
    stack_selector: u64,
) -> bool {
    let (Some(context), Some(frame)) = (CONTEXTS.0.get(task), FRAMES.0.get(task)) else {
        return false;
    };
    let Some(task_rsp) = stack_top.checked_sub(8) else { return false };
    *(task_rsp as *mut u64) = task_returned;

    let frame = &mut *frame.get();
    frame.0.fill(0);
    frame.0[15] = entry;
    frame.0[16] = code_selector;
    frame.0[17] = INTERRUPT_FLAGS;
    frame.0[18] = task_rsp;
    frame.0[19] = stack_selector;
    (*context.get()).saved_frame = frame as *mut InterruptFrame as u64;
    true
}

/// Save the interrupted task's complete IST frame and return the persistent
/// frame of the task selected for restore.
pub unsafe fn switch(current: usize, next: usize, current_ist_frame: u64) -> u64 {
    if current == next || !is_ready() {
        return current_ist_frame;
    }
    let (Some(current_context), Some(next_context), Some(current_frame)) = (
        CONTEXTS.0.get(current),
        CONTEXTS.0.get(next),
        FRAMES.0.get(current),
    ) else {
        return current_ist_frame;
    };
    if (*next_context.get()).saved_frame == 0 {
        return current_ist_frame;
    }
    aloha_context_switch(
        current_context.get(),
        next_context.get(),
        current_ist_frame,
        current_frame.get(),
    )
}

#[inline]
unsafe fn xsetbv(value: u64) {
    asm!(
        "xsetbv",
        in("ecx") 0u32,
        in("eax") value as u32,
        in("edx") (value >> 32) as u32,
        options(nostack)
    );
}

global_asm!(r#"
.global aloha_context_capture
.type aloha_context_capture,@function
aloha_context_capture:
    mov r8, rdi
    mov rax, cr3
    mov [r8 + 8], rax

    mov ecx, 0xc0000100
    rdmsr
    shl rdx, 32
    or rax, rdx
    mov [r8 + 16], rax

    mov ecx, 0xc0000101
    rdmsr
    shl rdx, 32
    or rax, rdx
    mov [r8 + 24], rax

    mov eax, [r8 + 32]
    mov edx, [r8 + 36]
    xsave64 [r8 + 64]
    ret
.size aloha_context_capture, .-aloha_context_capture

.global aloha_context_switch
.type aloha_context_switch,@function
aloha_context_switch:
    mov r9, rdi
    mov r10, rsi
    mov r11, rcx

    mov rdi, rcx
    mov rsi, rdx
    mov ecx, 20
    rep movsq
    mov [r9], r11

    mov rax, cr3
    mov [r9 + 8], rax

    mov ecx, 0xc0000100
    rdmsr
    shl rdx, 32
    or rax, rdx
    mov [r9 + 16], rax

    mov ecx, 0xc0000101
    rdmsr
    shl rdx, 32
    or rax, rdx
    mov [r9 + 24], rax

    mov eax, [r9 + 32]
    mov edx, [r9 + 36]
    xsave64 [r9 + 64]

    mov r8, cr3
    mov r11, [r10 + 8]
    cmp r8, r11
    je 1f
    mov cr3, r11
1:
    mov rax, [r10 + 16]
    mov rdx, rax
    shr rdx, 32
    mov ecx, 0xc0000100
    wrmsr

    mov rax, [r10 + 24]
    mov rdx, rax
    shr rdx, 32
    mov ecx, 0xc0000101
    wrmsr

    mov eax, [r10 + 32]
    mov edx, [r10 + 36]
    xrstor64 [r10 + 64]
    mov rax, [r10]
    ret
.size aloha_context_switch, .-aloha_context_switch
"#);
