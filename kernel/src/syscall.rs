//! Versioned syscall ABI and safe Rust dispatcher.
//!
//! This layer deliberately lands before the SYSCALL/SYSRET assembly entry. It
//! freezes numbers, errno encoding and pointer validation independently of the
//! CPU entry mechanism, so both `int 0x80` smoke code and future LSTAR entry can
//! share one audited dispatcher.

use crate::{process::{Process, ProcessState}, serial};

pub const ABI_VERSION: u64 = 1;
pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT: u64 = 2;
pub const SYS_SLEEP: u64 = 3;
const MAX_WRITE: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i64)]
pub enum Errno {
    Invalid = 1,
    BadAddress = 2,
    TooLarge = 3,
    NotSupported = 4,
}

impl Errno {
    pub const fn encoded(self) -> u64 {
        (-(self as i64)) as u64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyscallResult {
    pub value: u64,
    pub terminated: bool,
}

impl SyscallResult {
    const fn ok(value: u64) -> Self {
        Self { value, terminated: false }
    }

    const fn error(error: Errno) -> Self {
        Self { value: error.encoded(), terminated: false }
    }
}

pub fn dispatch(
    process: &mut Process,
    number: u64,
    arguments: [u64; 6],
) -> SyscallResult {
    match number {
        SYS_WRITE => write(process, arguments[0], arguments[1]),
        SYS_EXIT => {
            process.exit(arguments[0] as i32);
            SyscallResult { value: 0, terminated: true }
        }
        SYS_SLEEP => {
            if arguments[0] == 0 {
                return SyscallResult::error(Errno::Invalid);
            }
            process.state = ProcessState::Sleeping;
            SyscallResult::ok(0)
        }
        _ => SyscallResult::error(Errno::NotSupported),
    }
}

fn write(process: &Process, user_address: u64, length: u64) -> SyscallResult {
    let Ok(length) = usize::try_from(length) else {
        return SyscallResult::error(Errno::TooLarge);
    };
    if length > MAX_WRITE {
        return SyscallResult::error(Errno::TooLarge);
    }
    let mut buffer = [0u8; MAX_WRITE];
    if process
        .address_space
        .copy_from_user(&mut buffer[..length], user_address)
        .is_err()
    {
        return SyscallResult::error(Errno::BadAddress);
    }

    // Serial output remains byte-safe even when user data is not UTF-8.
    serial::info(format_args!("user[{}] write {} bytes", process.pid, length));
    for chunk in buffer[..length].chunks(32) {
        if let Ok(text) = core::str::from_utf8(chunk) {
            serial::info(format_args!("user: {}", text));
        } else {
            serial::info(format_args!("user: <binary {} bytes>", chunk.len()));
        }
    }
    SyscallResult::ok(length as u64)
}
