//! Bounded x86_64 frame-pointer backtraces for fatal kernel paths.
//!
//! Release builds retain RBP chains. We only follow aligned frames inside a
//! small monotonically increasing stack window and only print return addresses
//! that fall in the linked kernel text range. This is intentionally allocation
//! free and suitable after interrupts have been disabled.

use core::arch::asm;

use crate::{framebuffer, serial};

const MAX_FRAMES: usize = 24;
const MAX_STACK_WINDOW: u64 = 1024 * 1024;

unsafe extern "C" {
    static __text_start: u8;
    static __text_end: u8;
}

#[repr(C)]
struct StackFrame {
    previous: u64,
    return_address: u64,
}

pub fn dump_current() {
    let frame_pointer: u64;
    let stack_pointer: u64;
    unsafe {
        asm!("mov {}, rbp", out(reg) frame_pointer, options(nomem, nostack, preserves_flags));
        asm!("mov {}, rsp", out(reg) stack_pointer, options(nomem, nostack, preserves_flags));
    }
    dump(frame_pointer, stack_pointer, None);
}

pub fn dump_from(frame_pointer: u64, instruction_pointer: u64) {
    let stack_pointer: u64;
    unsafe {
        asm!("mov {}, rsp", out(reg) stack_pointer, options(nomem, nostack, preserves_flags));
    }
    dump(frame_pointer, stack_pointer, Some(instruction_pointer));
}

fn dump(mut frame_pointer: u64, stack_pointer: u64, first: Option<u64>) {
    let text_start = core::ptr::addr_of!(__text_start) as u64;
    let text_end = core::ptr::addr_of!(__text_end) as u64;
    let stack_end = stack_pointer.saturating_add(MAX_STACK_WINDOW);

    serial::emergency(format_args!("backtrace: text={:#x}..{:#x}", text_start, text_end));
    framebuffer::write_line("BACKTRACE:");

    let mut index = 0usize;
    if let Some(address) = first.filter(|address| in_text(*address, text_start, text_end)) {
        emit(index, address);
        index += 1;
    }

    while index < MAX_FRAMES {
        if frame_pointer & 0x7 != 0
            || frame_pointer < stack_pointer
            || frame_pointer > stack_end.saturating_sub(core::mem::size_of::<StackFrame>() as u64)
        {
            break;
        }

        let frame = unsafe { &*(frame_pointer as *const StackFrame) };
        let return_address = frame.return_address;
        if !in_text(return_address, text_start, text_end) {
            break;
        }
        emit(index, return_address);
        index += 1;

        let previous = frame.previous;
        if previous <= frame_pointer || previous > stack_end {
            break;
        }
        frame_pointer = previous;
    }

    if index == 0 {
        serial::emergency(format_args!("backtrace: no valid frames"));
        framebuffer::write_line("NO VALID FRAMES");
    }
}

fn emit(index: usize, address: u64) {
    serial::emergency(format_args!("backtrace: #{:02} {:#018x}", index, address));
    framebuffer::write_label_hex("FRAME: ", address);
}

const fn in_text(address: u64, start: u64, end: u64) -> bool {
    address >= start && address < end
}
