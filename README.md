# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable и полный QEMU regression suite.
- Ring 3, per-process PML4, W^X, safe user copies и process-owned kernel stacks.
- Реальный SYSCALL/SYSRET путь для user `write` и `exit`.
- ELF64 validation, PT_LOAD mapping, file copy и BSS zeroing.
- Загруженный ELF реально входит в Ring 3 по `e_entry` и завершает процесс через настоящий syscall `exit`.
- Отдельный QEMU ELF smoke проверяет parse, load, execute и exit end-to-end.

## Текущий этап: M1 Userland

Первая ELF user-программа запускается end-to-end. Следующий пакет: process table с PID/parent/wait/cleanup и user fault isolation, затем Rust user runtime и перенос shell.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
