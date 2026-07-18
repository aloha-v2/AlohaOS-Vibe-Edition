# AlohaOS Roadmap

## M0 Kernel Stable
- [x] Scheduler, memory, synchronization, diagnostics и QEMU suite.

## M1 Userland

### Выполнено
- [x] Ring 3, per-process memory, real syscalls, ELF execution и process scheduling.
- [x] User #UD/#PF isolation и runner recovery.
- [x] NX execution negative test.
- [x] Write-to-code W^X fault test.
- [x] Unmapped user stack guard fault test.
- [x] Bad syscall pointer возвращает EFAULT без kernel fault.
- [x] CI matrix для protection negative tests.

### Следующий слой
- [ ] Suspended syscall frame resume для sleep/wait с результатом в RAX.
- [ ] IRET fallback для valid non-SYSRET frames.
- [ ] Spawn ownership: ELF + Process + registry + rollback.
- [ ] Handle table и `read/open/close/stat/mmap`.
- [ ] Rust runtime и user-space shell.

## M2+
- [ ] IPC/shared memory, VM/VFS, networking/security, graphics/desktop, packages/tooling.

## Следующий пакет
1. Suspended sleep/wait syscall resume + IRET fallback.
2. Spawn ownership + handle table.
3. Channels + shared memory.
