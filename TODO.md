# AlohaOS Roadmap

## M0 Kernel Stable
- [x] Scheduler, memory, synchronization, diagnostics и QEMU suite.

## M1 Userland

### Выполнено
- [x] Ring 3, per-process memory, real syscalls и ELF execution.
- [x] Process registry, wait/reap, sleep deadlines и PIT wake/resume.
- [x] Exception CPL detection для #UD и #PF.
- [x] User invalid opcode завершает только Process.
- [x] User NX instruction fetch/page fault завершает только Process.
- [x] Fault status/exit code публикуются в registry, runner продолжает kernel.
- [x] QEMU negative tests для user #UD и NX fault.

### Следующий слой
- [ ] Write-to-code user fault test.
- [ ] User stack guard overflow test.
- [ ] Bad syscall pointer end-to-end negative test.
- [ ] Suspended syscall frame resume для sleep/wait.
- [ ] Spawn ownership: ELF + Process + registry + rollback.
- [ ] Handle table и `read/open/close/stat/mmap`.
- [ ] Rust runtime и user-space shell.

## M2+
- [ ] IPC/shared memory, VM/VFS, networking/security, graphics/desktop, packages/tooling.

## Следующий пакет
1. Negative protection tests: W^X write, stack guard, bad pointer.
2. Suspended syscall resume для sleep/wait.
3. Spawn ownership + handles.
4. Channels + shared memory.
