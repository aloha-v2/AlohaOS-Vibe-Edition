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
    (*allocator) = PhysicalFrameAllocator::EMPTY;

    for region in regions {
        if region.kind != MemoryRegionKind::Usable || region.page_count == 0 {
            continue;
        }
        let index = (*allocator).run_count;
        if index == MAX_MEMORY_REGIONS {
            break;
        }
        let start = align_up(region.physical_start, FRAME_SIZE);
        let end = region
            .physical_start
            .saturating_add(region.page_count.saturating_mul(FRAME_SIZE));
        if start >= end {
            continue;
        }
        (*allocator).runs[index] = FreeRun { next: start, end };
        (*allocator).run_count += 1;
        (*allocator).total_frames += (end - start) / FRAME_SIZE;
    }
}

pub fn allocate_frame() -> Option<u64> {
    unsafe {
        let allocator = core::ptr::addr_of_mut!(ALLOCATOR);
        while (*allocator).current_run < (*allocator).run_count {
            let run = &mut (*allocator).runs[(*allocator).current_run];
            if run.next < run.end {
                let frame = run.next;
                run.next += FRAME_SIZE;
                (*allocator).allocated_frames += 1;
                return Some(frame);
            }
            (*allocator).current_run += 1;
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
