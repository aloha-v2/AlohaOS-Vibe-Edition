//! Long-mode GDT and TSS, including a dedicated double-fault IST stack.

use core::arch::asm;
use core::mem::size_of;
use core::ptr::addr_of;

const KERNEL_CODE_SELECTOR: u16 = 0x08;
const DOUBLE_FAULT_IST_INDEX: u8 = 1;

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
struct InterruptStack([u8; 20 * 1024]);

#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

#[repr(C, align(16))]
struct AlignedGdt([u64; 5]);

static mut DOUBLE_FAULT_STACK: InterruptStack = InterruptStack([0; 20 * 1024]);
static mut TSS: TaskStateSegment = TaskStateSegment::ZERO;
static mut GDT: AlignedGdt = AlignedGdt([0; 5]);

pub fn init() {
    unsafe {
        let stack_top = addr_of!(DOUBLE_FAULT_STACK) as u64
            + size_of::<InterruptStack>() as u64;
        let mut ist = [0u64; 7];
        ist[(DOUBLE_FAULT_IST_INDEX - 1) as usize] = stack_top;
        TSS = TaskStateSegment {
            interrupt_stack_table: ist,
            ..TaskStateSegment::ZERO
        };

        GDT.0[0] = 0;
        GDT.0[1] = 0x00af_9a00_0000_ffff;
        GDT.0[2] = 0x00cf_9200_0000_ffff;

        let base = addr_of!(TSS) as u64;
        let limit = (size_of::<TaskStateSegment>() - 1) as u64;
        GDT.0[3] = (limit & 0xffff)
            | ((base & 0x00ff_ffff) << 16)
            | (0x89u64 << 40)
            | (((limit >> 16) & 0x0f) << 48)
            | (((base >> 24) & 0xff) << 56);
        GDT.0[4] = base >> 32;

        let pointer = GdtPointer {
            limit: (size_of::<AlignedGdt>() - 1) as u16,
            base: addr_of!(GDT) as u64,
        };
        asm!("lgdt [{}]", in(reg) &pointer, options(readonly, nostack));

        // Use immediate selectors. Passing the code selector through an
        // arbitrary register made the far-return sequence fragile under LTO.
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
            "mov ax, 0x18",
            "ltr ax",
            out("rax") _,
        );
    }
}

pub const fn code_selector() -> u16 {
    KERNEL_CODE_SELECTOR
}

pub const fn double_fault_ist() -> u8 {
    DOUBLE_FAULT_IST_INDEX
}
