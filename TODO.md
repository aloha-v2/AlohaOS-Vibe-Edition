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
- [x] Suspended syscall frame resume для sleep/wait с результатом в RAX.
- [x] IRET fallback для valid non-SYSRET frames.
- [x] Spawn ownership: ELF + Process + registry + rollback (+ rollback smoke в CI).
- [x] Per-process handle table и file syscalls `open/read/close/stat` поверх read-only FAT32 (+ handle smoke в CI).

### Следующий слой
- [ ] `mmap` для user-space.
- [ ] Rust runtime и user-space shell.

## M2+
- [ ] IPC/shared memory, VM/VFS, networking/security, graphics/desktop, packages/tooling.

## Следующий пакет
1. `mmap` + Rust user runtime.
2. User-space shell поверх spawn + handle table.
