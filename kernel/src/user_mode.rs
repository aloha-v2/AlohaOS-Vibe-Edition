//! First controlled Ring 3 round-trip for AlohaOS.
//!
//! A tiny user image runs under its process CR3 and returns only through the
//! DPL3 software trap at vector 0x80. The trap switches to TSS RSP0, records the
//! marker and resumes the suspended kernel call frame. This is a bootstrap
//! path; the real syscall ABI will replace the single global return slot.

use core::arch::{asm, global_asm};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::{gdt, process::Process};

const NO_MARKER: u64 = u64::MAX;

#[repr(transparent)]
pub struct ReturnSlot(UnsafeCell<u64>);
unsafe impl Sync for ReturnSlot {}

#[no_mangle]
pub static ALOHA_USER_RETURN_RSP: ReturnSlot = ReturnSlot(UnsafeCell::new(0));
static LAST_MARKER: AtomicU64 = AtomicU64::new(NO_MARKER);

unsafe extern "C" {
    fn aloha_enter_user(rip: u64, rsp: u64, code_selector: u64, data_selector: u64);
}

pub fn run(process: &mut Process) -> u64 {
    LAST_MARKER.store(NO_MARKER, Ordering::Release);
    process.mark_running();
    let _address_space = process.address_space.activate();
    unsafe {
        aloha_enter_user(
            process.entry,
            process.user_stack_top,
            gdt::user_code_selector() as u64,
            gdt::user_data_selector() as u64,
        );
    }
    let marker = LAST_MARKER.load(Ordering::Acquire);
    if marker == NO_MARKER {
        process.fault();
    } else {
        process.exit(0);
    }
    marker
}

#[no_mangle]
pub extern "C" fn rust_user_trap(marker: u64) -> ! {
    LAST_MARKER.store(marker, Ordering::Release);
    let return_rsp = unsafe { *ALOHA_USER_RETURN_RSP.0.get() };
    let kernel_data = gdt::kernel_data_selector() as u64;
    unsafe {
        asm!(
            "mov ax, {kernel_data:x}",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov rsp, {return_rsp}",
            "ret",
            kernel_data = in(reg) kernel_data,
            return_rsp = in(reg) return_rsp,
            options(noreturn)
        );
    }
}

global_asm!(r#"
.global aloha_enter_user
.type aloha_enter_user,@function
aloha_enter_user:
    mov [rip + ALOHA_USER_RETURN_RSP], rsp
    mov ax, cx
    mov ds, ax
    mov es, ax
    push rcx
    push rsi
    push 0x202
    push rdx
    push rdi
    iretq
.size aloha_enter_user, .-aloha_enter_user
"#);
