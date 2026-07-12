//! Polling legacy VirtIO block driver for QEMU.
//!
//! Queue memory and request bookkeeping live in one scheduler-aware mutex. A
//! request keeps the guard until the device completes it, so concurrent callers
//! cannot overwrite DMA descriptors or the shared sector buffer. Interrupts
//! remain enabled while the polling device is busy.

use core::arch::asm;
use core::ptr::{addr_of, addr_of_mut};
use core::sync::atomic::{fence, Ordering};

use crate::sync::Mutex;

const VIRTIO_VENDOR: u16 = 0x1af4;
const VIRTIO_BLOCK_LEGACY: u16 = 0x1001;
const QUEUE_SIZE_MAX: u16 = 256;
const QUEUE_MEMORY_SIZE: usize = 3 * 4096;

#[repr(C, align(4096))]
struct QueueMemory([u8; QUEUE_MEMORY_SIZE]);

#[repr(C)]
struct RequestHeader {
    request_type: u32,
    reserved: u32,
    sector: u64,
}

#[repr(C, align(512))]
struct SectorBuffer([u8; 512]);

struct DriverState {
    queue: QueueMemory,
    header: RequestHeader,
    buffer: SectorBuffer,
    status: u8,
    io_base: u16,
    queue_size: u16,
    available_offset: usize,
    used_offset: usize,
    available_index: u16,
    last_used_index: u16,
    ready: bool,
}

impl DriverState {
    const EMPTY: Self = Self {
        queue: QueueMemory([0; QUEUE_MEMORY_SIZE]),
        header: RequestHeader {
            request_type: 0,
            reserved: 0,
            sector: 0,
        },
        buffer: SectorBuffer([0; 512]),
        status: 0xff,
        io_base: 0,
        queue_size: 0,
        available_offset: 0,
        used_offset: 0,
        available_index: 0,
        last_used_index: 0,
        ready: false,
    };
}

static DRIVER: Mutex<DriverState> = Mutex::new(DriverState::EMPTY);

pub fn init() -> bool {
    let Some((bus, device, function)) = find_device() else {
        return false;
    };
    let address = pci_read(bus, device, function, 0x10);
    if address & 1 == 0 {
        return false;
    }
    let io_base = (address & 0xfffc) as u16;

    let command = pci_read(bus, device, function, 0x04);
    pci_write(bus, device, function, 0x04, command | 0x0000_0005);

    let mut state = DRIVER.lock();
    state.ready = false;
    unsafe {
        outb(io_base + 18, 0);
        outb(io_base + 18, 1);
        outb(io_base + 18, 3);
        let _device_features = inl(io_base);
        outl(io_base + 4, 0);
        outw(io_base + 14, 0);

        let queue_size = inw(io_base + 12);
        if queue_size < 3 || queue_size > QUEUE_SIZE_MAX {
            return false;
        }

        let available_offset = 16 * queue_size as usize;
        let available_end = available_offset + 6 + 2 * queue_size as usize;
        let used_offset = align_up(available_end, 4096);
        let used_end = used_offset + 6 + 8 * queue_size as usize;
        if used_end > QUEUE_MEMORY_SIZE {
            return false;
        }

        core::ptr::write_bytes(state.queue.0.as_mut_ptr(), 0, QUEUE_MEMORY_SIZE);
        state.io_base = io_base;
        state.queue_size = queue_size;
        state.available_offset = available_offset;
        state.used_offset = used_offset;
        state.available_index = 0;
        state.last_used_index = 0;

        let queue_physical = addr_of!(state.queue) as u64;
        if queue_physical & 0xfff != 0 || queue_physical > u32::MAX as u64 * 4096 {
            return false;
        }
        outl(io_base + 8, (queue_physical >> 12) as u32);
        outb(io_base + 18, 7);
        state.ready = inb(io_base + 18) & 0x80 == 0;
        state.ready
    }
}

pub fn is_ready() -> bool {
    DRIVER.lock().ready
}

pub fn read_sector(sector: u64, output: &mut [u8; 512]) -> bool {
    let mut state = DRIVER.lock();
    if !state.ready {
        return false;
    }

    unsafe {
        let state_ptr: *mut DriverState = &mut *state;
        (*state_ptr).header = RequestHeader {
            request_type: 0,
            reserved: 0,
            sector,
        };
        (*state_ptr).status = 0xff;

        write_descriptor(
            state_ptr,
            0,
            addr_of!((*state_ptr).header) as u64,
            16,
            1,
            1,
        );
        write_descriptor(
            state_ptr,
            1,
            addr_of!((*state_ptr).buffer) as u64,
            512,
            3,
            2,
        );
        write_descriptor(
            state_ptr,
            2,
            addr_of!((*state_ptr).status) as u64,
            1,
            2,
            0,
        );

        let queue = addr_of_mut!((*state_ptr).queue).cast::<u8>();
        let ring_slot = (*state_ptr).available_index % (*state_ptr).queue_size;
        write_u16(
            queue.add((*state_ptr).available_offset + 4 + ring_slot as usize * 2),
            0,
        );
        fence(Ordering::Release);
        (*state_ptr).available_index = (*state_ptr).available_index.wrapping_add(1);
        write_u16(
            queue.add((*state_ptr).available_offset + 2),
            (*state_ptr).available_index,
        );
        outw((*state_ptr).io_base + 16, 0);

        let mut spins = 0usize;
        while read_u16(queue.add((*state_ptr).used_offset + 2))
            == (*state_ptr).last_used_index
        {
            core::hint::spin_loop();
            spins += 1;
            if spins == 50_000_000 {
                return false;
            }
        }
        (*state_ptr).last_used_index = (*state_ptr).last_used_index.wrapping_add(1);
        fence(Ordering::Acquire);
        if (*state_ptr).status != 0 {
            return false;
        }
        output.copy_from_slice(&(*state_ptr).buffer.0);
        true
    }
}

unsafe fn write_descriptor(
    state: *mut DriverState,
    index: usize,
    address: u64,
    length: u32,
    flags: u16,
    next: u16,
) {
    let pointer = addr_of_mut!((*state).queue).cast::<u8>().add(index * 16);
    core::ptr::write_unaligned(pointer.cast::<u64>(), address);
    core::ptr::write_unaligned(pointer.add(8).cast::<u32>(), length);
    core::ptr::write_unaligned(pointer.add(12).cast::<u16>(), flags);
    core::ptr::write_unaligned(pointer.add(14).cast::<u16>(), next);
}

unsafe fn write_u16(pointer: *mut u8, value: u16) {
    core::ptr::write_volatile(pointer.cast::<u16>(), value);
}

unsafe fn read_u16(pointer: *mut u8) -> u16 {
    core::ptr::read_volatile(pointer.cast::<u16>())
}

fn find_device() -> Option<(u8, u8, u8)> {
    for bus in 0u16..=255 {
        for device in 0u8..32 {
            for function in 0u8..8 {
                let value = pci_read(bus as u8, device, function, 0);
                if value as u16 == VIRTIO_VENDOR
                    && (value >> 16) as u16 == VIRTIO_BLOCK_LEGACY
                {
                    return Some((bus as u8, device, function));
                }
            }
        }
    }
    None
}

fn pci_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = 0x8000_0000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | (offset as u32 & 0xfc);
    unsafe {
        outl(0xcf8, address);
        inl(0xcfc)
    }
}

fn pci_write(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = 0x8000_0000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | (offset as u32 & 0xfc);
    unsafe {
        outl(0xcf8, address);
        outl(0xcfc, value);
    }
}

const fn align_up(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack));
    value
}

unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack));
    value
}

unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    asm!("in eax, dx", in("dx") port, out("eax") value, options(nomem, nostack));
    value
}

unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack));
}

unsafe fn outl(port: u16, value: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack));
}
