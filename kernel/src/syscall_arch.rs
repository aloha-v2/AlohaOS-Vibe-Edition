//! x86_64 SYSCALL entry and active-process architecture bridge.

use core::arch::{asm, global_asm};
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::{gdt, process::Process, process_table, syscall, syscall_entry::{ReturnPath, UserReturnFrame}};

const IA32_EFER:u32=0xc000_0080;const IA32_STAR:u32=0xc000_0081;const IA32_LSTAR:u32=0xc000_0082;const IA32_FMASK:u32=0xc000_0084;const EFER_SCE:u64=1;const SYSCALL_MASK:u64=(1<<8)|(1<<9)|(1<<10)|(1<<18);
static INITIALIZED:AtomicBool=AtomicBool::new(false);
unsafe extern "C"{fn aloha_syscall_entry();fn aloha_return_process_runner(return_rsp:u64)->!;static mut ALOHA_SYSCALL_KERNEL_STACK:u64;static mut ALOHA_SYSCALL_KERNEL_RETURN_RSP:u64;static mut ALOHA_SYSCALL_PROCESS:u64;}

#[derive(Clone,Copy,Debug,Default)]#[repr(C)]pub struct SyscallFrame{pub number:u64,pub arguments:[u64;6],pub user_rip:u64,pub user_rflags:u64,pub user_rsp:u64,pub result:u64}

pub fn init()->bool{if !crate::syscall_entry::cpu_supports_syscall(){return false}if INITIALIZED.swap(true,Ordering::AcqRel){return true}let star=(0x10u64<<48)|((gdt::code_selector()as u64)<<32);unsafe{wrmsr(IA32_EFER,rdmsr(IA32_EFER)|EFER_SCE);wrmsr(IA32_STAR,star);wrmsr(IA32_LSTAR,aloha_syscall_entry as*const()as u64);wrmsr(IA32_FMASK,SYSCALL_MASK);}configuration_valid()}
pub fn install_process(process:&mut Process){unsafe{ptr::write_volatile(&raw mut ALOHA_SYSCALL_KERNEL_STACK,process.kernel_stack_top());ptr::write_volatile(&raw mut ALOHA_SYSCALL_PROCESS,process as*mut Process as u64);}}
pub fn uninstall_process(){unsafe{ptr::write_volatile(&raw mut ALOHA_SYSCALL_KERNEL_STACK,0);ptr::write_volatile(&raw mut ALOHA_SYSCALL_KERNEL_RETURN_RSP,0);ptr::write_volatile(&raw mut ALOHA_SYSCALL_PROCESS,0);}}
pub fn active_kernel_stack()->u64{unsafe{ptr::read_volatile(&raw const ALOHA_SYSCALL_KERNEL_STACK)}}
pub fn dispatch_frame(process:&mut Process,frame:&mut SyscallFrame)->bool{let result=syscall::dispatch(process,frame.number,frame.arguments);frame.result=result.value;result.terminated}

#[no_mangle]pub extern "C" fn rust_syscall_dispatch(frame:*mut SyscallFrame)->u8{let p=active_process_ptr();if frame.is_null()||p.is_null(){return 1}let process=unsafe{&mut*p};let frame=unsafe{&mut*frame};if dispatch_frame(process,frame){return 1}let candidate=UserReturnFrame{rip:frame.user_rip,rsp:frame.user_rsp,rflags:frame.user_rflags};match candidate.return_path(){ReturnPath::Sysret=>{frame.user_rflags=candidate.sanitized().rflags;0}ReturnPath::Iret|ReturnPath::Reject=>{process.fault();let _=process_table::fault(process.pid,-1);1}}}

/// Terminate only the active Ring 3 process, then resume its kernel runner.
#[no_mangle]pub extern "C" fn rust_user_fault(vector:u64,error:u64,rip:u64,address:u64)->!{let pointer=active_process_ptr();if pointer.is_null(){crate::serial::emergency(format_args!("user fault without active process"));crate::halt()}let process=unsafe{&mut*pointer};let code=-((vector as i32).max(1));process.fault();let _=process_table::fault(process.pid,code);process_table::orphan_children(process.pid);crate::serial::info(format_args!("user[{}] fault vector={} error={:#x} rip={:#x} address={:#x}",process.pid,vector,error,rip,address));let return_rsp=unsafe{ptr::read_volatile(&raw const ALOHA_SYSCALL_KERNEL_RETURN_RSP)};if return_rsp==0{crate::halt()}unsafe{aloha_return_process_runner(return_rsp)}}

fn active_process_ptr()->*mut Process{unsafe{ptr::read_volatile(&raw const ALOHA_SYSCALL_PROCESS)as*mut Process}}
pub fn configuration_valid()->bool{let expected=(0x10u64<<48)|((gdt::code_selector()as u64)<<32);unsafe{rdmsr(IA32_EFER)&EFER_SCE!=0&&rdmsr(IA32_STAR)==expected&&rdmsr(IA32_LSTAR)==aloha_syscall_entry as*const()as u64&&rdmsr(IA32_FMASK)==SYSCALL_MASK}}
#[inline]unsafe fn rdmsr(msr:u32)->u64{let l:u32;let h:u32;asm!("rdmsr",in("ecx")msr,out("eax")l,out("edx")h,options(nostack));(h as u64)<<32|l as u64}
#[inline]unsafe fn wrmsr(msr:u32,v:u64){asm!("wrmsr",in("ecx")msr,in("eax")v as u32,in("edx")(v>>32)as u32,options(nostack));}

global_asm!(r#"
.section .bss
.align 8
.global ALOHA_SYSCALL_KERNEL_STACK
ALOHA_SYSCALL_KERNEL_STACK:.zero 8
.global ALOHA_SYSCALL_KERNEL_RETURN_RSP
ALOHA_SYSCALL_KERNEL_RETURN_RSP:.zero 8
.global ALOHA_SYSCALL_PROCESS
ALOHA_SYSCALL_PROCESS:.zero 8
.global ALOHA_SYSCALL_USER_RSP
ALOHA_SYSCALL_USER_RSP:.zero 8
.section .text
.global aloha_syscall_entry
.type aloha_syscall_entry,@function
aloha_syscall_entry:
 mov [rip+ALOHA_SYSCALL_USER_RSP],rsp
 mov rsp,[rip+ALOHA_SYSCALL_KERNEL_STACK]
 test rsp,rsp
 jz .Lhalt
 sub rsp,96
 mov [rsp+0],rax
 mov [rsp+8],rdi
 mov [rsp+16],rsi
 mov [rsp+24],rdx
 mov [rsp+32],r10
 mov [rsp+40],r8
 mov [rsp+48],r9
 mov [rsp+56],rcx
 mov [rsp+64],r11
 mov rax,[rip+ALOHA_SYSCALL_USER_RSP]
 mov [rsp+72],rax
 mov qword ptr [rsp+80],0
 mov rdi,rsp
 call rust_syscall_dispatch
 test al,al
 jnz .Lterminated
 mov rax,[rsp+80]
 mov rcx,[rsp+56]
 mov r11,[rsp+64]
 mov rdx,[rsp+72]
 mov rsp,rdx
 sysretq
.Lterminated:
 mov rdi,[rip+ALOHA_SYSCALL_KERNEL_RETURN_RSP]
 jmp aloha_return_process_runner
.global aloha_return_process_runner
.type aloha_return_process_runner,@function
aloha_return_process_runner:
 mov ax,0x10
 mov ds,ax
 mov es,ax
 mov ss,ax
 mov rsp,rdi
 ret
.Lhalt:
 cli
1: hlt
 jmp 1b
.size aloha_syscall_entry,.-aloha_syscall_entry
"#);
