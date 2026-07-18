//! Controlled Ring 3 execution and resumption for user processes.
use core::arch::global_asm;
use core::sync::atomic::{AtomicU64,Ordering};
use crate::{gdt,process::{Process,ProcessState},process_runtime,process_table,serial,syscall_arch,syscall_entry::ReturnPath};
const NO_MARKER:u64=u64::MAX;static LAST_MARKER:AtomicU64=AtomicU64::new(NO_MARKER);
unsafe extern "C"{fn aloha_enter_user(rip:u64,rsp:u64,code_selector:u64,data_selector:u64);}
pub fn run(process:&mut Process)->u64{LAST_MARKER.store(NO_MARKER,Ordering::Release);process.mark_running();serial::info(format_args!("user[{}] run enter state=Running entry={:#x} stack={:#x}",process.pid,process.entry,process.user_stack_top));syscall_arch::install_process(process);{let guard=process.address_space.activate();unsafe{aloha_enter_user(process.entry,process.user_stack_top,gdt::user_code_selector()as u64,gdt::user_data_selector()as u64);}drop(guard);}syscall_arch::uninstall_process();let marker=LAST_MARKER.load(Ordering::Acquire);serial::info(format_args!("user[{}] run return marker={:#x} state={:?} exit={}",process.pid,marker,process.state,process.exit_code));if marker==NO_MARKER&&process.state==ProcessState::Running{process.fault();}else if marker!=NO_MARKER{process.exit(0);}marker}
pub fn resume(process:&mut Process)->Result<Option<ReturnPath>,process_table::TableError>{let Some(frame)=process_runtime::resume_frame(process)?else{return Ok(None)};Ok(Some(syscall_arch::resume_process(process,&frame)))}
#[no_mangle]pub extern "C" fn rust_user_trap(marker:u64){LAST_MARKER.store(marker,Ordering::Release);}
global_asm!(r#"
.section .bss
.align 8
.global ALOHA_USER_RETURN_RSP
ALOHA_USER_RETURN_RSP:.zero 8
.section .text
.global aloha_enter_user
.type aloha_enter_user,@function
aloha_enter_user:
 mov [rip+ALOHA_USER_RETURN_RSP],rsp
 mov [rip+ALOHA_SYSCALL_KERNEL_RETURN_RSP],rsp
 mov ax,cx
 mov ds,ax
 mov es,ax
 push rcx
 push rsi
 push 0x202
 push rdx
 push rdi
 iretq
.size aloha_enter_user,.-aloha_enter_user
"#);
