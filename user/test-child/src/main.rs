#![no_std]
#![no_main]
use aloha_user::{entry,panic_handler,write};
fn main()->i32{let _=write(b"child: hello from a real user ELF\n");37}
entry!(main);
panic_handler!();
