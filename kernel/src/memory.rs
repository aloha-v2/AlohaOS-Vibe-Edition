//! Physical 4 KiB frame allocator initialized from AlohaBoot's UEFI map.
//!
//! Fresh frames come from monotonic usable-memory runs. Released frames are
//! linked through their own first word and reused before untouched memory. The
//! allocator is protected by the kernel IRQ-safe lock, so timer preemption
//! cannot corrupt its metadata.

use common::{MemoryMapInfo, MemoryRegionKind, MAX_MEMORY_REGIONS};

use crate::sync::IrqSpinLock;

pub const FRAME_SIZE: u64 = 4096;
const NO_FRAME: u64 = u64::MAX;

#[derive(Clone, Copy)]
struct FreeRun {
    start: u64,
    next: u64,
    end: u64,
}

impl FreeRun {
    const EMPTY: Self = Self {
        start: 0,
        next: 0,
        end: 0,
    };

    fn contains(&self, frame: u64) -> bool {
        frame >= self.start && frame < self.end
    }
}

pub struct FrameStats {
    pub total_usable: u64,
    pub free: u64,
    pub allocated: u64,
    pub reclaimed: u64,
}

struct PhysicalFrameAllocator {
    runs: [FreeRun; MAX_MEMORY_REGIONS],
    run_count: usize,
    current_run: usize,
    total_frames: u64,
    allocated_frames: u64,
    reclaimed_head: u64,
    reclaimed_frames: u64,
    initialized: bool,
}

impl PhysicalFrameAllocator {
    const EMPTY: Self = Self {
        runs: [FreeRun::EMPTY; MAX_MEMORY_REGIONS],
        run_count: 0,
        current_run: 0,
        total_frames: 0,
        allocated_frames: 0,
        reclaimed_head: NO_FRAME,
        reclaimed_frames: 0,
        initialized: false,
    };

    fn owns(&self, frame: u64) -> bool {
        self.runs[..self.run_count]
            .iter()
            .any(|run| run.contains(frame))
    }

    fn allocate_reclaimed(&mut self) -> Option<u64> {
        if self.reclaimed_head == NO_FRAME {
            return None;
        }
        let frame = self.reclaimed_head;
        self.reclaimed_head = unsafe { (frame as *const u64).read() };
        self.reclaimed_frames -= 1;
        self.allocated_frames += 1;
        Some(frame)
    }

    fn allocate_fresh(&mut self, count: u64) -> Option<u64> {
        let bytes = count.checked_mul(FRAME_SIZE)?;
        let mut index = self.current_run;
        while index < self.run_count {
            let run = &mut self.runs[index];
            if run.next.checked_add(bytes)? <= run.end {
                let start = run.next;
                run.next += bytes;
                self.allocated_frames += count;
                self.current_run = index;
                return Some(start);
            }
            index += 1;
            self.current_run = index;
        }
        None
    }

    unsafe fn release(&mut self, frame: u64) -> bool {
        if !self.initialized
            || frame % FRAME_SIZE != 0
            || !self.owns(frame)
            || self.allocated_frames == 0
        {
            return false;
        }

        // SAFETY: callers may release only uniquely owned frames. Identity
        // mappings are retained when paging switches to the kernel PML4.
        (frame as *mut u64).write(self.reclaimed_head);
        self.reclaimed_head = frame;
        self.reclaimed_frames += 1;
        self.allocated_frames -= 1;
        true
    }
}

static ALLOCATOR: IrqSpinLock<PhysicalFrameAllocator> =
    IrqSpinLock::new(PhysicalFrameAllocator::EMPTY);

pub unsafe fn init(map: MemoryMapInfo) {
    let regions = core::slice::from_raw_parts(map.regions, map.region_count);
    let mut allocator = ALLOCATOR.lock();
    *allocator = PhysicalFrameAllocator::EMPTY;

    for region in regions {
        if region.kind != MemoryRegionKind::Usable || region.page_count == 0 {
            continue;
        }
        let index = allocator.run_count;
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
        allocator.runs[index] = FreeRun {
            start,
            next: start,
            end,
        };
        allocator.run_count += 1;
        allocator.total_frames += (end - start) / FRAME_SIZE;
    }
    allocator.initialized = allocator.run_count != 0;
}

pub fn allocate_frame() -> Option<u64> {
    let mut allocator = ALLOCATOR.lock();
    allocator
        .allocate_reclaimed()
        .or_else(|| allocator.allocate_fresh(1))
}

/// Reserve `count` physically contiguous 4 KiB frames.
///
/// Single-frame requests reuse reclaimed frames. Multi-frame requests remain
/// monotonic until a range-aware coalescing allocator is needed by user space.
pub fn allocate_contiguous(count: u64) -> Option<u64> {
    if count == 0 {
        return None;
    }
    if count == 1 {
        return allocate_frame();
    }
    ALLOCATOR.lock().allocate_fresh(count)
}

/// Return one uniquely owned frame to the allocator.
///
/// # Safety
///
/// The frame must no longer be mapped or referenced by any owner and must not
/// have been released already. Violating this contract can corrupt the free
/// list or hand the same physical memory to multiple subsystems.
pub unsafe fn deallocate_frame(frame: u64) -> bool {
    ALLOCATOR.lock().release(frame)
}

/// Return a uniquely owned contiguous range one frame at a time.
///
/// # Safety
///
/// The complete range must satisfy the ownership requirements documented on
/// [`deallocate_frame`]. A partial failure leaves earlier frames reclaimed.
pub unsafe fn deallocate_contiguous(start: u64, count: u64) -> bool {
    if count == 0 || start % FRAME_SIZE != 0 {
        return false;
    }
    for index in 0..count {
        let Some(frame) = index
            .checked_mul(FRAME_SIZE)
            .and_then(|offset| start.checked_add(offset))
        else {
            return false;
        };
        if !deallocate_frame(frame) {
            return false;
        }
    }
    true
}

pub fn stats() -> FrameStats {
    let allocator = ALLOCATOR.lock();
    FrameStats {
        total_usable: allocator.total_frames,
        free: allocator
            .total_frames
            .saturating_sub(allocator.allocated_frames),
        allocated: allocator.allocated_frames,
        reclaimed: allocator.reclaimed_frames,
    }
}

const fn align_up(value: u64, alignment: u64) -> u64 {
    value.saturating_add(alignment - 1) & !(alignment - 1)
}
