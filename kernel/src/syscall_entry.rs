//! Architecture-independent validation for the future SYSCALL entry path.
//!
//! MSRs and assembly are intentionally enabled only after these invariants are
//! tested. `sysretq` is unsafe for noncanonical state, so doubtful frames are
//! forced onto an `iretq` return path.

use core::arch::x86_64::__cpuid;
use crate::address_space::{USER_REGION_END, USER_REGION_START};

const RFLAGS_ALWAYS_ONE: u64 = 1 << 1;
const RFLAGS_INTERRUPT_ENABLE: u64 = 1 << 9;
const FORBIDDEN_USER_RFLAGS: u64 = (3 << 12) | (1 << 14) | (1 << 17);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnPath { Sysret, Iret, Reject }

#[derive(Clone, Copy, Debug)]
pub struct UserReturnFrame {
    pub rip: u64,
    pub rsp: u64,
    pub rflags: u64,
}

impl UserReturnFrame {
    pub fn sanitized(mut self) -> Self {
        self.rflags |= RFLAGS_ALWAYS_ONE | RFLAGS_INTERRUPT_ENABLE;
        self.rflags &= !FORBIDDEN_USER_RFLAGS;
        self
    }

    pub fn return_path(self) -> ReturnPath {
        if !is_user_address(self.rip) || !is_user_address(self.rsp) {
            return ReturnPath::Reject;
        }
        if self.rflags & FORBIDDEN_USER_RFLAGS != 0 {
            return ReturnPath::Iret;
        }
        ReturnPath::Sysret
    }
}

pub fn cpu_supports_syscall() -> bool {
    let maximum = unsafe { __cpuid(0x8000_0000) }.eax;
    maximum >= 0x8000_0001 && (unsafe { __cpuid(0x8000_0001) }.edx & (1 << 11)) != 0
}

pub const fn is_canonical(address: u64) -> bool {
    address <= 0x0000_7fff_ffff_ffff || address >= 0xffff_8000_0000_0000
}

pub const fn is_user_address(address: u64) -> bool {
    is_canonical(address) && address >= USER_REGION_START && address < USER_REGION_END
}
