//! First controlled Ring 3 round-trip for AlohaOS.
//!
//! A tiny user image runs under its process CR3 and returns only through the
//! DPL3 software trap at vector 0x80. Bootstrap stack bookkeeping stays wholly
//! in assembly; Rust only records the marker and updates process state.

use core::arch::global_asm;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::{gdt, process::Process};

const NO_MARKER: u64 = u64::MAX;
static LAST_MARKER: AtomicU64 = AtomicU64::new(NO_MARKER);

unsafe extern "C" {
    fn aloha_enter_user(rip: u64, rsp: u64, code_selector: u64, data_selector: u64);
}

pub fn run(process: &mut Process) -> u64 {
    LAST_MARKER.store(NO_MARKER, Ordering::Release);
    process.mark_running();

    // Keep the immutable address-space borrow in its own scope. The guard must
    // restore kernel CR3 and drop before Process state is mutated below.
    {
        let address_space_guard = process.address_space.activate();
        unsafe {
            aloha_enter_user(
                process.entry,
                process.user_stack_top,
                gdt::user_code_selector() as u64,
                gdt::user_data_selector() as u64,
            );
        }
        drop(address_space_guard);
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
pub extern "C" fn rust_user_trap(marker: u64) {
    LAST_MARKER.store(marker, Ordering::Release);
}

global_asm!(r#"
.section .bss
.align 8
.global ALOHA_USER_RETURN_RSP
ALOHA_USER_RETURN_RSP:
    .zero 8

.section .text
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
