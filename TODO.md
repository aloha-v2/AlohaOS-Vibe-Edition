# AlohaOS Roadmap

## M0 Kernel Stable

- [x] Context switching, scheduler, synchronization, memory reclamation, diagnostics и QEMU tests.

## M1 Userland

### Process memory и Ring 3

- [x] Ring 3 descriptors, TSS `RSP0`, per-process PML4 и USER/NX W^X mappings.
- [x] CR3 guard, Process lifecycle, validated user copies и Ring 3 round-trip.

### Syscall ABI и entry

- [x] Versioned numbers, errno и dispatcher для `write/exit/sleep`.
- [x] Per-process 32 KiB kernel entry stack с ownership cleanup.
- [x] Canonical user RIP/RSP validation.
- [x] RFLAGS sanitization и классификация `sysret` / `iret` / reject.
- [x] CPUID проверка поддержки SYSCALL/SYSRET.
- [ ] Настроить EFER.SCE, STAR, LSTAR и FMASK.
- [ ] Assembly entry сохраняет user RSP/RCX/R11 и callee-saved registers.
- [ ] Переключать на kernel entry stack до вызова Rust dispatcher.
- [ ] Реальный QEMU syscall smoke для `write/exit/sleep`.
- [ ] Затем `read/open/close/stat/mmap/spawn/wait` и handles.

### ELF, runtime и isolation

- [ ] ELF64 validation, PT_LOAD W^X mappings, BSS zeroing и overlap rejection.
- [ ] Process table, parent/child, waiters и cleanup.
- [ ] User faults завершают процесс, не kernel.
- [ ] Rust user runtime, syscall wrappers, app Cargo build и user-space shell.

## M2 IPC, VM и storage

- [ ] Channels, pipes, shared memory/MAP_SHARED, Unix sockets и signals.
- [ ] Slab, VM areas, demand paging, file mappings и OOM policy.
- [ ] VFS, writable FAT32, page cache, permissions/capabilities и settings.

## M3-M5

- [ ] ACPI/APIC/SMP, networking/security, graphics/desktop, packages/tooling/docs.

## Следующий пакет

1. EFER/STAR/LSTAR/FMASK + assembly entry on per-process stack.
2. Подключить dispatcher и QEMU smoke `write/exit/sleep`.
3. ELF loader + process table + fault isolation.
4. Channels + shared memory.
