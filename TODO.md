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
- [x] Anonymous `mmap`: bounded page allocation, zero-fill, NX/W^X, overflow/overlap checks и partial-map rollback.
- [x] Rust `no_std` user runtime `aloha-user`: syscall wrappers, errno mapping, `entry!` и `panic_handler!`.
- [x] Shell `stat` и synchronous `spawn <file>`: FAT32 read, ELF validation, Ring 3 run и exit code.

### Сейчас делаем
- [ ] `SYS_SPAWN`: supervisor-managed child ownership, user path validation и atomic rollback.
- [ ] User-space shell: вынести shell execution из kernel и добавить `ls`, `cat`, `spawn`, `wait` через runtime.
- [ ] User runtime integration test: собрать маленький user ELF и прогнать его через QEMU.

## M2+
- [ ] IPC/shared memory, VM/VFS, networking/security, graphics/desktop, packages/tooling.

## Следующий пакет
1. `SYS_SPAWN` + supervisor child table, включая wait/exit ownership.
2. User-space shell на `aloha-user`.
3. QEMU end-to-end test: shell запускает user ELF и проверяет exit code.
