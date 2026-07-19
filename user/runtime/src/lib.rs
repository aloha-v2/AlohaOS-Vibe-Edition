//! Minimal `no_std` runtime and ABI v1 syscall wrappers for AlohaOS userland.
#![no_std]

use core::arch::asm;

pub const ABI_VERSION: u64 = 1;

const SYS_WRITE: u64 = 1;
const SYS_EXIT: u64 = 2;
const SYS_SLEEP: u64 = 3;
const SYS_WAIT: u64 = 4;
const SYS_GETPID: u64 = 5;
const SYS_OPEN: u64 = 6;
const SYS_READ: u64 = 7;
const SYS_CLOSE: u64 = 8;
const SYS_STAT: u64 = 9;
const SYS_MMAP: u64 = 10;

pub const PROT_READ: u64 = 1;
pub const PROT_WRITE: u64 = 2;
pub const PROT_EXEC: u64 = 4;
pub const MAP_ANONYMOUS: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errno {
    Invalid,
    BadAddress,
    TooLarge,
    NotSupported,
    NoSuchProcess,
    NotChild,
    Busy,
    NoSuchFile,
    BadHandle,
    TooManyOpen,
    OutOfMemory,
    Unknown(u64),
}

pub type Result<T> = core::result::Result<T, Errno>;

#[inline]
fn decode(value: u64) -> Result<u64> {
    let signed = value as i64;
    if signed >= 0 {
        return Ok(value);
    }
    Err(match signed.wrapping_neg() as u64 {
        1 => Errno::Invalid,
        2 => Errno::BadAddress,
        3 => Errno::TooLarge,
        4 => Errno::NotSupported,
        5 => Errno::NoSuchProcess,
        6 => Errno::NotChild,
        7 => Errno::Busy,
        8 => Errno::NoSuchFile,
        9 => Errno::BadHandle,
        10 => Errno::TooManyOpen,
        11 => Errno::OutOfMemory,
        number => Errno::Unknown(number),
    })
}

#[inline]
unsafe fn syscall6(number: u64, arguments: [u64; 6]) -> u64 {
    let mut result = number;
    asm!(
        "syscall",
        inlateout("rax") result,
        in("rdi") arguments[0],
        in("rsi") arguments[1],
        in("rdx") arguments[2],
        in("r10") arguments[3],
        in("r8") arguments[4],
        in("r9") arguments[5],
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );
    result
}

#[inline]
fn call(number: u64, arguments: [u64; 6]) -> Result<u64> {
    decode(unsafe { syscall6(number, arguments) })
}

pub fn write(bytes: &[u8]) -> Result<usize> {
    call(SYS_WRITE, [bytes.as_ptr() as u64, bytes.len() as u64, 0, 0, 0, 0])
        .map(|value| value as usize)
}

pub fn exit(code: i32) -> ! {
    unsafe { syscall6(SYS_EXIT, [code as u32 as u64, 0, 0, 0, 0, 0]); }
    loop { core::hint::spin_loop(); }
}

pub fn sleep(ticks: u64, now: u64) -> Result<()> {
    call(SYS_SLEEP, [ticks, now, 0, 0, 0, 0]).map(|_| ())
}

pub fn wait(pid: u64) -> Result<i32> {
    call(SYS_WAIT, [pid, 0, 0, 0, 0, 0]).map(|value| value as u32 as i32)
}

pub fn getpid() -> Result<u64> {
    call(SYS_GETPID, [0; 6])
}

pub fn open(path: &[u8]) -> Result<usize> {
    call(SYS_OPEN, [path.as_ptr() as u64, path.len() as u64, 0, 0, 0, 0])
        .map(|value| value as usize)
}

pub fn read(handle: usize, buffer: &mut [u8]) -> Result<usize> {
    call(SYS_READ, [handle as u64, buffer.as_mut_ptr() as u64, buffer.len() as u64, 0, 0, 0])
        .map(|value| value as usize)
}

pub fn close(handle: usize) -> Result<()> {
    call(SYS_CLOSE, [handle as u64, 0, 0, 0, 0, 0]).map(|_| ())
}

pub fn stat(path: &[u8]) -> Result<u64> {
    let mut size = 0u64;
    call(SYS_STAT, [path.as_ptr() as u64, path.len() as u64, &mut size as *mut u64 as u64, 0, 0, 0])?;
    Ok(size)
}

pub fn mmap(length: usize, protection: u64) -> Result<*mut u8> {
    call(SYS_MMAP, [0, length as u64, protection, MAP_ANONYMOUS, 0, 0])
        .map(|address| address as *mut u8)
}

/// Define the freestanding `_start` symbol and exit with the application's code.
#[macro_export]
macro_rules! entry {
    ($main:path) => {
        #[no_mangle]
        pub extern "C" fn _start() -> ! {
            let code: i32 = $main();
            $crate::exit(code)
        }
    };
}

/// Install a tiny panic handler suitable for early user programs.
#[macro_export]
macro_rules! panic_handler {
    () => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
            $crate::exit(101)
        }
    };
}
