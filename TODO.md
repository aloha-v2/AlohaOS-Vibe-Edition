# AlohaOS Roadmap

## M0 Kernel Stable

- [x] Scheduler, memory, synchronization, diagnostics и QEMU regression suite.

## M1 Userland

### Ring 3, syscalls и ELF

- [x] Per-process memory, W^X, safe copies, kernel stacks и Ring 3 entry.
- [x] Real LSTAR/SYSRET user `write`/`exit` path.
- [x] ELF validation, PT_LOAD mapping, BSS zeroing и ELF execution smoke.
- [ ] IRET fallback и scheduler-backed `sleep`.
- [ ] `read/open/close/stat/mmap/spawn/wait` и handle table.

### Process table

- [x] Bounded process registry и monotonic nonzero PID allocation.
- [x] Parent/child metadata и lifecycle state.
- [x] Exit status, wait/reap и protection от wait живого процесса.
- [x] Reparent orphan children к kernel PID 0.
- [x] Process table smoke coverage.
- [ ] Связать registry с owned Process objects и syscall spawn/exit/wait.
- [ ] Blocking wait queue вместо polling `StillRunning`.

### Fault isolation

- [ ] Определять CPL по exception CS.
- [ ] User page fault/invalid opcode переводят только текущий Process в Faulted.
- [ ] Возврат на kernel process runner без panic kernel.
- [ ] Negative QEMU tests: NX execute, write-to-code, stack guard, bad pointer.

### Runtime и shell

- [ ] Rust user `_start`, panic strategy и syscall wrappers.
- [ ] Отдельный user app Cargo pipeline.
- [ ] Перенести shell в user space.

## M2 IPC, VM и storage

- [ ] Channels, pipes, shared memory/MAP_SHARED, Unix sockets и signals.
- [ ] Slab, VM areas, demand paging, file mappings и OOM policy.
- [ ] VFS, writable FAT32, page cache, permissions/capabilities и settings.

## Следующий пакет

1. Registry ownership + spawn/exit/wait syscalls.
2. User fault isolation + negative QEMU tests.
3. IRET fallback + scheduler-backed sleep.
4. Channels + shared memory.
