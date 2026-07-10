//! AlohaOS kernel entry point.
#![no_std]
#![no_main]

extern crate alloc;

use alloc::{boxed::Box, string::String, vec::Vec};
use core::panic::PanicInfo;
use common::BootInfo;

mod font;
mod framebuffer;
mod gdt;
mod interrupts;
mod memory;
mod paging;
mod heap;
mod pic;
mod keyboard;
mod timer;
mod scheduler;
mod virtio_blk;
mod fat32;
mod shell;

#[no_mangle]
pub extern "sysv64" fn _start(boot_info: *const BootInfo) -> ! {
    unsafe { core::arch::asm!("cli", options(nomem, nostack)) };
    let info = unsafe { &*boot_info };
    framebuffer::init(info.framebuffer);
    framebuffer::clear_console();
    gdt::init();
    interrupts::init();
    unsafe { memory::init(info.memory_map) };

    let test_frame = memory::allocate_frame()
        .unwrap_or_else(|| fatal("NO USABLE PHYSICAL MEMORY"));
    let paging = match paging::init(info.memory_map) {
        Ok(value) => value,
        Err(paging::PagingError::OutOfFrames) => fatal("PAGE TABLE ALLOCATION FAILED"),
        Err(paging::PagingError::PhysicalAddressTooLarge) => fatal("PHYSICAL MEMORY EXCEEDS DIRECT MAP"),
    };
    if !paging::verify_direct_map(test_frame) {
        fatal("HIGHER HALF DIRECT MAP TEST FAILED");
    }

    heap::init().unwrap_or_else(|| fatal("KERNEL HEAP ALLOCATION FAILED"));
    let boxed = Box::new(1u64);
    let values = Vec::from([1u64, 2, 3]);
    let title = String::from("ALLOC");
    core::hint::black_box((&boxed, &values, &title, paging.pml4_physical));
    drop(title);
    drop(values);
    drop(boxed);

    let block_ready = virtio_blk::init();
    let fat_ready = block_ready && fat32::init();
    core::hint::black_box((block_ready, fat_ready));

    scheduler::init();
    unsafe {
        pic::init_timer_and_keyboard();
        timer::init();
    }
    interrupts::enable();
    shell::run()
}

fn fatal(message: &str) -> ! {
    framebuffer::panic_header(message);
    halt()
}

pub fn halt() -> ! {
    loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)) }; }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { core::arch::asm!("cli", options(nomem, nostack)) };
    framebuffer::panic_header("RUST KERNEL PANIC");
    halt()
}
