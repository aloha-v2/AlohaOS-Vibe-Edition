# AlohaOS Roadmap

## M0 Kernel Stable

- [x] Scheduler, memory, synchronization, diagnostics и QEMU regression suite.

## M1 Userland

### Ring 3 и syscalls

- [x] Ring 3, per-process PML4, USER/NX W^X, CR3 guard и safe user copies.
- [x] Process-owned kernel stacks, LSTAR entry, dispatcher и validated SYSRET.
- [x] Real user `write` и `exit` syscall QEMU smoke.
- [ ] IRET fallback и scheduler-backed `sleep` resume.
- [ ] `read/open/close/stat/mmap/spawn/wait` и handle table.

### ELF loader

- [x] ELF64 magic/class/endian/type/machine validation.
- [x] ELF/program-header bounds и integer-overflow checks.
- [x] Bounded PT_LOAD plan, user-region enforcement и entry-in-executable validation.
- [x] W^X rejection, segment overlap rejection и BSS size planning.
- [x] Valid/malformed/W+X smoke tests.
- [ ] Мапить все страницы PT_LOAD с итоговыми permissions.
- [ ] Копировать file bytes через physical ownership и zero-fill BSS.
- [ ] Поддержать page-aligned overlap внутри одного compatible segment layout.
- [ ] Запустить Process с ELF entry и guarded stack.

### Process table и isolation

- [ ] PID allocator, parent/child, waiters, handles и cleanup.
- [ ] User page fault/invalid opcode завершают процесс, не kernel.
- [ ] Negative CI: bad pointer, NX execute, write-to-code, stack overflow, malformed ELF.
- [ ] Rust user runtime, wrappers, app build и user-space shell.

## M2 IPC, VM и storage

- [ ] Channels, pipes, shared memory/MAP_SHARED, Unix sockets и signals.
- [ ] Slab, VM areas, demand paging, file mappings и OOM policy.
- [ ] VFS, writable FAT32, page cache, permissions/capabilities и settings.

## M3-M5

- [ ] ACPI/APIC/SMP, networking/security, graphics/desktop, packages/tooling/docs.

## Следующий пакет

1. PT_LOAD page mapping + file copy + BSS zeroing.
2. ELF entry execution smoke.
3. Process table + wait/cleanup + fault isolation.
4. Channels + shared memory.
