//! Per-process x86_64 address spaces with a dedicated user PML4 slot.
//!
//! The current kernel still executes from identity-mapped low memory, so a new
//! process root copies the existing kernel mappings and reserves PML4 slot 1
//! exclusively for user pages. Page-table ownership is explicit and all frames
//! allocated by an address space are reclaimed on drop.

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
            // Never inherit another process's user region.
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
                unsafe {
                    ptr::write_bytes(child as *mut u8, 0, memory::FRAME_SIZE as usize)
                };
                table.entries[*index] = child | PRESENT | WRITABLE | USER;
                table_frame = child;
            } else {
                // User permission must propagate through every table level.
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

impl Drop for AddressSpace {
    fn drop(&mut self) {
        // Leaf/data frames and page-table frames are all uniquely owned. Drop
        // in reverse allocation order so the root is released last.
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

fn read_cr3() -> u64 {
    let value: u64;
    unsafe { asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags)) };
    value
}

pub const fn user_flag() -> u64 {
    USER
}

pub const fn writable_flag() -> u64 {
    WRITABLE
}

pub const fn no_execute_flag() -> u64 {
    NO_EXECUTE
}
