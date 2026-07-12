# AlohaOS Roadmap

## M0 Kernel Stable
- [x] Scheduler, memory, synchronization, diagnostics и QEMU suite.

## M1 Userland

### Выполнено
- [x] Ring 3, per-process memory, real syscalls и ELF execution.
- [x] Process registry, PID/parent/child, wait/reap и orphan handling.
- [x] Sleep deadlines и blocking wait wake events.
- [x] PIT вызывает process deadline advancement.
- [x] Runner reconciliation после sleep/wait wake.
- [x] Blocking wait completion возвращает child status и reaps запись.

### Следующий слой
- [ ] Syscall `sleep` должен suspend текущий user frame и resume с того же RIP.
- [ ] Syscall `wait` должен resume и вернуть status в RAX автоматически.
- [ ] User page fault/invalid opcode завершают process, не kernel.
- [ ] Negative QEMU: NX execute, write-to-code, stack guard, bad pointer.
- [ ] Spawn ownership: ELF + Process + registry + rollback.
- [ ] Handle table и `read/open/close/stat/mmap`.
- [ ] Rust runtime и user-space shell.

## M2+
- [ ] IPC/shared memory, VM/VFS, networking/security, graphics/desktop, packages/tooling.

## Следующий пакет
1. User fault isolation + negative QEMU tests.
2. Suspended syscall frame resume для sleep/wait.
3. Spawn ownership + handles.
4. Channels + shared memory.
