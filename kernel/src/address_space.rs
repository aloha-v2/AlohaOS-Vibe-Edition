//! Per-process x86_64 address spaces, safe activation and user memory access.

use core::arch::asm;
use core::ptr;

use crate::memory;

pub const USER_REGION_START: u64 = 0x0000_0080_0000_0000;
pub const USER_REGION_END: u64 = 0x0000_0100_0000_0000;

const ENTRY_COUNT: usize = 512;
const ADDRESS_MASK: u64 = 0x000f_ffff_ffff_f000;
const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const USER: u64 = 1 << 2;
const NO_EXECUTE: u64 = 1 << 63;
const INTERRUPT_ENABLE: u64 = 1 << 9;
const USER_PML4_INDEX: usize = 1;
const MAX_OWNED_FRAMES: usize = 64;

#[repr(C, align(4096))]
struct PageTable {
    entries: [u64; ENTRY_COUNT],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapError {
    InvalidAddress,
    AlreadyMapped,
    OutOfFrames,
    OwnershipLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserAccessError {
    NonCanonical,
    OutsideUserRegion,
    Overflow,
    Unmapped,
    NotUser,
    NotWritable,
}

pub struct AddressSpace {
    root: u64,
    owned: [u64; MAX_OWNED_FRAMES],
    owned_count: usize,
}

impl AddressSpace {
    pub fn new() -> Result<Self, MapError> {
        let root = memory::allocate_frame().ok_or(MapError::OutOfFrames)?;
        unsafe {
            ptr::write_bytes(root as *mut u8, 0, memory::FRAME_SIZE as usize);
            let current = (read_cr3() & ADDRESS_MASK) as *const PageTable;
            ptr::copy_nonoverlapping(current, root as *mut PageTable, 1);
            (*(root as *mut PageTable)).entries[USER_PML4_INDEX] = 0;
        }
        let mut space = Self {
            root,
            owned: [0; MAX_OWNED_FRAMES],
            owned_count: 0,
        };
        space.track(root)?;
        Ok(space)
    }

    pub fn root_frame(&self) -> u64 {
        self.root
    }

    /// Activate this address space with interrupts disabled until guard drop.
    /// This prevents timer preemption from observing a temporary CR3.
    pub fn activate(&self) -> AddressSpaceGuard<'_> {
        let flags = read_rflags();
        unsafe { asm!("cli", options(nomem, nostack)) };
        let previous = read_cr3();
        if previous & ADDRESS_MASK != self.root {
            unsafe { write_cr3(self.root) };
        }
        AddressSpaceGuard {
            space: self,
            previous,
            restore_interrupts: flags & INTERRUPT_ENABLE != 0,
        }
    }

    pub fn map_zeroed_user_page(
        &mut self,
        virtual_address: u64,
        writable: bool,
        executable: bool,
    ) -> Result<u64, MapError> {
        if virtual_address % memory::FRAME_SIZE != 0
            || !(USER_REGION_START..USER_REGION_END).contains(&virtual_address)
        {
            return Err(MapError::InvalidAddress);
        }
        let page = memory::allocate_frame().ok_or(MapError::OutOfFrames)?;
        if let Err(error) = self.track(page) {
            unsafe { memory::deallocate_frame(page) };
            return Err(error);
        }
        unsafe { ptr::write_bytes(page as *mut u8, 0, memory::FRAME_SIZE as usize) };
        if let Err(error) = self.map_page(virtual_address, page, writable, executable) {
            self.untrack_last(page);
            unsafe { memory::deallocate_frame(page) };
            return Err(error);
        }
        Ok(page)
    }

    pub fn translate(&self, virtual_address: u64) -> Option<(u64, u64)> {
        let indices = indices(virtual_address);
        let mut table_frame = self.root;
        for index in &indices[..3] {
            let entry = unsafe { (*(table_frame as *const PageTable)).entries[*index] };
            if entry & PRESENT == 0 {
                return None;
            }
            table_frame = entry & ADDRESS_MASK;
        }
        let entry = unsafe { (*(table_frame as *const PageTable)).entries[indices[3]] };
        (entry & PRESENT != 0).then_some((
            (entry & ADDRESS_MASK) | (virtual_address & (memory::FRAME_SIZE - 1)),
            entry,
        ))
    }

    pub fn validate_user_range(
        &self,
        address: u64,
        length: usize,
        writable: bool,
    ) -> Result<(), UserAccessError> {
        if !is_canonical(address) {
            return Err(UserAccessError::NonCanonical);
        }
        if length == 0 {
            return if (USER_REGION_START..=USER_REGION_END).contains(&address) {
                Ok(())
            } else {
                Err(UserAccessError::OutsideUserRegion)
            };
        }
        let end = address
            .checked_add(length as u64)
            .ok_or(UserAccessError::Overflow)?;
        if address < USER_REGION_START || end > USER_REGION_END || end <= address {
            return Err(UserAccessError::OutsideUserRegion);
        }

        let mut page = address & !(memory::FRAME_SIZE - 1);
        let last = (end - 1) & !(memory::FRAME_SIZE - 1);
        loop {
            let (_, flags) = self.translate(page).ok_or(UserAccessError::Unmapped)?;
            if flags & USER == 0 {
                return Err(UserAccessError::NotUser);
            }
            if writable && flags & WRITABLE == 0 {
                return Err(UserAccessError::NotWritable);
            }
            if page == last {
                break;
            }
            page = page
                .checked_add(memory::FRAME_SIZE)
                .ok_or(UserAccessError::Overflow)?;
        }
        Ok(())
    }

    pub fn copy_from_user(
        &self,
        destination: &mut [u8],
        user_address: u64,
    ) -> Result<(), UserAccessError> {
        self.validate_user_range(user_address, destination.len(), false)?;
        self.copy_user_bytes(destination, user_address);
        Ok(())
    }

    pub fn copy_to_user(
        &self,
        user_address: u64,
        source: &[u8],
    ) -> Result<(), UserAccessError> {
        self.validate_user_range(user_address, source.len(), true)?;
        let mut copied = 0usize;
        while copied < source.len() {
            let virtual_address = user_address + copied as u64;
            let (physical, _) = self.translate(virtual_address).unwrap();
            let available = (memory::FRAME_SIZE - physical % memory::FRAME_SIZE) as usize;
            let count = available.min(source.len() - copied);
            unsafe {
                ptr::copy_nonoverlapping(source[copied..].as_ptr(), physical as *mut u8, count)
            };
            copied += count;
        }
        Ok(())
    }

    fn copy_user_bytes(&self, destination: &mut [u8], user_address: u64) {
        let mut copied = 0usize;
        while copied < destination.len() {
            let virtual_address = user_address + copied as u64;
            let (physical, _) = self.translate(virtual_address).unwrap();
            let available = (memory::FRAME_SIZE - physical % memory::FRAME_SIZE) as usize;
            let count = available.min(destination.len() - copied);
            unsafe {
                ptr::copy_nonoverlapping(
                    physical as *const u8,
                    destination[copied..].as_mut_ptr(),
                    count,
                )
            };
            copied += count;
        }
    }

    fn map_page(
        &mut self,
        virtual_address: u64,
        physical: u64,
        writable: bool,
        executable: bool,
    ) -> Result<(), MapError> {
        let indices = indices(virtual_address);
        let mut table_frame = self.root;
        for index in &indices[..3] {
            let table = unsafe { &mut *(table_frame as *mut PageTable) };
            let entry = table.entries[*index];
            if entry & PRESENT == 0 {
                let child = memory::allocate_frame().ok_or(MapError::OutOfFrames)?;
                if let Err(error) = self.track(child) {
                    unsafe { memory::deallocate_frame(child) };
                    return Err(error);
                }
                unsafe { ptr::write_bytes(child as *mut u8, 0, memory::FRAME_SIZE as usize) };
                table.entries[*index] = child | PRESENT | WRITABLE | USER;
                table_frame = child;
            } else {
                table.entries[*index] = entry | USER;
                table_frame = entry & ADDRESS_MASK;
            }
        }
        let table = unsafe { &mut *(table_frame as *mut PageTable) };
        let slot = &mut table.entries[indices[3]];
        if *slot & PRESENT != 0 {
            return Err(MapError::AlreadyMapped);
        }
        let mut flags = PRESENT | USER;
        if writable {
            flags |= WRITABLE;
        }
        if !executable {
            flags |= NO_EXECUTE;
        }
        *slot = physical | flags;
        Ok(())
    }

    fn track(&mut self, frame: u64) -> Result<(), MapError> {
        if self.owned_count == MAX_OWNED_FRAMES {
            return Err(MapError::OwnershipLimit);
        }
        self.owned[self.owned_count] = frame;
        self.owned_count += 1;
        Ok(())
    }

    fn untrack_last(&mut self, frame: u64) {
        if self.owned_count != 0 && self.owned[self.owned_count - 1] == frame {
            self.owned_count -= 1;
            self.owned[self.owned_count] = 0;
        }
    }
}

pub struct AddressSpaceGuard<'a> {
    space: &'a AddressSpace,
    previous: u64,
    restore_interrupts: bool,
}

impl AddressSpaceGuard<'_> {
    pub fn active_root(&self) -> u64 {
        self.space.root
    }
}

impl Drop for AddressSpaceGuard<'_> {
    fn drop(&mut self) {
        if read_cr3() != self.previous {
            unsafe { write_cr3(self.previous) };
        }
        if self.restore_interrupts {
            unsafe { asm!("sti", options(nomem, nostack)) };
        }
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        debug_assert_ne!(read_cr3() & ADDRESS_MASK, self.root);
        while self.owned_count != 0 {
            self.owned_count -= 1;
            let frame = self.owned[self.owned_count];
            self.owned[self.owned_count] = 0;
            unsafe {
                let _ = memory::deallocate_frame(frame);
            }
        }
    }
}

fn indices(address: u64) -> [usize; 4] {
    [
        ((address >> 39) & 0x1ff) as usize,
        ((address >> 30) & 0x1ff) as usize,
        ((address >> 21) & 0x1ff) as usize,
        ((address >> 12) & 0x1ff) as usize,
    ]
}

const fn is_canonical(address: u64) -> bool {
    address <= 0x0000_7fff_ffff_ffff || address >= 0xffff_8000_0000_0000
}

fn read_cr3() -> u64 {
    let value: u64;
    unsafe { asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags)) };
    value
}

unsafe fn write_cr3(value: u64) {
    asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
}

fn read_rflags() -> u64 {
    let value: u64;
    unsafe { asm!("pushfq", "pop {}", out(reg) value, options(nomem, preserves_flags)) };
    value
}

pub const fn user_flag() -> u64 { USER }
pub const fn writable_flag() -> u64 { WRITABLE }
pub const fn no_execute_flag() -> u64 { NO_EXECUTE }
