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

### Сейчас делаем
- [ ] `SYS_SPAWN`: запуск ELF из FAT32 из user-space с атомарным rollback.
- [ ] User-space shell: command line, `ls`, `cat`, `spawn`, `wait`, exit status.
- [ ] User runtime integration test: собрать маленький user ELF и прогнать его через QEMU.

## M2+
- [ ] IPC/shared memory, VM/VFS, networking/security, graphics/desktop, packages/tooling.

## Следующий пакет
1. `SYS_SPAWN` поверх существующего `spawn_elf` и read-only FAT32.
2. User-space shell, который использует `open/read/close`, `spawn` и `wait`.
3. QEMU end-to-end test: shell запускает user ELF и проверяет exit code.
