//! IDT and assembly ISR entry stubs for fatal CPU exceptions.

use core::arch::{asm, global_asm};
use core::mem::size_of;
use core::ptr::addr_of;

use crate::{framebuffer, gdt, halt};

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    attributes: u8,
    offset_middle: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const MISSING: Self = Self {
        offset_low: 0,
        selector: 0,
        ist: 0,
        attributes: 0,
        offset_middle: 0,
        offset_high: 0,
        reserved: 0,
    };

    fn handler(address: u64, ist: u8) -> Self {
        Self {
            offset_low: address as u16,
            selector: gdt::code_selector(),
            ist: ist & 0x07,
            attributes: 0x8e, // present, ring 0, interrupt gate
            offset_middle: (address >> 16) as u16,
            offset_high: (address >> 32) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::MISSING; 256];

unsafe extern "C" {
    fn isr_divide_error();
    fn isr_breakpoint();
    fn isr_invalid_opcode();
    fn isr_double_fault();
    fn isr_general_protection();
    fn isr_page_fault();
}

pub fn init() {
    unsafe {
        IDT[0] = IdtEntry::handler(isr_divide_error as usize as u64, 0);
        IDT[3] = IdtEntry::handler(isr_breakpoint as usize as u64, 0);
        IDT[6] = IdtEntry::handler(isr_invalid_opcode as usize as u64, 0);
        IDT[8] = IdtEntry::handler(
            isr_double_fault as usize as u64,
            gdt::double_fault_ist(),
        );
        IDT[13] = IdtEntry::handler(isr_general_protection as usize as u64, 0);
        IDT[14] = IdtEntry::handler(isr_page_fault as usize as u64, 0);

        let pointer = IdtPointer {
            limit: (size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: addr_of!(IDT) as u64,
        };
        asm!("lidt [{}]", in(reg) &pointer, options(readonly, nostack));
    }
}

/// Called by the assembly stubs with a normalized exception record.
#[no_mangle]
pub extern "C" fn rust_exception_handler(
    vector: u64,
    error_code: u64,
    instruction_pointer: u64,
    fault_address: u64,
) -> ! {
    unsafe { asm!("cli", options(nomem, nostack)) };

    let name = match vector {
        0 => "DIVIDE BY ZERO",
        3 => "BREAKPOINT",
        6 => "INVALID OPCODE",
        8 => "DOUBLE FAULT",
        13 => "GENERAL PROTECTION FAULT",
        14 => "PAGE FAULT",
        _ => "UNKNOWN CPU EXCEPTION",
    };

    framebuffer::panic_header(name);
    framebuffer::write_label_hex("VECTOR: ", vector);
    framebuffer::write_label_hex("ERROR:  ", error_code);
    framebuffer::write_label_hex("RIP:    ", instruction_pointer);
    if vector == 14 {
        framebuffer::write_label_hex("CR2:    ", fault_address);
    }
    halt()
}

// Exceptions 8, 13 and 14 push an error code. The other installed vectors do
// not. Every stub extracts RIP from the corresponding hardware stack frame and
// enters one non-returning Rust handler using the SysV AMD64 ABI.
global_asm!(r#"
.intel_syntax noprefix

.global isr_divide_error
isr_divide_error:
    cli
    mov rdx, [rsp]
    xor esi, esi
    xor ecx, ecx
    mov edi, 0
    jmp exception_trampoline

.global isr_breakpoint
isr_breakpoint:
    cli
    mov rdx, [rsp]
    xor esi, esi
    xor ecx, ecx
    mov edi, 3
    jmp exception_trampoline

.global isr_invalid_opcode
isr_invalid_opcode:
    cli
    mov rdx, [rsp]
    xor esi, esi
    xor ecx, ecx
    mov edi, 6
    jmp exception_trampoline

.global isr_double_fault
isr_double_fault:
    cli
    mov rsi, [rsp]
    mov rdx, [rsp + 8]
    xor ecx, ecx
    mov edi, 8
    jmp exception_trampoline

.global isr_general_protection
isr_general_protection:
    cli
    mov rsi, [rsp]
    mov rdx, [rsp + 8]
    xor ecx, ecx
    mov edi, 13
    jmp exception_trampoline

.global isr_page_fault
isr_page_fault:
    cli
    mov rsi, [rsp]
    mov rdx, [rsp + 8]
    mov rcx, cr2
    mov edi, 14
    jmp exception_trampoline

exception_trampoline:
    and rsp, -16
    call rust_exception_handler
    ud2
"#);
