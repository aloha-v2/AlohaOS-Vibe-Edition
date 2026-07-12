# AlohaOS Roadmap

Цель: стабильная x86_64 ОС с изолированным user space, VFS, IPC, networking и desktop applications.

## M0 Kernel Stable

- [x] Context switching, scheduler, synchronization, memory reclamation, panic diagnostics и smoke tests.

## M1 Userland

### Process memory и Ring 3

- [x] Ring 3 descriptors, TSS `RSP0`, per-process PML4 и USER/NX W^X mappings.
- [x] CR3 guard, Process lifecycle и validated multi-page user copies.
- [x] Bootstrap image, guarded user stack, `iretq`, DPL3 trap и Ring 3 QEMU smoke.

### Syscall ABI

- [x] Versioned syscall numbers, negative errno encoding и единый Rust dispatcher.
- [x] `write`, `exit`, `sleep` semantics с pointer/length validation и bounded copies.
- [x] Dispatcher smoke tests: success, oversized write, sleep lifecycle и exit code.
- [ ] Настроить EFER.SCE, STAR, LSTAR и FMASK.
- [ ] Per-process kernel entry stack вместо bootstrap global return slot.
- [ ] Assembly entry сохраняет user RSP/RCX/R11 и callee-saved registers.
- [ ] Canonical RIP/RSP checks и fallback на `iretq` вместо опасного `sysretq`.
- [ ] Подключить dispatcher к реальному `syscall` instruction и добавить QEMU syscall smoke.
- [ ] Затем `read/open/close/stat/mmap/spawn/wait` и handles.

### ELF, runtime и isolation

- [ ] ELF64 validation, PT_LOAD W^X mappings, BSS zeroing и overlap rejection.
- [ ] Process table, parent/child, waiters и cleanup.
- [ ] User faults завершают процесс, не kernel.
- [ ] Negative CI: bad pointer, NX execution, write-to-code, stack overflow, malformed ELF.
- [ ] Rust user runtime, syscall wrappers, app Cargo build и user-space shell.

## M2 IPC, VM и storage

- [ ] Channels/message queues, pipes, shared memory + `MAP_SHARED`, затем Unix sockets и signals.
- [ ] Slab allocator, VM areas, mmap/mprotect, demand paging, file mappings и OOM policy.
- [ ] VFS, writable FAT32, page cache, `/tmp`, permissions/capabilities и persistent settings.

## M3 Hardware, network и security

- [ ] ACPI/APIC/HPET/SMP/PCIe, power management и device manager.
- [ ] VirtIO-net, ARP/IPv4/ICMP, UDP/TCP, sockets, DHCP, DNS и HTTP.
- [ ] SMEP/SMAP, canaries, ASLR/KASLR, Secure Boot, watchdog и recovery.

## M4-M5 Graphics, desktop и distribution

- [ ] VirtIO GPU, input events, compositor, toolkit, desktop shell, apps и audio.
- [ ] libc/runtime/toolchain, package manager, signed repositories и updates.
- [ ] GDB stub, tracing, core dumps, profiler, fuzzing, regression CI и docs.

## Следующий пакет

1. SYSCALL MSRs + hardened assembly entry/return + per-process kernel stack.
2. Реальный syscall QEMU smoke для `write/exit/sleep`.
3. ELF loader + process table + fault isolation.
4. Channels + shared memory как первый IPC слой.
