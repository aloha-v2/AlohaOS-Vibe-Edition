//! Physical 4 KiB frame allocator initialized from AlohaBoot's UEFI map.

use common::{MemoryMapInfo, MemoryRegionKind, MAX_MEMORY_REGIONS};

pub const FRAME_SIZE: u64 = 4096;

#[derive(Clone, Copy)]
struct FreeRun {
    next: u64,
    end: u64,
}

impl FreeRun {
    const EMPTY: Self = Self { next: 0, end: 0 };
}

pub struct FrameStats {
    pub total_usable: u64,
    pub free: u64,
    pub allocated: u64,
}

struct PhysicalFrameAllocator {
    runs: [FreeRun; MAX_MEMORY_REGIONS],
    run_count: usize,
    current_run: usize,
    total_frames: u64,
    allocated_frames: u64,
}

impl PhysicalFrameAllocator {
    const EMPTY: Self = Self {
        runs: [FreeRun::EMPTY; MAX_MEMORY_REGIONS],
        run_count: 0,
        current_run: 0,
        total_frames: 0,
        allocated_frames: 0,
    };
}

static mut ALLOCATOR: PhysicalFrameAllocator = PhysicalFrameAllocator::EMPTY;

pub unsafe fn init(map: MemoryMapInfo) {
    let regions = core::slice::from_raw_parts(map.regions, map.region_count);
    let allocator = core::ptr::addr_of_mut!(ALLOCATOR);
    *allocator = PhysicalFrameAllocator::EMPTY;

    for region in regions {
        if region.kind != MemoryRegionKind::Usable || region.page_count == 0 {
            continue;
        }
        let index = (*allocator).run_count;
        if index == MAX_MEMORY_REGIONS {
            break;
        }
        let start = align_up(region.physical_start, FRAME_SIZE);
        let end = region.physical_start.saturating_add(
            region.page_count.saturating_mul(FRAME_SIZE),
        );
        if start >= end {
            continue;
        }
        (*allocator).runs[index] = FreeRun { next: start, end };
        (*allocator).run_count += 1;
        (*allocator).total_frames += (end - start) / FRAME_SIZE;
    }
}

pub fn allocate_frame() -> Option<u64> {
    allocate_contiguous(1)
}

/// Reserve `count` physically contiguous 4 KiB frames.
///
/// This is deliberately simple and monotonic. Reclamation will be added when
/// process address spaces need frame deallocation.
pub fn allocate_contiguous(count: u64) -> Option<u64> {
    if count == 0 {
        return None;
    }
    let bytes = count.checked_mul(FRAME_SIZE)?;
    unsafe {
        let allocator = core::ptr::addr_of_mut!(ALLOCATOR);
        let mut index = (*allocator).current_run;
        while index < (*allocator).run_count {
            let run = &mut (*allocator).runs[index];
            if run.next.checked_add(bytes)? <= run.end {
                let start = run.next;
                run.next += bytes;
                (*allocator).allocated_frames += count;
                (*allocator).current_run = index;
                return Some(start);
            }
            index += 1;
            (*allocator).current_run = index;
        }
        None
    }
}

pub fn stats() -> FrameStats {
    unsafe {
        let allocator = core::ptr::addr_of!(ALLOCATOR);
        FrameStats {
            total_usable: (*allocator).total_frames,
            free: (*allocator).total_frames - (*allocator).allocated_frames,
            allocated: (*allocator).allocated_frames,
        }
    }
}

const fn align_up(value: u64, alignment: u64) -> u64 {
    (value + alignment - 1) & !(alignment - 1)
}
