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
    unsafe { memory::init(info.memory_map) };

    let test_frame = memory::allocate_frame().unwrap_or_else(|| fatal("NO USABLE PHYSICAL MEMORY"));
    let paging = match paging::init(info.memory_map) {
        Ok(stats) => stats,
        Err(paging::PagingError::OutOfFrames) => fatal("PAGE TABLE ALLOCATION FAILED"),
        Err(paging::PagingError::PhysicalAddressTooLarge) => fatal("PHYSICAL MEMORY EXCEEDS DIRECT MAP"),
    };
    if !paging::verify_direct_map(test_frame) { fatal("HIGHER HALF DIRECT MAP TEST FAILED"); }

    let initial_heap = heap::init().unwrap_or_else(|| fatal("KERNEL HEAP ALLOCATION FAILED"));
    let boxed = Box::new(0xa10a_05u64);
    let mut values = Vec::with_capacity(128);
    for value in 0..128u64 { values.push(value * 3); }
    let title = String::from("ALOHAOS ALLOC ONLINE");
    core::hint::black_box((&boxed, &values, &title));

    framebuffer::write_line("GDT TSS IDT READY");
    framebuffer::write_line("MEMORY PAGING HEAP READY");
    framebuffer::write_label_hex("PML4 PHYSICAL: ", paging.pml4_physical);
    framebuffer::write_label_hex("HEAP PHYSICAL: ", initial_heap.physical_start);

    unsafe { pic::init_keyboard_only() };
    framebuffer::write_line("PIC 8259 READY");
    framebuffer::write_line("PS2 KEYBOARD IRQ1 READY");
    framebuffer::write_line("");
    framebuffer::set_color(0xf5, 0xa6, 0x23);
    framebuffer::write_text("TYPE HERE: ");
    framebuffer::set_color(0xff, 0xff, 0xff);
    interrupts::enable();

    keyboard_loop()
}

fn keyboard_loop() -> ! {
    loop {
        while let Some(scancode) = keyboard::pop_scancode() {
            if let Some(character) = keyboard::decode(scancode) {
                framebuffer::write_byte(character);
            }
        }
        // Atomic enable-and-sleep avoids missing an IRQ between checking the
        // queue and halting. An interrupt wakes the CPU and returns here.
        unsafe { core::arch::asm!("sti", "hlt", options(nomem, nostack)) };
    }
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
