//! Shared ABI between AlohaBoot and the AlohaOS kernel.
#![no_std]

pub const MAX_MEMORY_REGIONS: usize = 256;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
    Unknown = 2,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FrameBufferInfo {
    pub addr: u64,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub pixel_format: PixelFormat,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryRegionKind {
    Reserved = 0,
    Usable = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    Mmio = 4,
    Runtime = 5,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MemoryRegion {
    pub physical_start: u64,
    pub page_count: u64,
    pub kind: MemoryRegionKind,
}

impl MemoryRegion {
    pub const EMPTY: Self = Self {
        physical_start: 0,
        page_count: 0,
        kind: MemoryRegionKind::Reserved,
    };
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MemoryMapInfo {
    pub regions: *const MemoryRegion,
    pub region_count: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct BootInfo {
    pub framebuffer: FrameBufferInfo,
    pub memory_map: MemoryMapInfo,
}

impl BootInfo {
    pub const EMPTY: Self = Self {
        framebuffer: FrameBufferInfo {
            addr: 0,
            size: 0,
            width: 0,
            height: 0,
            stride: 0,
            pixel_format: PixelFormat::Unknown,
        },
        memory_map: MemoryMapInfo {
            regions: core::ptr::null(),
            region_count: 0,
        },
    };
}
