//! x86_64 SYSCALL/SYSRET entry wired to the safe Rust dispatcher.

use core::arch::{asm, global_asm};
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::{gdt, process::Process, syscall, syscall_entry::{ReturnPath, UserReturnFrame}};

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

unsafe extern "C" {
    fn aloha_syscall_entry();
    static mut ALOHA_SYSCALL_KERNEL_STACK: u64;
    static mut ALOHA_SYSCALL_KERNEL_RETURN_RSP: u64;
    static mut ALOHA_SYSCALL_PROCESS: u64;
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
    let star = ((0x10u64) << 48) | ((gdt::code_selector() as u64) << 32);
    unsafe {
        wrmsr(IA32_EFER, rdmsr(IA32_EFER) | EFER_SCE);
        wrmsr(IA32_STAR, star);
        wrmsr(IA32_LSTAR, aloha_syscall_entry as *const () as u64);
        wrmsr(IA32_FMASK, SYSCALL_MASK);
    }
    configuration_valid()
}

/// Install the single-core active process immediately before entering user mode.
pub fn install_process(process: &mut Process, kernel_return_rsp: u64) {
    unsafe {
        ptr::write_volatile(&raw mut ALOHA_SYSCALL_KERNEL_STACK, process.kernel_stack_top());
        ptr::write_volatile(&raw mut ALOHA_SYSCALL_KERNEL_RETURN_RSP, kernel_return_rsp);
        ptr::write_volatile(&raw mut ALOHA_SYSCALL_PROCESS, process as *mut Process as u64);
    }
}

pub fn active_kernel_stack() -> u64 {
    unsafe { ptr::read_volatile(&raw const ALOHA_SYSCALL_KERNEL_STACK) }
}

pub fn dispatch_frame(process: &mut Process, frame: &mut SyscallFrame) -> bool {
    let result = syscall::dispatch(process, frame.number, frame.arguments);
    frame.result = result.value;
    result.terminated
}

#[no_mangle]
pub extern "C" fn rust_syscall_dispatch(frame: *mut SyscallFrame) -> u8 {
    let process_ptr = unsafe { ptr::read_volatile(&raw const ALOHA_SYSCALL_PROCESS) };
    if frame.is_null() || process_ptr == 0 {
        return 1;
    }
    let process = unsafe { &mut *(process_ptr as *mut Process) };
    let frame = unsafe { &mut *frame };
    let terminated = dispatch_frame(process, frame);
    if terminated {
        return 1;
    }

    let candidate = UserReturnFrame {
        rip: frame.user_rip,
        rsp: frame.user_rsp,
        rflags: frame.user_rflags,
    };
    match candidate.return_path() {
        ReturnPath::Sysret => {
            frame.user_rflags = candidate.sanitized().rflags;
            0
        }
        // The assembly fast path only implements SYSRET today. Fail closed for
        // frames that require IRET until the audited fallback lands.
        ReturnPath::Iret | ReturnPath::Reject => {
            process.fault();
            1
        }
    }
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
    asm!("wrmsr", in("ecx") msr, in("eax") value as u32, in("edx") (value >> 32) as u32, options(nostack));
}

global_asm!(r#"
.section .bss
.align 8
.global ALOHA_SYSCALL_KERNEL_STACK
ALOHA_SYSCALL_KERNEL_STACK: .zero 8
.global ALOHA_SYSCALL_KERNEL_RETURN_RSP
ALOHA_SYSCALL_KERNEL_RETURN_RSP: .zero 8
.global ALOHA_SYSCALL_PROCESS
ALOHA_SYSCALL_PROCESS: .zero 8
.global ALOHA_SYSCALL_USER_RSP
ALOHA_SYSCALL_USER_RSP: .zero 8

.section .text
.global aloha_syscall_entry
.type aloha_syscall_entry,@function
aloha_syscall_entry:
    mov [rip + ALOHA_SYSCALL_USER_RSP], rsp
    mov rsp, [rip + ALOHA_SYSCALL_KERNEL_STACK]
    test rsp, rsp
    jz .Lsyscall_halt

    sub rsp, 96
    mov [rsp + 0], rax
    mov [rsp + 8], rdi
    mov [rsp + 16], rsi
    mov [rsp + 24], rdx
    mov [rsp + 32], r10
    mov [rsp + 40], r8
    mov [rsp + 48], r9
    mov [rsp + 56], rcx
    mov [rsp + 64], r11
    mov rax, [rip + ALOHA_SYSCALL_USER_RSP]
    mov [rsp + 72], rax
    mov qword ptr [rsp + 80], 0

    mov rdi, rsp
    call rust_syscall_dispatch
    test al, al
    jnz .Lsyscall_terminated

    mov rax, [rsp + 80]
    mov rcx, [rsp + 56]
    mov r11, [rsp + 64]
    mov rdx, [rsp + 72]
    mov rsp, rdx
    sysretq

.Lsyscall_terminated:
    mov rsp, [rip + ALOHA_SYSCALL_KERNEL_RETURN_RSP]
    ret
.Lsyscall_halt:
    cli
1:
    hlt
    jmp 1b
.size aloha_syscall_entry, .-aloha_syscall_entry
"#);
