//! Extended x86_64 task context: address space, FS/GS bases and XSAVE state.

use core::arch::{asm, x86_64::__cpuid_count};
use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

const TASK_COUNT: usize = 2;
const XSTATE_CAPACITY: usize = 16 * 1024;
const IA32_FS_BASE: u32 = 0xc000_0100;
const IA32_GS_BASE: u32 = 0xc000_0101;

#[repr(C, align(64))]
struct Xstate([u8; XSTATE_CAPACITY]);

struct TaskContext {
 cr3: u64,
 fs_base: u64,
 gs_base: u64,
 xstate: Xstate,
}

impl TaskContext {
 const EMPTY: Self = Self {
  cr3: 0,
  fs_base: 0,
  gs_base: 0,
  xstate: Xstate([0; XSTATE_CAPACITY]),
 };
}

struct ContextSlots([UnsafeCell<TaskContext>; TASK_COUNT]);
unsafe impl Sync for ContextSlots {}

static CONTEXTS: ContextSlots = ContextSlots([
 UnsafeCell::new(TaskContext::EMPTY),
 UnsafeCell::new(TaskContext::EMPTY),
]);
static READY: AtomicBool = AtomicBool::new(false);
static XCR0_MASK: AtomicU64 = AtomicU64::new(0);

/// Enable XSAVE and clone the boot CPU state into both initial task slots.
pub fn init() -> bool {
 let basic = unsafe { __cpuid_count(1, 0) };
 if basic.ecx & (1 << 26) == 0 {
  return false;
 }

 unsafe {
  let mut cr4: u64;
  asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
  cr4 |= 1 << 9; // OSFXSR
  cr4 |= 1 << 10; // OSXMMEXCPT
  cr4 |= 1 << 18; // OSXSAVE
  asm!("mov cr4, {}", in(reg) cr4, options(nostack, preserves_flags));
 }

 let supported = unsafe { __cpuid_count(0x0d, 0) };
 let supported_mask = supported.eax as u64 | ((supported.edx as u64) << 32);
 let mut enabled = supported_mask & 0b11; // x87 and SSE are mandatory.
 if enabled != 0b11 {
  return false;
 }
 if basic.ecx & (1 << 28) != 0 && supported_mask & (1 << 2) != 0 {
  enabled |= 1 << 2; // AVX/YMM state.
 }
 unsafe { xsetbv(enabled) };

 let configured = unsafe { __cpuid_count(0x0d, 0) };
 if configured.ebx as usize > XSTATE_CAPACITY {
  return false;
 }
 XCR0_MASK.store(enabled, Ordering::Release);
 unsafe {
  save(0);
  ptr::copy_nonoverlapping(
   CONTEXTS.0[0].get(),
   CONTEXTS.0[1].get(),
   1,
  );
 }
 READY.store(true, Ordering::Release);
 true
}

pub fn is_ready() -> bool { READY.load(Ordering::Acquire) }
pub fn xcr0_mask() -> u64 { XCR0_MASK.load(Ordering::Acquire) }

/// Called with interrupts disabled from the timer path.
pub unsafe fn switch(current: usize, next: usize) {
 if current == next || !is_ready() { return; }
 save(current);
 restore(next);
}

unsafe fn save(task: usize) {
 let context = &mut *CONTEXTS.0[task].get();
 context.cr3 = read_cr3();
 context.fs_base = rdmsr(IA32_FS_BASE);
 context.gs_base = rdmsr(IA32_GS_BASE);
 let mask = XCR0_MASK.load(Ordering::Relaxed);
 asm!(
  "xsave64 [{}]",
  in(reg) context.xstate.0.as_mut_ptr(),
  in("eax") mask as u32,
  in("edx") (mask >> 32) as u32,
  options(nostack),
 );
}

unsafe fn restore(task: usize) {
 let context = &*CONTEXTS.0[task].get();
 if read_cr3() != context.cr3 {
  asm!("mov cr3, {}", in(reg) context.cr3, options(nostack, preserves_flags));
 }
 wrmsr(IA32_FS_BASE, context.fs_base);
 wrmsr(IA32_GS_BASE, context.gs_base);
 let mask = XCR0_MASK.load(Ordering::Relaxed);
 asm!(
  "xrstor64 [{}]",
  in(reg) context.xstate.0.as_ptr(),
  in("eax") mask as u32,
  in("edx") (mask >> 32) as u32,
  options(nostack),
 );
}

#[inline]
unsafe fn xsetbv(value: u64) {
 asm!("xsetbv", in("ecx") 0u32, in("eax") value as u32, in("edx") (value >> 32) as u32, options(nostack));
}

#[inline]
unsafe fn read_cr3() -> u64 {
 let value: u64;
 asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
 value
}

#[inline]
unsafe fn rdmsr(msr: u32) -> u64 {
 let low: u32;
 let high: u32;
 asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nostack));
 low as u64 | ((high as u64) << 32)
}

#[inline]
unsafe fn wrmsr(msr: u32, value: u64) {
 asm!("wrmsr", in("ecx") msr, in("eax") value as u32, in("edx") (value >> 32) as u32, options(nostack));
}
