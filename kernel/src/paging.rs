//! Transitional x86_64 paging owned by the kernel.
//!
//! We preserve UEFI's lower-half identity mappings, install a fresh PML4, and
//! build a higher-half direct map for the first 512 GiB of physical memory.

use core::arch::asm;
use core::ptr;

use common::MemoryMapInfo;

use crate::memory;

pub const PHYSICAL_MEMORY_OFFSET: u64 = 0xffff_8000_0000_0000;
const PAGE_2M: u64 = 2 * 1024 * 1024;
const DIRECT_MAP_LIMIT: u64 = 512 * 1024 * 1024 * 1024;
const ENTRY_COUNT: usize = 512;
const ADDRESS_MASK: u64 = 0x000f_ffff_ffff_f000;
const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const HUGE_PAGE: u64 = 1 << 7;
const DIRECT_MAP_PML4_INDEX: usize = 256;

#[repr(C, align(4096))]
struct PageTable {
    entries: [u64; ENTRY_COUNT],
}

pub struct PagingStats {
    pub pml4_physical: u64,
    pub mapped_2m_pages: u64,
    pub table_frames: u64,
    pub mapped_physical_bytes: u64,
}

pub enum PagingError {
    OutOfFrames,
    PhysicalAddressTooLarge,
}

pub fn init(memory_map: MemoryMapInfo) -> Result<PagingStats, PagingError> {
    let maximum_physical = maximum_physical_address(memory_map);
    if maximum_physical > DIRECT_MAP_LIMIT {
        return Err(PagingError::PhysicalAddressTooLarge);
    }

    let old_cr3 = read_cr3() & ADDRESS_MASK;
    let new_pml4_frame = memory::allocate_frame().ok_or(PagingError::OutOfFrames)?;
    let direct_pdpt_frame = memory::allocate_frame().ok_or(PagingError::OutOfFrames)?;
    let mut table_frames = 2u64;

    unsafe {
        zero_frame(new_pml4_frame);
        zero_frame(direct_pdpt_frame);

        // Keep the firmware-created identity mappings alive while the kernel is
        // still linked at 0x200000. Only the higher-half direct-map slot is new.
        ptr::copy_nonoverlapping(
            old_cr3 as *const PageTable,
            new_pml4_frame as *mut PageTable,
            1,
        );
        let pml4 = &mut *(new_pml4_frame as *mut PageTable);
        pml4.entries[DIRECT_MAP_PML4_INDEX] = direct_pdpt_frame | PRESENT | WRITABLE;

        let pdpt = &mut *(direct_pdpt_frame as *mut PageTable);
        let mapped_end = align_up(maximum_physical, PAGE_2M);
        let mut physical = 0u64;
        let mut mapped_pages = 0u64;
        let mut current_pdpt_index = usize::MAX;
        let mut current_pd_frame = 0u64;

        while physical < mapped_end {
            let pdpt_index = (physical >> 30) as usize;
            if pdpt_index != current_pdpt_index {
                current_pd_frame = memory::allocate_frame().ok_or(PagingError::OutOfFrames)?;
                table_frames += 1;
                zero_frame(current_pd_frame);
                pdpt.entries[pdpt_index] = current_pd_frame | PRESENT | WRITABLE;
                current_pdpt_index = pdpt_index;
            }

            let page_directory = &mut *(current_pd_frame as *mut PageTable);
            let page_index = ((physical >> 21) & 0x1ff) as usize;
            page_directory.entries[page_index] = physical | PRESENT | WRITABLE | HUGE_PAGE;
            physical += PAGE_2M;
            mapped_pages += 1;
        }

        write_cr3(new_pml4_frame);

        Ok(PagingStats {
            pml4_physical: new_pml4_frame,
            mapped_2m_pages: mapped_pages,
            table_frames,
            mapped_physical_bytes: mapped_end,
        })
    }
}

/// Verify that an allocated physical frame and its higher-half alias address
/// the same storage. The caller owns `physical_frame`.
pub fn verify_direct_map(physical_frame: u64) -> bool {
    unsafe {
        let physical = physical_frame as *mut u64;
        let virtual_alias = PHYSICAL_MEMORY_OFFSET.wrapping_add(physical_frame) as *const u64;
        const TEST_VALUE: u64 = 0xa10a_05d1_4ec7_0001;
        ptr::write_volatile(physical, TEST_VALUE);
        ptr::read_volatile(virtual_alias) == TEST_VALUE
    }
}

fn maximum_physical_address(map: MemoryMapInfo) -> u64 {
    if map.regions.is_null() {
        return 0;
    }
    let regions = unsafe { core::slice::from_raw_parts(map.regions, map.region_count) };
    regions.iter().fold(0, |maximum, region| {
        let end = region
            .physical_start
            .saturating_add(region.page_count.saturating_mul(memory::FRAME_SIZE));
        maximum.max(end)
    })
}

unsafe fn zero_frame(frame: u64) {
    ptr::write_bytes(frame as *mut u8, 0, memory::FRAME_SIZE as usize);
}

fn read_cr3() -> u64 {
    let value: u64;
    unsafe { asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags)) };
    value
}

unsafe fn write_cr3(value: u64) {
    asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
}

const fn align_up(value: u64, alignment: u64) -> u64 {
    value.saturating_add(alignment - 1) & !(alignment - 1)
}
