# AlohaOS Roadmap

## M0 Kernel Stable

- [x] Scheduler, memory, synchronization, diagnostics и QEMU regression suite.

## M1 Userland

### Process memory и Ring 3

- [x] Ring 3 descriptors, TSS `RSP0`, per-process PML4, USER/NX W^X и safe user copies.
- [x] Process lifecycle, CR3 guard, owned kernel entry stacks и Ring 3 round-trip.

### Syscall ABI и architecture

- [x] Versioned numbers, errno и dispatcher для `write/exit/sleep`.
- [x] Canonical RIP/RSP checks, RFLAGS sanitization и return-path classification.
- [x] CPUID SYSCALL capability check.
- [x] Настроить и readback-проверить EFER.SCE, STAR, LSTAR и FMASK.
- [x] Формализовать `SyscallFrame`: number, 6 args, user RIP/RFLAGS/RSP, result.
- [x] Регистрация active per-process kernel entry stack.
- [x] Fail-closed LSTAR stub до готовности полного trampoline.
- [ ] Assembly entry сохраняет user RSP/RCX/R11 и callee-saved registers.
- [ ] Переключить RSP на active process kernel stack до вызова Rust.
- [ ] Вызвать dispatcher через `SyscallFrame` и обработать terminated/sleeping state.
- [ ] Проверенный SYSRET fast path и IRET fallback.
- [ ] QEMU smoke с настоящей инструкцией `syscall` для `write/exit/sleep`.
- [ ] Затем `read/open/close/stat/mmap/spawn/wait` и handles.

### ELF, runtime и isolation

- [ ] ELF64 validation, PT_LOAD W^X, BSS zeroing и overlap rejection.
- [ ] Process table, parent/child, waiters и cleanup.
- [ ] User faults завершают процесс, не kernel.
- [ ] Rust user runtime, syscall wrappers, app build и user-space shell.

## M2 IPC, VM и storage

- [ ] Channels, pipes, shared memory/MAP_SHARED, Unix sockets и signals.
- [ ] Slab, VM areas, demand paging, file mappings и OOM policy.
- [ ] VFS, writable FAT32, page cache, permissions/capabilities и settings.

## M3-M5

- [ ] ACPI/APIC/SMP, networking/security, graphics/desktop, packages/tooling/docs.

## Следующий пакет

1. Full assembly LSTAR trampoline на process kernel stack.
2. Dispatcher bridge + SYSRET/IRET return.
3. Real-syscall QEMU smoke `write/exit/sleep`.
4. ELF loader + process table + fault isolation.
