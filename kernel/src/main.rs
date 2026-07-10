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

#[no_mangle]
pub extern "sysv64" fn _start(boot_info: *const BootInfo) -> ! {
    unsafe { core::arch::asm!("cli", options(nomem, nostack)) };

    let info = unsafe { &*boot_info };
    framebuffer::init(info.framebuffer);
    framebuffer::clear(0x0f, 0x17, 0x2a);
    framebuffer::set_color(0xf5, 0xa6, 0x23);
    framebuffer::write_line("ALOHAOS");
    framebuffer::set_color(0xd7, 0xe0, 0xee);
    framebuffer::write_line("");

    gdt::init();
    interrupts::init();
    framebuffer::write_line("GDT TSS IDT READY");

    unsafe { memory::init(info.memory_map) };
    framebuffer::write_line("UEFI MEMORY MAP READY");

    let test_frame = memory::allocate_frame().unwrap_or_else(|| fatal("NO USABLE PHYSICAL MEMORY"));
    let paging = match paging::init(info.memory_map) {
        Ok(stats) => stats,
        Err(paging::PagingError::OutOfFrames) => fatal("PAGE TABLE ALLOCATION FAILED"),
        Err(paging::PagingError::PhysicalAddressTooLarge) => fatal("PHYSICAL MEMORY EXCEEDS DIRECT MAP"),
    };
    if !paging::verify_direct_map(test_frame) {
        fatal("HIGHER HALF DIRECT MAP TEST FAILED");
    }
    framebuffer::write_line("4 LEVEL PAGING READY");
    framebuffer::write_line("HIGHER HALF DIRECT MAP READY");
    framebuffer::write_label_hex("PML4 PHYSICAL:   ", paging.pml4_physical);

    let initial_heap = heap::init().unwrap_or_else(|| fatal("KERNEL HEAP ALLOCATION FAILED"));
    framebuffer::write_label_hex("HEAP PHYSICAL:   ", initial_heap.physical_start);
    framebuffer::write_label_hex("HEAP VIRTUAL:    ", initial_heap.virtual_start);
    framebuffer::write_label_hex("HEAP SIZE:       ", initial_heap.size as u64);

    // Real liballoc smoke test: Box, Vec and String all use our global heap.
    let boxed = Box::new(0xa10a_05u64);
    let mut values = Vec::with_capacity(128);
    for value in 0..128u64 {
        values.push(value * 3);
    }
    let title = String::from("ALOHAOS ALLOC ONLINE");
    let checksum: u64 = values.iter().copied().sum::<u64>() ^ *boxed;
    core::hint::black_box((&boxed, &values, &title));

    let heap_stats = heap::stats();
    framebuffer::write_label_hex("HEAP USED:       ", heap_stats.used as u64);
    framebuffer::write_label_hex("HEAP FREE:       ", heap_stats.free as u64);
    framebuffer::write_label_hex("ALLOC CHECKSUM:  ", checksum);
    framebuffer::write_line("BOX VEC STRING READY");
    framebuffer::write_line("");
    framebuffer::write_line("MEMORY STAGE 3 READY");

    halt()
}

fn fatal(message: &str) -> ! {
    framebuffer::panic_header(message);
    halt()
}

pub fn halt() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { core::arch::asm!("cli", options(nomem, nostack)) };
    framebuffer::panic_header("RUST KERNEL PANIC");
    halt()
}
