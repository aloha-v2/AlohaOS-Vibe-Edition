//! Legacy 8259 PIC, PIT/keyboard IRQ routing, and machine reset.

use core::arch::asm;

pub const MASTER_OFFSET: u8 = 32;
pub const TIMER_VECTOR: u8 = MASTER_OFFSET;
pub const KEYBOARD_VECTOR: u8 = MASTER_OFFSET + 1;

const MASTER_COMMAND: u16 = 0x20;
const MASTER_DATA: u16 = 0x21;
const SLAVE_COMMAND: u16 = 0xa0;
const SLAVE_DATA: u16 = 0xa1;
const EOI: u8 = 0x20;

pub unsafe fn init_timer_and_keyboard() {
    outb(MASTER_COMMAND, 0x11); io_wait();
    outb(SLAVE_COMMAND, 0x11); io_wait();
    outb(MASTER_DATA, MASTER_OFFSET); io_wait();
    outb(SLAVE_DATA, MASTER_OFFSET + 8); io_wait();
    outb(MASTER_DATA, 4); io_wait();
    outb(SLAVE_DATA, 2); io_wait();
    outb(MASTER_DATA, 1); io_wait();
    outb(SLAVE_DATA, 1); io_wait();
    outb(MASTER_DATA, 0xfc);
    outb(SLAVE_DATA, 0xff);
}

pub unsafe fn end_of_interrupt(vector: u8) {
    if vector >= MASTER_OFFSET + 8 { outb(SLAVE_COMMAND, EOI); }
    outb(MASTER_COMMAND, EOI);
}

pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}

pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

unsafe fn io_wait() { outb(0x80, 0); }

pub fn reboot() -> ! {
    unsafe {
        asm!("cli", options(nomem, nostack));

        // Q35 reset-control register. Set SYS_RST first, then assert RCPU.
        // The two-step sequence is more reliable than writing 0x06 directly.
        outb(0xcf9, 0x02);
        io_wait();
        outb(0xcf9, 0x06);

        // 8042 fallback for chipsets without CF9 reset support.
        for _ in 0..100_000 {
            if inb(0x64) & 0x02 == 0 {
                outb(0x64, 0xfe);
                break;
            }
            asm!("pause", options(nomem, nostack));
        }

        loop { asm!("hlt", options(nomem, nostack)); }
    }
}
