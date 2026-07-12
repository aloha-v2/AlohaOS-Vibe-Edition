# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable: scheduler, memory, synchronization, diagnostics и QEMU regression suite.
- Ring 3, per-process PML4, W^X, safe user copies и process-owned kernel entry stacks.
- Versioned syscall ABI и dispatcher для `write`, `exit`, `sleep`.
- EFER/STAR/LSTAR/FMASK, full assembly entry, switch с user RSP на process kernel stack и register frame.
- Реальный user `syscall` вызывает `write`, возвращается через validated `sysretq`, затем `exit` возвращает управление kernel.
- Отдельный QEMU syscall smoke проверяет LSTAR, dispatcher, SYSRET и process exit end-to-end.

## Текущий этап: M1 Userland

Первый настоящий syscall round-trip готов. Следующий пакет: IRET fallback для сложных return frames, реальный `sleep`, process table, ELF64 loader и fault isolation.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
