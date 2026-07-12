//! Long-mode GDT and TSS with Ring 0/Ring 3 segments and dedicated stacks.
//!
//! Descriptor storage is written exactly once during single-core boot, before
//! interrupts are enabled, then remains pinned for the lifetime of the kernel.

use core::arch::asm;
use core::cell::UnsafeCell;
use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut};
use core::sync::atomic::{AtomicBool, Ordering};

const KERNEL_CODE_SELECTOR: u16 = 0x08;
const KERNEL_DATA_SELECTOR: u16 = 0x10;
const USER_DATA_SELECTOR: u16 = 0x18 | 3;
const USER_CODE_SELECTOR: u16 = 0x20 | 3;
const TSS_SELECTOR: u16 = 0x28;
const DOUBLE_FAULT_IST_INDEX: u8 = 1;
const SCHEDULER_IST_INDEX: u8 = 2;
const INTERRUPT_STACK_SIZE: usize = 20 * 1024;
const RSP0_STACK_SIZE: usize = 32 * 1024;

#[repr(C, packed)]
struct TaskStateSegment {
    reserved_1: u32,
    privilege_stack_table: [u64; 3],
    reserved_2: u64,
    interrupt_stack_table: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    iomap_base: u16,
}

impl TaskStateSegment {
    const ZERO: Self = Self {
        reserved_1: 0,
        privilege_stack_table: [0; 3],
        reserved_2: 0,
        interrupt_stack_table: [0; 7],
        reserved_3: 0,
        reserved_4: 0,
        iomap_base: size_of::<Self>() as u16,
    };
}

#[repr(C, align(16))]
struct InterruptStack([u8; INTERRUPT_STACK_SIZE]);

#[repr(C, align(16))]
struct PrivilegeStack([u8; RSP0_STACK_SIZE]);

#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

#[repr(C, align(16))]
struct AlignedGdt([u64; 7]);

struct DescriptorStorage {
    double_fault_stack: UnsafeCell<InterruptStack>,
    scheduler_stack: UnsafeCell<InterruptStack>,
    rsp0_stack: UnsafeCell<PrivilegeStack>,
    tss: UnsafeCell<TaskStateSegment>,
    gdt: UnsafeCell<AlignedGdt>,
}

unsafe impl Sync for DescriptorStorage {}

static STORAGE: DescriptorStorage = DescriptorStorage {
    double_fault_stack: UnsafeCell::new(InterruptStack([0; INTERRUPT_STACK_SIZE])),
    scheduler_stack: UnsafeCell::new(InterruptStack([0; INTERRUPT_STACK_SIZE])),
    rsp0_stack: UnsafeCell::new(PrivilegeStack([0; RSP0_STACK_SIZE])),
    tss: UnsafeCell::new(TaskStateSegment::ZERO),
    gdt: UnsafeCell::new(AlignedGdt([0; 7])),
};
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init() {
    if INITIALIZED.swap(true, Ordering::AcqRel) {
        return;
    }

    unsafe {
        let mut privilege_stacks = [0u64; 3];
        privilege_stacks[0] = addr_of!(*STORAGE.rsp0_stack.get()) as u64
            + size_of::<PrivilegeStack>() as u64;

        let mut interrupt_stacks = [0u64; 7];
        interrupt_stacks[(DOUBLE_FAULT_IST_INDEX - 1) as usize] =
            addr_of!(*STORAGE.double_fault_stack.get()) as u64
                + size_of::<InterruptStack>() as u64;
        interrupt_stacks[(SCHEDULER_IST_INDEX - 1) as usize] =
            addr_of!(*STORAGE.scheduler_stack.get()) as u64
                + size_of::<InterruptStack>() as u64;

        addr_of_mut!(*STORAGE.tss.get()).write(TaskStateSegment {
            privilege_stack_table: privilege_stacks,
            interrupt_stack_table: interrupt_stacks,
            ..TaskStateSegment::ZERO
        });

        let tss_base = STORAGE.tss.get() as u64;
        let tss_limit = (size_of::<TaskStateSegment>() - 1) as u64;
        let gdt = &mut *STORAGE.gdt.get();
        gdt.0[0] = 0;
        gdt.0[1] = 0x00af_9a00_0000_ffff; // Ring 0 code
        gdt.0[2] = 0x00cf_9200_0000_ffff; // Ring 0 data
        gdt.0[3] = 0x00cf_f200_0000_ffff; // Ring 3 data
        gdt.0[4] = 0x00af_fa00_0000_ffff; // Ring 3 code
        gdt.0[5] = (tss_limit & 0xffff)
            | ((tss_base & 0x00ff_ffff) << 16)
            | (0x89u64 << 40)
            | (((tss_limit >> 16) & 0x0f) << 48)
            | (((tss_base >> 24) & 0xff) << 56);
        gdt.0[6] = tss_base >> 32;

        let pointer = GdtPointer {
            limit: (size_of::<AlignedGdt>() - 1) as u16,
            base: STORAGE.gdt.get() as u64,
        };
        asm!("lgdt [{}]", in(reg) &pointer, options(readonly, nostack));
        asm!(
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "xor eax, eax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ax, 0x28",
            "ltr ax",
            out("rax") _,
        );
    }
}

pub const fn code_selector() -> u16 {
    KERNEL_CODE_SELECTOR
}

pub const fn kernel_data_selector() -> u16 {
    KERNEL_DATA_SELECTOR
}

pub const fn user_code_selector() -> u16 {
    USER_CODE_SELECTOR
}

pub const fn user_data_selector() -> u16 {
    USER_DATA_SELECTOR
}

pub fn rsp0() -> u64 {
    unsafe { addr_of!((*STORAGE.tss.get()).privilege_stack_table[0]).read_unaligned() }
}

pub const fn double_fault_ist() -> u8 {
    DOUBLE_FAULT_IST_INDEX
}

pub const fn scheduler_ist() -> u8 {
    SCHEDULER_IST_INDEX
}
