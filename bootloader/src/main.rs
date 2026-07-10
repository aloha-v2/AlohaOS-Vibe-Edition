#![no_main]
#![no_std]

use uefi::prelude::*;

/// AlohaBoot stage 0.
///
/// At this bootstrap stage the kernel is linked into the UEFI image. The next
/// milestone will load an independent kernel ELF, obtain the memory map and
/// leave UEFI boot services before transferring control.
#[entry]
fn efi_main() -> Status {
    if uefi::helpers::init().is_err() {
        return Status::ABORTED;
    }

    uefi::println!("{}", aloha_kernel::boot_banner());
    Status::SUCCESS
}
