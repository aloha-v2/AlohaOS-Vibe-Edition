//! AlohaBoot — the AlohaOS UEFI bootloader.
//!
//! Responsibilities:
//!   1. Grab the linear framebuffer from the Graphics Output Protocol.
//!   2. Read `\alohaos\kernel.elf` from the boot volume.
//!   3. Load its PT_LOAD segments at their physical addresses.
//!   4. Exit boot services and jump into the kernel (sysv64 ABI).
#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use core::mem;
use core::ptr;

use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat as UefiPixelFormat};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{AllocateType, MemoryType};

use common::{BootInfo, FrameBufferInfo, PixelFormat};
use xmas_elf::program::Type as ProgramType;
use xmas_elf::ElfFile;

/// Lives in the bootloader image, which stays resident after ExitBootServices,
/// so the kernel can safely read it through the pointer we hand over.
static mut BOOT_INFO: BootInfo = BootInfo::EMPTY;

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();
    let bs = system_table.boot_services();

    // --- 1. Graphics: pull the linear framebuffer out of GOP ---
    let (fb_addr, fb_size, width, height, stride, uefi_pf) = {
        let gop_handle = bs.get_handle_for_protocol::<GraphicsOutput>().unwrap();
        let mut gop = bs
            .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
            .unwrap();
        let mode_info = gop.current_mode_info();
        let (w, h) = mode_info.resolution();
        let stride = mode_info.stride();
        let pf = mode_info.pixel_format();
        let mut fb = gop.frame_buffer();
        (fb.as_mut_ptr() as u64, fb.size(), w, h, stride, pf)
    };

    let pixel_format = match uefi_pf {
        UefiPixelFormat::Rgb => PixelFormat::Rgb,
        UefiPixelFormat::Bgr => PixelFormat::Bgr,
        _ => PixelFormat::Unknown,
    };

    // --- 2. Read the kernel ELF from the boot volume ---
    let kernel_bytes = {
        let loaded_image = bs
            .open_protocol_exclusive::<LoadedImage>(image_handle)
            .unwrap();
        let device = loaded_image.device().unwrap();
        let mut sfs = bs
            .open_protocol_exclusive::<SimpleFileSystem>(device)
            .unwrap();
        let mut root = sfs.open_volume().unwrap();

        let handle = root
            .open(
                uefi::cstr16!("alohaos\\kernel.elf"),
                FileMode::Read,
                FileAttribute::empty(),
            )
            .unwrap();
        let mut file = match handle.into_type().unwrap() {
            FileType::Regular(f) => f,
            FileType::Dir(_) => panic!("alohaos\\kernel.elf is a directory"),
        };

        let info = file.get_boxed_info::<FileInfo>().unwrap();
        let size = info.file_size() as usize;
        let mut buf = vec![0u8; size];
        let read = file.read(&mut buf).unwrap();
        buf.truncate(read);
        buf
    };

    // --- 3. Load ELF LOAD segments at their physical addresses ---
    let elf = ElfFile::new(&kernel_bytes).unwrap();
    let entry_point = elf.header.pt2.entry_point();

    for ph in elf.program_iter() {
        if ph.get_type() != Ok(ProgramType::Load) {
            continue;
        }
        let phys = ph.physical_addr();
        let mem_size = ph.mem_size() as usize;
        let file_size = ph.file_size() as usize;
        let offset = ph.offset() as usize;

        if mem_size == 0 {
            continue;
        }

        let pages = (mem_size + 0xfff) / 0x1000;
        bs.allocate_pages(AllocateType::Address(phys), MemoryType::LOADER_DATA, pages)
            .unwrap();

        unsafe {
            let dest = phys as *mut u8;
            ptr::copy(kernel_bytes.as_ptr().add(offset), dest, file_size);
            if mem_size > file_size {
                // Zero the .bss tail.
                ptr::write_bytes(dest.add(file_size), 0, mem_size - file_size);
            }
        }
    }

    // --- 4. Publish boot info for the kernel ---
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
        };
    }
    let boot_info_ptr = ptr::addr_of!(BOOT_INFO);

    // --- 5. Leave the firmware behind and jump into the kernel ---
    let _ = unsafe { system_table.exit_boot_services(MemoryType::LOADER_DATA) };

    let kernel_entry: extern "sysv64" fn(*const BootInfo) -> ! =
        unsafe { mem::transmute(entry_point) };
    kernel_entry(boot_info_ptr);
}
