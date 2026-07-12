# AlohaOS Roadmap

## M0 Kernel Stable
- [x] Scheduler, memory, synchronization, diagnostics и QEMU suite.

## M1 Userland

### Выполнено
- [x] Ring 3, per-process memory, W^X, safe copies и kernel entry stacks.
- [x] Real syscalls, ELF loading/execution и process registry.
- [x] PID/parent/child, exit status, wait/reap и orphan reparenting.
- [x] Sleep deadlines и time-driven Ready transition.
- [x] Blocking wait registration и wake parent on child exit/fault.
- [x] One-shot wake events и smoke coverage.

### Следующий слой
- [ ] Вызывать `process_table::advance_time` из timer tick.
- [ ] Process runner должен resume Ready process после sleep/wait wake.
- [ ] Blocking `wait` syscall должен возвращать status после resumption, не Busy.
- [ ] IRET fallback для non-SYSRET-safe return frames.
- [ ] User page fault/invalid opcode завершают process, не kernel.
- [ ] Spawn ownership: ELF + Process + registry slot + rollback.
- [ ] Handle table и `read/open/close/stat/mmap`.
- [ ] Rust runtime и user-space shell.

## M2+
- [ ] IPC channels/shared memory, VM/VFS, hardware/network/security, graphics/desktop, packages/tooling.

## Следующий пакет
1. Timer integration + process runner resume.
2. User fault isolation + negative QEMU tests.
3. Spawn ownership + handles.
4. Channels + shared memory.
