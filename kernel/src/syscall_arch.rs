//! x86_64 SYSCALL MSR configuration and register context.
//!
//! This module installs the architectural contract without yet executing a
//! production `syscall` round-trip. The next step wires the assembly trampoline
//! to the tested Rust dispatcher and selects SYSRET or IRET from validated state.

use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::{gdt, process::Process, syscall};

const IA32_EFER: u32 = 0xc000_0080;
const IA32_STAR: u32 = 0xc000_0081;
const IA32_LSTAR: u32 = 0xc000_0082;
const IA32_FMASK: u32 = 0xc000_0084;
const EFER_SCE: u64 = 1;
const RFLAGS_TF: u64 = 1 << 8;
const RFLAGS_IF: u64 = 1 << 9;
const RFLAGS_DF: u64 = 1 << 10;
const RFLAGS_AC: u64 = 1 << 18;
const SYSCALL_MASK: u64 = RFLAGS_TF | RFLAGS_IF | RFLAGS_DF | RFLAGS_AC;

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static ACTIVE_KERNEL_STACK: AtomicU64 = AtomicU64::new(0);

unsafe extern "C" {
    fn aloha_syscall_entry();
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct SyscallFrame {
    pub number: u64,
    pub arguments: [u64; 6],
    pub user_rip: u64,
    pub user_rflags: u64,
    pub user_rsp: u64,
    pub result: u64,
}

pub fn init() -> bool {
    if !crate::syscall_entry::cpu_supports_syscall() {
        return false;
    }
    if INITIALIZED.swap(true, Ordering::AcqRel) {
        return true;
    }

    // SYSRET derives SS = base + 8 | 3 and CS = base + 16 | 3.
    // With base 0x10 this matches user data 0x1b and user code 0x23.
    let star = ((0x10u64) << 48) | ((gdt::code_selector() as u64) << 32);
    unsafe {
        wrmsr(IA32_EFER, rdmsr(IA32_EFER) | EFER_SCE);
        wrmsr(IA32_STAR, star);
        wrmsr(IA32_LSTAR, aloha_syscall_entry as *const () as u64);
        wrmsr(IA32_FMASK, SYSCALL_MASK);
    }
    configuration_valid()
}

pub fn install_process(process: &Process) {
    ACTIVE_KERNEL_STACK.store(process.kernel_stack_top(), Ordering::Release);
}

pub fn active_kernel_stack() -> u64 {
    ACTIVE_KERNEL_STACK.load(Ordering::Acquire)
}

pub fn dispatch_frame(process: &mut Process, frame: &mut SyscallFrame) -> bool {
    let result = syscall::dispatch(process, frame.number, frame.arguments);
    frame.result = result.value;
    result.terminated
}

pub fn configuration_valid() -> bool {
    let expected_star = ((0x10u64) << 48) | ((gdt::code_selector() as u64) << 32);
    unsafe {
        rdmsr(IA32_EFER) & EFER_SCE != 0
            && rdmsr(IA32_STAR) == expected_star
            && rdmsr(IA32_LSTAR) == aloha_syscall_entry as *const () as u64
            && rdmsr(IA32_FMASK) == SYSCALL_MASK
    }
}

#[inline]
unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nostack));
    (high as u64) << 32 | low as u64
}

#[inline]
unsafe fn wrmsr(msr: u32, value: u64) {
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") value as u32,
        in("edx") (value >> 32) as u32,
        options(nostack)
    );
}

// Deliberately fail-closed until the full save/switch/return trampoline lands.
// If invoked early, disable interrupts and halt rather than running Rust on the
// user stack or returning through an unchecked SYSRET state.
global_asm!(r#"
.global aloha_syscall_entry
.type aloha_syscall_entry,@function
aloha_syscall_entry:
    cli
1:
    hlt
    jmp 1b
.size aloha_syscall_entry, .-aloha_syscall_entry
"#);
