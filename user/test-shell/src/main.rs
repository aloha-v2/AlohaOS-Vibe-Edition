#![no_std]
#![no_main]
use aloha_user::{entry,panic_handler,spawn,wait,write};
fn main()->i32{let pid=match spawn(b"CHILD.ELF"){Ok(pid)=>pid,Err(_)=>return 10};let _=write(b"shell: spawned child\n");match wait(pid){Ok(status)=>status,Err(_)=>11}}
entry!(main);
panic_handler!();
