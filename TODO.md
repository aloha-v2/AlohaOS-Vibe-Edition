# AlohaOS Roadmap

## M0 Kernel Stable

- [x] Scheduler, memory, synchronization, diagnostics и QEMU regression suite.

## M1 Userland

### Ring 3, syscall и ELF

- [x] Ring 3, per-process PML4, W^X, CR3 guard и safe user copies.
- [x] Process-owned kernel stacks, LSTAR entry, dispatcher и validated SYSRET.
- [x] Real user `write` и `exit` syscall QEMU smoke.
- [x] ELF64 validation, PT_LOAD mapping, file copy и BSS zeroing.
- [x] Загруженный ELF выполняется с `e_entry` и завершает процесс syscall `exit`.
- [x] Отдельный ELF execution QEMU CI job.
- [ ] IRET fallback и scheduler-backed `sleep` resume.
- [ ] `read/open/close/stat/mmap/spawn/wait` и handle table.

### Process table и isolation

- [ ] PID allocator и bounded process table.
- [ ] Parent/child relationship, exit status, waiters и deterministic cleanup.
- [ ] User page fault/invalid opcode завершают процесс, не kernel.
- [ ] Negative CI: bad pointer, NX execute, write-to-code, stack overflow, malformed ELF.

### Runtime и shell

- [ ] Rust user `_start`, panic strategy и syscall wrappers.
- [ ] Отдельный user app Cargo target/build pipeline.
- [ ] Перенести shell в user space.

## M2 IPC, VM и storage

- [ ] Channels, pipes, shared memory/MAP_SHARED, Unix sockets и signals.
- [ ] Slab, VM areas, demand paging, file mappings и OOM policy.
- [ ] VFS, writable FAT32, page cache, permissions/capabilities и settings.

## M3-M5

- [ ] ACPI/APIC/SMP, networking/security, graphics/desktop, packages/tooling/docs.

## Следующий пакет

1. Process table + wait/cleanup semantics.
2. User fault isolation + negative QEMU tests.
3. IRET fallback + scheduler-backed sleep.
4. Rust user runtime and shell migration.
