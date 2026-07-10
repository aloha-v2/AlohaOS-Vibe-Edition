//! AlohaOS kernel entry point.
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use common::BootInfo;

mod font;
mod framebuffer;
mod gdt;
mod interrupts;

#[no_mangle]
pub extern "sysv64" fn _start(boot_info: *const BootInfo) -> ! {
    // No hardware IRQs are enabled until the interrupt controller is ready.
    unsafe { core::arch::asm!("cli", options(nomem, nostack)) };

    let info = unsafe { &*boot_info };
    framebuffer::init(info.framebuffer);
    framebuffer::clear(0x0f, 0x17, 0x2a);

    gdt::init();
    interrupts::init();

    framebuffer::set_color(0xf5, 0xa6, 0x23);
    framebuffer::write_line("ALOHAOS");
    framebuffer::set_color(0xd7, 0xe0, 0xee);
    framebuffer::write_line("");
    framebuffer::write_line("GDT  READY");
    framebuffer::write_line("TSS  READY");
    framebuffer::write_line("IDT  READY");
    framebuffer::write_line("");
    framebuffer::write_line("CPU EXCEPTION HANDLERS READY");

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
