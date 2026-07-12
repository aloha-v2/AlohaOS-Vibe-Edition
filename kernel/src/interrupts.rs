//! IDT, CPU exception stubs, hardware IRQs and the first Ring 3 trap.

use core::arch::{asm, global_asm};
use core::cell::UnsafeCell;
use core::mem::size_of;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::{framebuffer, gdt, halt, keyboard, pic, serial, timer};

pub const USER_TRAP_VECTOR: usize = 0x80;

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry { offset_low:u16, selector:u16, ist:u8, attributes:u8, offset_middle:u16, offset_high:u32, reserved:u32 }
impl IdtEntry {
    const MISSING:Self=Self{offset_low:0,selector:0,ist:0,attributes:0,offset_middle:0,offset_high:0,reserved:0};
    fn gate(address:u64,ist:u8,attributes:u8)->Self{Self{offset_low:address as u16,selector:gdt::code_selector(),ist:ist&7,attributes,offset_middle:(address>>16)as u16,offset_high:(address>>32)as u32,reserved:0}}
    fn kernel(address:u64,ist:u8)->Self{Self::gate(address,ist,0x8e)}
    fn user_trap(address:u64)->Self{Self::gate(address,0,0xee)}
}
#[repr(C,packed)]struct IdtPointer{limit:u16,base:u64}
struct IdtStorage(UnsafeCell<[IdtEntry;256]>);unsafe impl Sync for IdtStorage{}
static IDT:IdtStorage=IdtStorage(UnsafeCell::new([IdtEntry::MISSING;256]));static INITIALIZED:AtomicBool=AtomicBool::new(false);
unsafe extern "C"{fn isr_divide_error();fn isr_breakpoint();fn isr_invalid_opcode();fn isr_double_fault();fn isr_general_protection();fn isr_page_fault();fn irq_timer();fn irq_keyboard();fn user_trap_80();}

pub fn init(){if INITIALIZED.swap(true,Ordering::AcqRel){return}unsafe{let idt=&mut*IDT.0.get();idt[0]=IdtEntry::kernel(isr_divide_error as*const()as u64,0);idt[3]=IdtEntry::kernel(isr_breakpoint as*const()as u64,0);idt[6]=IdtEntry::kernel(isr_invalid_opcode as*const()as u64,0);idt[8]=IdtEntry::kernel(isr_double_fault as*const()as u64,gdt::double_fault_ist());idt[13]=IdtEntry::kernel(isr_general_protection as*const()as u64,0);idt[14]=IdtEntry::kernel(isr_page_fault as*const()as u64,0);idt[pic::TIMER_VECTOR as usize]=IdtEntry::kernel(irq_timer as*const()as u64,gdt::scheduler_ist());idt[pic::KEYBOARD_VECTOR as usize]=IdtEntry::kernel(irq_keyboard as*const()as u64,0);idt[USER_TRAP_VECTOR]=IdtEntry::user_trap(user_trap_80 as*const()as u64);let pointer=IdtPointer{limit:(size_of::<[IdtEntry;256]>()-1)as u16,base:IDT.0.get()as u64};asm!("lidt [{}]",in(reg)&pointer,options(readonly,nostack));}}
pub fn enable(){unsafe{asm!("sti",options(nomem,nostack))}}
#[no_mangle]pub extern "C" fn rust_timer_interrupt(stack:u64)->u64{timer::interrupt(stack)}
#[no_mangle]pub extern "C" fn rust_keyboard_interrupt(){keyboard::interrupt();}
#[no_mangle]pub extern "C" fn rust_exception_handler(vector:u64,error_code:u64,instruction_pointer:u64,fault_address:u64)->!{unsafe{asm!("cli",options(nomem,nostack))};serial::emergency(format_args!("CPU EXCEPTION vector={} error={:#x} rip={:#x} cr2={:#x}",vector,error_code,instruction_pointer,fault_address));let name=match vector{0=>"DIVIDE BY ZERO",3=>"BREAKPOINT",6=>"INVALID OPCODE",8=>"DOUBLE FAULT",13=>"GENERAL PROTECTION FAULT",14=>"PAGE FAULT",_=>"UNKNOWN CPU EXCEPTION"};framebuffer::panic_header(name);framebuffer::write_label_hex("VECTOR: ",vector);framebuffer::write_label_hex("ERROR: ",error_code);framebuffer::write_label_hex("RIP: ",instruction_pointer);if vector==14{framebuffer::write_label_hex("CR2: ",fault_address)}halt()}

global_asm!(r#"
.global isr_divide_error
isr_divide_error: cli; mov rdx,[rsp]; xor esi,esi; xor ecx,ecx; mov edi,0; jmp exception_trampoline
.global isr_breakpoint
isr_breakpoint: cli; mov rdx,[rsp]; xor esi,esi; xor ecx,ecx; mov edi,3; jmp exception_trampoline
.global isr_invalid_opcode
isr_invalid_opcode: cli; mov rdx,[rsp]; xor esi,esi; xor ecx,ecx; mov edi,6; jmp exception_trampoline
.global isr_double_fault
isr_double_fault: cli; mov rsi,[rsp]; mov rdx,[rsp+8]; xor ecx,ecx; mov edi,8; jmp exception_trampoline
.global isr_general_protection
isr_general_protection: cli; mov rsi,[rsp]; mov rdx,[rsp+8]; xor ecx,ecx; mov edi,13; jmp exception_trampoline
.global isr_page_fault
isr_page_fault: cli; mov rsi,[rsp]; mov rdx,[rsp+8]; mov rcx,cr2; mov edi,14; jmp exception_trampoline
exception_trampoline: and rsp,-16; call rust_exception_handler; ud2
.global irq_timer
irq_timer:
 push rax;push rcx;push rdx;push rsi;push rdi;push r8;push r9;push r10;push r11;push rbx;push rbp;push r12;push r13;push r14;push r15
 mov rdi,rsp;mov rbx,rsp;and rsp,-16;call rust_timer_interrupt;mov rsp,rax
 pop r15;pop r14;pop r13;pop r12;pop rbp;pop rbx;pop r11;pop r10;pop r9;pop r8;pop rdi;pop rsi;pop rdx;pop rcx;pop rax;iretq
.global irq_keyboard
irq_keyboard:
 push rax;push rcx;push rdx;push rsi;push rdi;push r8;push r9;push r10;push r11;push rbx;push rbp;push r12;push r13;push r14;push r15
 mov rbx,rsp;and rsp,-16;call rust_keyboard_interrupt;mov rsp,rbx
 pop r15;pop r14;pop r13;pop r12;pop rbp;pop rbx;pop r11;pop r10;pop r9;pop r8;pop rdi;pop rsi;pop rdx;pop rcx;pop rax;iretq
.global user_trap_80
user_trap_80:
 cli
 mov rdi, rax
 and rsp, -16
 call rust_user_trap
 ud2
"#);
