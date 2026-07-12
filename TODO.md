# AlohaOS Roadmap

## M0 Kernel Stable

- [x] Scheduler, memory, synchronization, diagnostics и QEMU regression suite.

## M1 Userland

### Ring 3, syscall и ELF

- [x] Per-process memory, W^X, safe copies, kernel stacks и Ring 3 entry.
- [x] Real LSTAR/SYSRET `write`/`exit` path.
- [x] ELF validation, loading, BSS zeroing и execution.

### Process lifecycle syscalls

- [x] Bounded registry, PID allocation, parent/child и orphan reparenting.
- [x] Register existing Process metadata.
- [x] `getpid` syscall.
- [x] `sleep` синхронизирует Process и registry state.
- [x] `exit` сохраняет status, reparent-ит children и завершает process runner.
- [x] `wait` проверяет parent, возвращает Busy для live child и reaps exited child.
- [ ] Blocking wait queue и wake parent при child exit.
- [ ] Scheduler-backed sleep deadline и automatic resume.
- [ ] `spawn` ownership: ELF image, Process object, registry slot и rollback при ошибке.
- [ ] Handle table и `read/open/close/stat/mmap`.

### Isolation и runtime

- [ ] User page fault/invalid opcode завершают process, не kernel.
- [ ] Negative CI: bad pointer, NX execute, write-to-code, stack guard.
- [ ] Rust user runtime, app Cargo pipeline и user-space shell.

## M2 IPC, VM и storage

- [ ] Channels, pipes, shared memory/MAP_SHARED, Unix sockets и signals.
- [ ] Slab, demand paging, file mappings и OOM policy.
- [ ] VFS, writable FAT32, page cache, permissions/capabilities и settings.

## Следующий пакет

1. Blocking wait + scheduler-backed sleep/wakeup.
2. User fault isolation + negative QEMU tests.
3. Spawn ownership + handles.
4. Channels + shared memory.
