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
mod paging;

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
    let before = memory::stats();
    framebuffer::write_line("UEFI MEMORY MAP READY");
    framebuffer::write_label_hex("USABLE FRAMES:  ", before.total_usable);

    // Reserve one ordinary frame before page-table construction and use it to
    // prove that the physical and higher-half virtual addresses are aliases.
    let test_frame = match memory::allocate_frame() {
        Some(frame) => frame,
        None => fatal("NO USABLE PHYSICAL MEMORY"),
    };

    let paging = match paging::init(info.memory_map) {
        Ok(stats) => stats,
        Err(paging::PagingError::OutOfFrames) => fatal("PAGE TABLE ALLOCATION FAILED"),
        Err(paging::PagingError::PhysicalAddressTooLarge) => {
            fatal("PHYSICAL MEMORY EXCEEDS DIRECT MAP")
        }
    };

    if !paging::verify_direct_map(test_frame) {
        fatal("HIGHER HALF DIRECT MAP TEST FAILED");
    }

    framebuffer::write_line("4 LEVEL PAGING READY");
    framebuffer::write_line("HIGHER HALF DIRECT MAP READY");
    framebuffer::write_label_hex("PML4 PHYSICAL:   ", paging.pml4_physical);
    framebuffer::write_label_hex("MAPPED 2M PAGES: ", paging.mapped_2m_pages);
    framebuffer::write_label_hex("PAGE TABLES:     ", paging.table_frames);
    framebuffer::write_label_hex("MAPPED BYTES:    ", paging.mapped_physical_bytes);

    let after = memory::stats();
    framebuffer::write_label_hex("FREE FRAMES:     ", after.free);
    framebuffer::write_line("");
    framebuffer::write_line("MEMORY STAGE 2 READY");

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
