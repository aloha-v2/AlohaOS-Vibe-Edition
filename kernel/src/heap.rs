//! Reclaiming first-fit linked-list heap allocator.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::{align_of, size_of};
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::{memory, paging};

pub const HEAP_SIZE: usize = 1024 * 1024;
const HEAP_FRAMES: u64 = HEAP_SIZE as u64 / memory::FRAME_SIZE;

#[repr(C)]
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

struct HeapState {
    head: *mut FreeBlock,
    virtual_start: usize,
    physical_start: u64,
    allocated: usize,
    initialized: bool,
}

pub struct LinkedListHeap {
    locked: AtomicBool,
    state: UnsafeCell<HeapState>,
}

unsafe impl Sync for LinkedListHeap {}

impl LinkedListHeap {
    const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            state: UnsafeCell::new(HeapState {
                head: ptr::null_mut(),
                virtual_start: 0,
                physical_start: 0,
                allocated: 0,
                initialized: false,
            }),
        }
    }

    fn lock(&self) {
        while self.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
    }

    fn unlock(&self) { self.locked.store(false, Ordering::Release); }
}

#[global_allocator]
static HEAP: LinkedListHeap = LinkedListHeap::new();

unsafe impl GlobalAlloc for LinkedListHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = normalized(layout);
        self.lock();
        let state = &mut *self.state.get();
        if !state.initialized { self.unlock(); return ptr::null_mut(); }

        let mut link: *mut *mut FreeBlock = &mut state.head;
        while !(*link).is_null() {
            let block = *link;
            let block_start = block as usize;
            let block_end = block_start + (*block).size;
            let mut allocation_start = align_up(block_start, align);
            let mut prefix = allocation_start - block_start;
            if prefix != 0 && prefix < size_of::<FreeBlock>() {
                allocation_start = align_up(block_start + size_of::<FreeBlock>(), align);
                prefix = allocation_start - block_start;
            }
            let Some(allocation_end) = allocation_start.checked_add(size) else { break };
            if allocation_end <= block_end {
                let next = (*block).next;
                *link = next;
                if prefix >= size_of::<FreeBlock>() { add_region(state, block_start, prefix); }
                let suffix = block_end - allocation_end;
                if suffix >= size_of::<FreeBlock>() { add_region(state, allocation_end, suffix); }
                state.allocated += size;
                self.unlock();
                return allocation_start as *mut u8;
            }
            link = &mut (*block).next;
        }
        self.unlock();
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        let (size, _) = normalized(layout);
        self.lock();
        let state = &mut *self.state.get();
        add_region(state, pointer as usize, size);
        state.allocated = state.allocated.saturating_sub(size);
        self.unlock();
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
        state.head = ptr::null_mut();
        state.virtual_start = virtual_start;
        state.physical_start = physical_start;
        state.allocated = 0;
        state.initialized = true;
        add_region(state, virtual_start, HEAP_SIZE);
    }
    HEAP.unlock();
    Some(stats())
}

pub fn stats() -> HeapStats {
    HEAP.lock();
    let result = unsafe {
        let state = &*HEAP.state.get();
        HeapStats {
            virtual_start: state.virtual_start as u64,
            physical_start: state.physical_start,
            size: HEAP_SIZE,
            used: state.allocated,
            free: HEAP_SIZE.saturating_sub(state.allocated),
        }
    };
    HEAP.unlock();
    result
}

unsafe fn add_region(state: &mut HeapState, start: usize, size: usize) {
    if size < size_of::<FreeBlock>() || start % align_of::<FreeBlock>() != 0 { return; }
    let node = start as *mut FreeBlock;
    (*node).size = size;
    (*node).next = ptr::null_mut();

    let mut link: *mut *mut FreeBlock = &mut state.head;
    while !(*link).is_null() && (*link as usize) < start { link = &mut (**link).next; }
    (*node).next = *link;
    *link = node;
    coalesce(state);
}

unsafe fn coalesce(state: &mut HeapState) {
    let mut current = state.head;
    while !current.is_null() {
        let next = (*current).next;
        if !next.is_null() && current as usize + (*current).size == next as usize {
            (*current).size += (*next).size;
            (*current).next = (*next).next;
        } else {
            current = next;
        }
    }
}

fn normalized(layout: Layout) -> (usize, usize) {
    let align = layout.align().max(align_of::<FreeBlock>());
    let size = align_up(layout.size().max(size_of::<FreeBlock>()), align_of::<FreeBlock>());
    (size, align)
}

const fn align_up(value: usize, alignment: usize) -> usize { (value + alignment - 1) & !(alignment - 1) }
