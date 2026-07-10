//! AlohaOS kernel entry point.
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use common::BootInfo;

mod font;
mod framebuffer;
mod gdt;
mod interrupts;
mod memory;

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
    let stats = memory::stats();
    framebuffer::write_line("UEFI MEMORY MAP READY");
    framebuffer::write_label_hex("MEMORY REGIONS: ", info.memory_map.region_count as u64);
    framebuffer::write_label_hex("USABLE FRAMES:  ", stats.total_usable);
    framebuffer::write_label_hex("FREE FRAMES:    ", stats.free);

    // Prove that allocation works without touching the returned frame yet.
    if let Some(frame) = memory::allocate_frame() {
        framebuffer::write_label_hex("FIRST FRAME:    ", frame);
        framebuffer::write_line("FRAME ALLOCATOR READY");
    } else {
        framebuffer::panic_header("NO USABLE PHYSICAL MEMORY");
    }

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
