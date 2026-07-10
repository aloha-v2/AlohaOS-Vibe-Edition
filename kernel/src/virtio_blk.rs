//! Polling legacy VirtIO block driver for QEMU.

use core::arch::asm;
use core::ptr::{addr_of, addr_of_mut};
use core::sync::atomic::{fence, Ordering};

const VIRTIO_VENDOR: u16 = 0x1af4;
const VIRTIO_BLOCK_LEGACY: u16 = 0x1001;
const QUEUE_SIZE_MAX: u16 = 128;
const QUEUE_USED_OFFSET: usize = 4096;

#[repr(C, align(4096))]
struct QueueMemory([u8; 8192]);

#[repr(C)]
struct RequestHeader {
    request_type: u32,
    reserved: u32,
    sector: u64,
}

#[repr(C, align(512))]
struct SectorBuffer([u8; 512]);

static mut QUEUE: QueueMemory = QueueMemory([0; 8192]);
static mut HEADER: RequestHeader = RequestHeader { request_type: 0, reserved: 0, sector: 0 };
static mut BUFFER: SectorBuffer = SectorBuffer([0; 512]);
static mut STATUS: u8 = 0xff;
static mut IO_BASE: u16 = 0;
static mut QUEUE_SIZE: u16 = 0;
static mut AVAILABLE_INDEX: u16 = 0;
static mut LAST_USED_INDEX: u16 = 0;
static mut READY: bool = false;

pub fn init() -> bool {
    let Some((bus, device, function)) = find_device() else { return false };
    let address = pci_read(bus, device, function, 0x10);
    if address & 1 == 0 { return false; }
    let io_base = (address & 0xfffc) as u16;

    let command = pci_read(bus, device, function, 0x04);
    pci_write(bus, device, function, 0x04, command | 0x0000_0005);

    unsafe {
        IO_BASE = io_base;
        outb(io_base + 18, 0);
        outb(io_base + 18, 1);
        outb(io_base + 18, 3);
        let _device_features = inl(io_base);
        outl(io_base + 4, 0);
        outw(io_base + 14, 0);
        let queue_size = inw(io_base + 12);
        if queue_size < 3 || queue_size > QUEUE_SIZE_MAX { return false; }
        QUEUE_SIZE = queue_size;
        core::ptr::write_bytes(addr_of_mut!(QUEUE).cast::<u8>(), 0, 8192);
        let queue_physical = addr_of!(QUEUE) as u64;
        if queue_physical & 0xfff != 0 || queue_physical > u32::MAX as u64 * 4096 { return false; }
        outl(io_base + 8, (queue_physical >> 12) as u32);
        outb(io_base + 18, 7);
        READY = inb(io_base + 18) & 0x80 == 0;
        READY
    }
}

pub fn is_ready() -> bool { unsafe { READY } }

pub fn read_sector(sector: u64, output: &mut [u8; 512]) -> bool {
    unsafe {
        if !READY { return false; }
        HEADER = RequestHeader { request_type: 0, reserved: 0, sector };
        STATUS = 0xff;

        write_descriptor(0, addr_of!(HEADER) as u64, 16, 1, 1);
        write_descriptor(1, addr_of!(BUFFER) as u64, 512, 3, 2);
        write_descriptor(2, addr_of!(STATUS) as u64, 1, 2, 0);

        let queue = addr_of_mut!(QUEUE).cast::<u8>();
        let ring_slot = AVAILABLE_INDEX % QUEUE_SIZE;
        write_u16(queue.add(4 + ring_slot as usize * 2), 0);
        fence(Ordering::Release);
        AVAILABLE_INDEX = AVAILABLE_INDEX.wrapping_add(1);
        write_u16(queue.add(2), AVAILABLE_INDEX);
        outw(IO_BASE + 16, 0);

        let mut spins = 0usize;
        while read_u16(queue.add(QUEUE_USED_OFFSET + 2)) == LAST_USED_INDEX {
            core::hint::spin_loop();
            spins += 1;
            if spins == 50_000_000 { return false; }
        }
        LAST_USED_INDEX = LAST_USED_INDEX.wrapping_add(1);
        fence(Ordering::Acquire);
        if STATUS != 0 { return false; }
        output.copy_from_slice(&BUFFER.0);
        true
    }
}

unsafe fn write_descriptor(index: usize, address: u64, length: u32, flags: u16, next: u16) {
    let pointer = addr_of_mut!(QUEUE).cast::<u8>().add(index * 16);
    core::ptr::write_unaligned(pointer.cast::<u64>(), address);
    core::ptr::write_unaligned(pointer.add(8).cast::<u32>(), length);
    core::ptr::write_unaligned(pointer.add(12).cast::<u16>(), flags);
    core::ptr::write_unaligned(pointer.add(14).cast::<u16>(), next);
}

unsafe fn write_u16(pointer: *mut u8, value: u16) { core::ptr::write_volatile(pointer.cast::<u16>(), value); }
unsafe fn read_u16(pointer: *mut u8) -> u16 { core::ptr::read_volatile(pointer.cast::<u16>()) }

fn find_device() -> Option<(u8, u8, u8)> {
    for bus in 0u16..=255 {
        for device in 0u8..32 {
            for function in 0u8..8 {
                let value = pci_read(bus as u8, device, function, 0);
                if value as u16 == VIRTIO_VENDOR && (value >> 16) as u16 == VIRTIO_BLOCK_LEGACY {
                    return Some((bus as u8, device, function));
                }
            }
        }
    }
    None
}

fn pci_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = 0x8000_0000u32 | ((bus as u32) << 16) | ((device as u32) << 11)
        | ((function as u32) << 8) | (offset as u32 & 0xfc);
    unsafe { outl(0xcf8, address); inl(0xcfc) }
}

fn pci_write(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = 0x8000_0000u32 | ((bus as u32) << 16) | ((device as u32) << 11)
        | ((function as u32) << 8) | (offset as u32 & 0xfc);
    unsafe { outl(0xcf8, address); outl(0xcfc, value); }
}

unsafe fn inb(port: u16) -> u8 { let value: u8; asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack)); value }
unsafe fn inw(port: u16) -> u16 { let value: u16; asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack)); value }
unsafe fn inl(port: u16) -> u32 { let value: u32; asm!("in eax, dx", in("dx") port, out("eax") value, options(nomem, nostack)); value }
unsafe fn outb(port: u16, value: u8) { asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack)); }
unsafe fn outw(port: u16, value: u16) { asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack)); }
unsafe fn outl(port: u16, value: u32) { asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack)); }
