#![no_std]

/// Text shown by the first AlohaOS kernel bootstrap.
/// Keeping this in the kernel crate makes the boot boundary explicit while
/// AlohaBoot still runs as a single UEFI image during stage 0.
#[must_use]
pub const fn boot_banner() -> &'static str {
    "AlohaOS"
}
