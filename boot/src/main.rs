//! AlohaBoot, the AlohaOS UEFI bootloader.
#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use core::{mem, ptr};
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat as UefiPixelFormat};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{AllocateType, MemoryAttribute, MemoryType};

use common::{
    BootInfo, FrameBufferInfo, MemoryMapInfo, MemoryRegion, MemoryRegionKind,
    PixelFormat, MAX_MEMORY_REGIONS,
};
use xmas_elf::program::Type as ProgramType;
use xmas_elf::ElfFile;

static mut BOOT_INFO: BootInfo = BootInfo::EMPTY;
static mut MEMORY_REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] =
    [MemoryRegion::EMPTY; MAX_MEMORY_REGIONS];

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();
    let bs = system_table.boot_services();

    let (fb_addr, fb_size, width, height, stride, uefi_pf) = {
        let handle = bs.get_handle_for_protocol::<GraphicsOutput>().unwrap();
        let mut gop = bs.open_protocol_exclusive::<GraphicsOutput>(handle).unwrap();
        let mode = gop.current_mode_info();
        let (width, height) = mode.resolution();
        let stride = mode.stride();
        let format = mode.pixel_format();
        let mut framebuffer = gop.frame_buffer();
        (
            framebuffer.as_mut_ptr() as u64,
            framebuffer.size(),
            width,
            height,
            stride,
            format,
        )
    };

    let pixel_format = match uefi_pf {
        UefiPixelFormat::Rgb => PixelFormat::Rgb,
        UefiPixelFormat::Bgr => PixelFormat::Bgr,
        _ => PixelFormat::Unknown,
    };

    let kernel_bytes = {
        let loaded_image = bs.open_protocol_exclusive::<LoadedImage>(image_handle).unwrap();
        let device = loaded_image.device().unwrap();
        let mut sfs = bs.open_protocol_exclusive::<SimpleFileSystem>(device).unwrap();
        let mut root = sfs.open_volume().unwrap();
        let handle = root
            .open(
                uefi::cstr16!("alohaos\\kernel.elf"),
                FileMode::Read,
                FileAttribute::empty(),
            )
            .unwrap();
        let mut file = match handle.into_type().unwrap() {
            FileType::Regular(file) => file,
            FileType::Dir(_) => panic!("kernel.elf is a directory"),
        };
        let size = file.get_boxed_info::<FileInfo>().unwrap().file_size() as usize;
        let mut bytes = vec![0u8; size];
        let read = file.read(&mut bytes).unwrap();
        bytes.truncate(read);
        bytes
    };

    let elf = ElfFile::new(&kernel_bytes).unwrap();
    let entry_point = elf.header.pt2.entry_point();
    for header in elf.program_iter() {
        if header.get_type() != Ok(ProgramType::Load) || header.mem_size() == 0 {
            continue;
        }
        let physical = header.physical_addr();
        let memory_size = header.mem_size() as usize;
        let file_size = header.file_size() as usize;
        let offset = header.offset() as usize;
        let pages = (memory_size + 0xfff) / 0x1000;
        bs.allocate_pages(AllocateType::Address(physical), MemoryType::LOADER_DATA, pages)
            .unwrap();
        unsafe {
            let destination = physical as *mut u8;
            ptr::copy(kernel_bytes.as_ptr().add(offset), destination, file_size);
            ptr::write_bytes(destination.add(file_size), 0, memory_size - file_size);
        }
    }

    // exit_boot_services returns the final map. Copy it into a stable, simple
    // boot protocol owned by AlohaBoot so the kernel does not depend on UEFI.
    let (_runtime, memory_map) = system_table.exit_boot_services(MemoryType::LOADER_DATA);
    let mut region_count = 0usize;
    for descriptor in memory_map.entries().take(MAX_MEMORY_REGIONS) {
        let kind = if descriptor.ty == MemoryType::CONVENTIONAL {
            MemoryRegionKind::Usable
        } else if descriptor.ty == MemoryType::ACPI_RECLAIM {
            MemoryRegionKind::AcpiReclaimable
        } else if descriptor.ty == MemoryType::ACPI_NON_VOLATILE {
            MemoryRegionKind::AcpiNvs
        } else if descriptor.ty == MemoryType::MMIO
            || descriptor.ty == MemoryType::MMIO_PORT_SPACE
        {
            MemoryRegionKind::Mmio
        } else if descriptor.att.contains(MemoryAttribute::RUNTIME) {
            MemoryRegionKind::Runtime
        } else {
            MemoryRegionKind::Reserved
        };

        unsafe {
            MEMORY_REGIONS[region_count] = MemoryRegion {
                physical_start: descriptor.phys_start,
                page_count: descriptor.page_count,
                kind,
            };
        }
        region_count += 1;
    }

    unsafe {
        BOOT_INFO = BootInfo {
            framebuffer: FrameBufferInfo {
                addr: fb_addr,
                size: fb_size,
                width,
                height,
                stride,
                pixel_format,
            },
            memory_map: MemoryMapInfo {
                regions: ptr::addr_of!(MEMORY_REGIONS).cast::<MemoryRegion>(),
                region_count,
            },
        };
    }

    let entry: extern "sysv64" fn(*const BootInfo) -> ! =
        unsafe { mem::transmute(entry_point) };
    entry(ptr::addr_of!(BOOT_INFO));
}
