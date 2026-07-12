//! Feature-gated smoke checks executed by headless QEMU CI.

use alloc::{boxed::Box, string::String, vec::Vec};
use core::arch::asm;

use crate::{address_space, gdt, heap, keyboard, memory, serial};

#[cfg(feature = "m0-smoke")]
pub fn run_nonfatal() {
    heap_smoke();
    frame_reclamation_smoke();
    keyboard_smoke();
    user_descriptor_smoke();
    user_address_space_smoke();
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
    let after = heap::stats();
    assert!(after.used > before.used);
    drop((boxed, values, text));
    serial::info(format_args!("m0-smoke: heap passed"));
}

#[cfg(feature = "m0-smoke")]
fn frame_reclamation_smoke() {
    let frame = memory::allocate_frame().expect("smoke frame allocation failed");
    assert!(unsafe { memory::deallocate_frame(frame) });
    let reused = memory::allocate_frame().expect("smoke frame reuse failed");
    assert_eq!(reused, frame);
    serial::info(format_args!("m0-smoke: frame reclamation passed"));
}

#[cfg(feature = "m0-smoke")]
fn keyboard_smoke() {
    keyboard::reset_decoder_for_smoke();
    assert!(matches!(
        keyboard::decode(0x10),
        Some(keyboard::Key::Character(b'q'))
    ));
    assert!(keyboard::decode(0x2a).is_none());
    assert!(matches!(
        keyboard::decode(0x1e),
        Some(keyboard::Key::Character(b'A'))
    ));
    assert!(keyboard::decode(0xaa).is_none());
    assert!(matches!(keyboard::decode(0x1c), Some(keyboard::Key::Enter)));
    assert!(keyboard::decode(0xe0).is_none());
    assert!(matches!(keyboard::decode(0x48), Some(keyboard::Key::Up)));
    serial::info(format_args!("m0-smoke: keyboard decode passed"));
}

#[cfg(feature = "m0-smoke")]
fn user_descriptor_smoke() {
    assert_eq!(gdt::kernel_data_selector(), 0x10);
    assert_eq!(gdt::user_data_selector(), 0x1b);
    assert_eq!(gdt::user_code_selector(), 0x23);
    assert_ne!(gdt::rsp0(), 0);
    assert_eq!(gdt::rsp0() & 0xf, 0);
    serial::info(format_args!("m1-smoke: ring3 descriptors and rsp0 passed"));
}

#[cfg(feature = "m0-smoke")]
fn user_address_space_smoke() {
    let before = memory::stats().allocated;
    let mut space = address_space::AddressSpace::new().expect("user PML4 allocation failed");
    let code_address = address_space::USER_REGION_START;
    let data_address = code_address + memory::FRAME_SIZE;
    let code_frame = space
        .map_zeroed_user_page(code_address, false, true)
        .expect("user code mapping failed");
    let data_frame = space
        .map_zeroed_user_page(data_address, true, false)
        .expect("user data mapping failed");

    let (translated_code, code_flags) = space.translate(code_address).unwrap();
    let (translated_data, data_flags) = space.translate(data_address).unwrap();
    assert_eq!(translated_code, code_frame);
    assert_eq!(translated_data, data_frame);
    assert_ne!(code_flags & address_space::user_flag(), 0);
    assert_eq!(code_flags & address_space::writable_flag(), 0);
    assert_eq!(code_flags & address_space::no_execute_flag(), 0);
    assert_ne!(data_flags & address_space::writable_flag(), 0);
    assert_ne!(data_flags & address_space::no_execute_flag(), 0);
    assert_ne!(space.root_frame(), 0);
    drop(space);
    assert_eq!(memory::stats().allocated, before);
    serial::info(format_args!("m1-smoke: user PML4 USER NX mappings passed"));
}

#[cfg(feature = "exception-smoke")]
pub fn trigger_exception() -> ! {
    serial::info(format_args!("exception-smoke: triggering breakpoint"));
    unsafe { asm!("int3", options(nomem, nostack)) };
    panic!("breakpoint handler returned")
}

#[cfg(not(feature = "exception-smoke"))]
pub fn trigger_exception() {}
