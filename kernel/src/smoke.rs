//! Feature-gated smoke checks executed by headless QEMU CI.

use alloc::{boxed::Box, string::String, vec::Vec};
use core::arch::asm;
use crate::{address_space, elf, gdt, heap, keyboard, memory, process, process_table, serial, syscall, syscall_arch, syscall_entry};

#[cfg(feature = "m0-smoke")]
pub fn run_nonfatal() {
    heap_smoke();
    frame_reclamation_smoke();
    keyboard_smoke();
    user_descriptor_smoke();
    process_foundation_smoke();
    syscall_smoke();
    elf_loader_smoke();
    process_table_smoke();
    serial::info(format_args!("m0-smoke: heap keyboard memory passed"));
}
#[cfg(not(feature = "m0-smoke"))]
pub fn run_nonfatal() {}

#[cfg(feature = "m0-smoke")]
fn heap_smoke() {
    let before = heap::stats();
    let boxed = Box::new(0xa10a_05u64);
    let values = Vec::from([3u64, 5, 8, 13]);
    let text = String::from("ALOHA-M0");
    assert_eq!(*boxed, 0xa10a_05);
    assert_eq!(values.iter().copied().sum::<u64>(), 29);
    assert_eq!(text.as_bytes(), b"ALOHA-M0");
    assert!(heap::stats().used > before.used);
    drop((boxed, values, text));
    serial::info(format_args!("m0-smoke: heap passed"));
}

#[cfg(feature = "m0-smoke")]
fn frame_reclamation_smoke() {
    let frame = memory::allocate_frame().unwrap();
    assert!(unsafe { memory::deallocate_frame(frame) });
    assert_eq!(memory::allocate_frame().unwrap(), frame);
    serial::info(format_args!("m0-smoke: frame reclamation passed"));
}

#[cfg(feature = "m0-smoke")]
fn keyboard_smoke() {
    keyboard::reset_decoder_for_smoke();
    assert!(matches!(keyboard::decode(0x10), Some(keyboard::Key::Character(b'q'))));
    assert!(keyboard::decode(0x2a).is_none());
    assert!(matches!(keyboard::decode(0x1e), Some(keyboard::Key::Character(b'A'))));
    assert!(keyboard::decode(0xaa).is_none());
    assert!(matches!(keyboard::decode(0x1c), Some(keyboard::Key::Enter)));
    serial::info(format_args!("m0-smoke: keyboard decode passed"));
}

#[cfg(feature = "m0-smoke")]
fn user_descriptor_smoke() {
    assert_eq!(gdt::user_data_selector(), 0x1b);
    assert_ne!(gdt::rsp0(), 0);
}

#[cfg(feature = "m0-smoke")]
fn process_foundation_smoke() {
    let before = memory::stats().allocated;
    let mut process = process::Process::new(7).unwrap();
    let data = process.user_stack_top - memory::FRAME_SIZE;
    process.address_space.copy_to_user(data, b"ALOHA").unwrap();
    let mut output = [0; 5];
    process.address_space.copy_from_user(&mut output, data).unwrap();
    assert_eq!(&output, b"ALOHA");
    assert_eq!(
        process.address_space.copy_to_user(process.entry, b"X"),
        Err(address_space::UserAccessError::NotWritable)
    );
    process.exit(42);
    drop(process);
    assert_eq!(memory::stats().allocated, before);
}

#[cfg(feature = "m0-smoke")]
fn syscall_smoke() {
    let mut process = process::Process::new(9).unwrap();
    let data = process.user_stack_top - memory::FRAME_SIZE;
    process.address_space.copy_to_user(data, b"syscall").unwrap();
    assert_eq!(syscall::dispatch(&mut process, syscall::SYS_WRITE, [data, 7, 0, 0, 0, 0]).value, 7);
    let frame = syscall_entry::UserReturnFrame {
        rip: process::USER_CODE_BASE,
        rsp: process::USER_STACK_TOP - 16,
        rflags: 0x202,
    };
    assert_eq!(frame.return_path(), syscall_entry::ReturnPath::Sysret);
    assert!(syscall_arch::configuration_valid());
}

#[cfg(feature = "m0-smoke")]
fn elf_loader_smoke() {
    let image = elf::test_image();
    let mut process = process::Process::new(12).unwrap();
    process.load_elf(&image).unwrap();
    let mut bytes = [0u8; 32];
    process.address_space.copy_from_user(&mut bytes, process.entry).unwrap();
    assert_eq!(&bytes[..13], &image[120..133]);
    assert_eq!(&bytes[13..], [0u8; 19]);
}

#[cfg(feature = "m0-smoke")]
fn process_table_smoke() {
    process_table::reset_for_smoke();
    let parent = process_table::spawn(None).unwrap();
    let child = process_table::spawn(Some(parent)).unwrap();
    let sibling = process_table::spawn(Some(parent)).unwrap();
    assert_ne!(parent, child);
    assert_eq!(process_table::wait(parent, child), Err(process_table::TableError::StillRunning));
    process_table::exit(child, 37).unwrap();
    assert_eq!(process_table::wait(parent, child), Ok(37));
    assert!(process_table::lookup(child).is_none());
    assert_eq!(process_table::orphan_children(parent), 1);
    assert_eq!(process_table::lookup(sibling).unwrap().parent, 0);
    assert_eq!(process_table::wait(parent, sibling), Err(process_table::TableError::NotChild));
    serial::info(format_args!("m1-smoke: process table PID wait reap orphan passed"));
}

#[cfg(feature = "ring3-smoke")]
pub fn run_ring3() -> ! {
    const MARKER: u32 = 0xa10a_0033;
    let image = [0xb8, (MARKER & 0xff) as u8, ((MARKER >> 8) & 0xff) as u8, ((MARKER >> 16) & 0xff) as u8, ((MARKER >> 24) & 0xff) as u8, 0xcd, 0x80, 0x0f, 0x0b];
    let mut process = process::Process::new(33).unwrap();
    assert!(process.load_bootstrap_image(&image));
    serial::info(format_args!("ring3-smoke: entering user mode"));
    assert_eq!(crate::user_mode::run(&mut process), MARKER as u64);
    serial::info(format_args!("ring3-smoke: user iretq rsp0 trap passed"));
    crate::halt()
}

#[cfg(feature = "syscall-smoke")]
pub fn run_syscall() -> ! {
    let mut process = process::Process::new(44).unwrap();
    let message = process.user_stack_top - memory::FRAME_SIZE;
    process.address_space.copy_to_user(message, b"SYSCALL").unwrap();
    let mut image = [0u8; 36];
    let mut index = 0;
    macro_rules! emit { ($($byte:expr),* $(,)?) => { $(image[index] = $byte; index += 1;)* }; }
    emit!(0xb8, 1, 0, 0, 0, 0x48, 0xbf);
    for byte in message.to_le_bytes() { image[index] = byte; index += 1; }
    emit!(0xbe, 7, 0, 0, 0, 0x0f, 0x05, 0xb8, 2, 0, 0, 0, 0xbf, 42, 0, 0, 0, 0x0f, 0x05, 0x0f, 0x0b);
    assert!(process.load_bootstrap_image(&image));
    serial::info(format_args!("syscall-smoke: entering real syscall path"));
    crate::user_mode::run(&mut process);
    assert_eq!(process.exit_code, 42);
    serial::info(format_args!("syscall-smoke: write and exit passed"));
    crate::halt()
}

#[cfg(feature = "elf-smoke")]
pub fn run_elf() -> ! {
    let image = elf::test_image();
    let mut process = process::Process::new(55).unwrap();
    process.load_elf(&image).unwrap();
    serial::info(format_args!("elf-smoke: entering loaded ELF"));
    crate::user_mode::run(&mut process);
    assert_eq!(process.exit_code, 37);
    serial::info(format_args!("elf-smoke: loaded ELF syscall exit passed"));
    crate::halt()
}

#[cfg(feature = "exception-smoke")]
pub fn trigger_exception() -> ! {
    serial::info(format_args!("exception-smoke: triggering breakpoint"));
    unsafe { asm!("int3", options(nomem, nostack)) };
    panic!("breakpoint handler returned")
}
#[cfg(not(feature = "exception-smoke"))]
pub fn trigger_exception() {}
