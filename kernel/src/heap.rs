//! Allocation-free bootstrap of a monotonic kernel heap.
//!
//! The heap is backed by contiguous physical frames and accessed through the
//! higher-half direct map. Deallocation is intentionally a no-op for now; this
//! allocator is ideal for early kernel objects with system lifetime.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::{memory, paging};

pub const HEAP_SIZE: usize = 1024 * 1024;
const HEAP_FRAMES: u64 = HEAP_SIZE as u64 / memory::FRAME_SIZE;

struct HeapState {
    start: usize,
    next: usize,
    end: usize,
    initialized: bool,
}

pub struct BumpHeap {
    locked: AtomicBool,
    state: UnsafeCell<HeapState>,
}

unsafe impl Sync for BumpHeap {}

impl BumpHeap {
    const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            state: UnsafeCell::new(HeapState {
                start: 0,
                next: 0,
                end: 0,
                initialized: false,
            }),
        }
    }

    fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

#[global_allocator]
static HEAP: BumpHeap = BumpHeap::new();

unsafe impl GlobalAlloc for BumpHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock();
        let state = &mut *self.state.get();
        if !state.initialized {
            self.unlock();
            return ptr::null_mut();
        }

        let start = align_up(state.next, layout.align());
        let Some(end) = start.checked_add(layout.size()) else {
            self.unlock();
            return ptr::null_mut();
        };
        if end > state.end {
            self.unlock();
            return ptr::null_mut();
        }

        state.next = end;
        self.unlock();
        start as *mut u8
    }

    unsafe fn dealloc(&self, _pointer: *mut u8, _layout: Layout) {
        // Early kernel allocations live until shutdown. A reclaiming linked-list
        // allocator can replace this without changing callers.
    }
}

pub struct HeapStats {
    pub virtual_start: u64,
    pub physical_start: u64,
    pub size: usize,
    pub used: usize,
    pub free: usize,
}

pub fn init() -> Option<HeapStats> {
    let physical_start = memory::allocate_contiguous(HEAP_FRAMES)?;
    let virtual_start = paging::PHYSICAL_MEMORY_OFFSET.checked_add(physical_start)? as usize;

    HEAP.lock();
    unsafe {
        let state = &mut *HEAP.state.get();
        state.start = virtual_start;
        state.next = virtual_start;
        state.end = virtual_start + HEAP_SIZE;
        state.initialized = true;
    }
    HEAP.unlock();
    Some(stats_with_physical(physical_start))
}

pub fn stats() -> HeapStats {
    stats_with_physical(0)
}

fn stats_with_physical(physical_start: u64) -> HeapStats {
    HEAP.lock();
    let result = unsafe {
        let state = &*HEAP.state.get();
        HeapStats {
            virtual_start: state.start as u64,
            physical_start,
            size: state.end.saturating_sub(state.start),
            used: state.next.saturating_sub(state.start),
            free: state.end.saturating_sub(state.next),
        }
    };
    HEAP.unlock();
    result
}

const fn align_up(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}
