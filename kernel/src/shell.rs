//! Minimal allocation-free AlohaOS command shell.

use crate::{framebuffer, heap, keyboard, memory, pic};

const COMMAND_CAPACITY: usize = 128;

pub fn run() -> ! {
    banner();
    let mut command = [0u8; COMMAND_CAPACITY];
    let mut length = 0usize;

    loop {
        while let Some(scancode) = keyboard::pop_scancode() {
            let Some(character) = keyboard::decode(scancode) else { continue };
            match character {
                b'\n' => {
                    framebuffer::write_byte(b'\n');
                    execute(&command[..length]);
                    length = 0;
                    prompt();
                }
                8 => {
                    // Do not let backspace eat the prompt or previous output.
                    if length > 0 { length -= 1; framebuffer::write_byte(8); }
                }
                byte if byte.is_ascii_graphic() || byte == b' ' => {
                    if length < COMMAND_CAPACITY { command[length] = byte.to_ascii_lowercase(); length += 1; framebuffer::write_byte(byte); }
                }
                _ => {}
            }
        }
        unsafe { core::arch::asm!("sti", "hlt", options(nomem, nostack)) };
    }
}

fn banner() {
    framebuffer::clear_console();
    framebuffer::set_color(0xf5,0xa6,0x23);
    framebuffer::write_line("ALOHAOS SHELL");
    framebuffer::set_color(0xd7,0xe0,0xee);
    framebuffer::write_line("TYPE HELP FOR COMMANDS");
    framebuffer::write_line("");
    prompt();
}

fn prompt() { framebuffer::set_color(0xf5,0xa6,0x23); framebuffer::write_text("ALOHA> "); framebuffer::set_color(255,255,255); }

fn execute(raw: &[u8]) {
    let command = trim(raw);
    match command {
        b"" => {}
        b"help" => { framebuffer::write_line("HELP CLEAR MEMINFO REBOOT"); }
        b"clear" => { framebuffer::clear_console(); framebuffer::set_color(0xf5,0xa6,0x23); framebuffer::write_line("ALOHAOS SHELL"); }
        b"meminfo" => meminfo(),
        b"reboot" => { framebuffer::write_line("REBOOTING"); pic::reboot(); }
        _ => { framebuffer::set_color(255,0x80,0x80); framebuffer::write_line("UNKNOWN COMMAND"); framebuffer::set_color(255,255,255); }
    }
}

fn meminfo() {
    let frames = memory::stats();
    let heap = heap::stats();
    framebuffer::write_label_dec("PHYSICAL TOTAL KB: ", frames.total_usable * 4);
    framebuffer::write_label_dec("PHYSICAL FREE KB:  ", frames.free * 4);
    framebuffer::write_label_dec("PHYSICAL USED KB:  ", frames.allocated * 4);
    framebuffer::write_label_dec("HEAP SIZE KB:      ", (heap.size / 1024) as u64);
    framebuffer::write_label_dec("HEAP USED BYTES:   ", heap.used as u64);
    framebuffer::write_label_dec("DROPPED KEYS:      ", keyboard::dropped_scancodes() as u64);
}

fn trim(mut value: &[u8]) -> &[u8] {
    while value.first() == Some(&b' ') { value = &value[1..]; }
    while value.last() == Some(&b' ') { value = &value[..value.len()-1]; }
    value
}
