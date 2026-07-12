//! Feature-gated smoke checks executed by headless QEMU CI.

use alloc::{boxed::Box, string::String, vec::Vec};
use core::arch::asm;

use crate::{address_space, gdt, heap, keyboard, memory, process, serial};

#[cfg(feature = "m0-smoke")]
pub fn run_nonfatal() {
    heap_smoke();
    frame_reclamation_smoke();
    keyboard_smoke();
    user_descriptor_smoke();
    process_foundation_smoke();
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
    assert_eq!(gdt::user_code_selector(), 0x23);
    assert_ne!(gdt::rsp0(), 0);
    serial::info(format_args!("m1-smoke: ring3 descriptors and rsp0 passed"));
}

#[cfg(feature = "m0-smoke")]
fn process_foundation_smoke() {
    let before = memory::stats().allocated;
    let mut process = process::Process::new(7).expect("process setup failed");
    assert_eq!(process.pid, 7);
    assert_eq!(process.state, process::ProcessState::Ready);

    let data = process.user_stack_top - memory::FRAME_SIZE;
    process.address_space.copy_to_user(data, b"ALOHA").unwrap();
    let mut copied = [0u8; 5];
    process.address_space.copy_from_user(&mut copied, data).unwrap();
    assert_eq!(&copied, b"ALOHA");
    assert_eq!(
        process.address_space.copy_to_user(process.entry, b"X"),
        Err(address_space::UserAccessError::NotWritable)
    );
    assert_eq!(
        process.address_space.validate_user_range(u64::MAX - 2, 8, false),
        Err(address_space::UserAccessError::NonCanonical)
    );

    let original_root;
    {
        let guard = process.address_space.activate();
        assert_eq!(guard.active_root(), process.address_space.root_frame());
        original_root = guard.active_root();
    }
    assert_ne!(original_root, 0);
    process.mark_running();
    process.exit(42);
    assert_eq!(process.state, process::ProcessState::Exited);
    assert_eq!(process.exit_code, 42);
    drop(process);
    assert_eq!(memory::stats().allocated, before);
    serial::info(format_args!("m1-smoke: CR3 process user-copy lifecycle passed"));
}

#[cfg(feature = "exception-smoke")]
pub fn trigger_exception() -> ! {
    serial::info(format_args!("exception-smoke: triggering breakpoint"));
    unsafe { asm!("int3", options(nomem, nostack)) };
    panic!("breakpoint handler returned")
}
#[cfg(not(feature = "exception-smoke"))]
pub fn trigger_exception() {}
