# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable и полный QEMU regression suite.
- Ring 3, per-process address spaces, W^X, safe user copies и process-owned kernel stacks.
- Реальный SYSCALL/SYSRET путь для user `write` и `exit`.
- Allocation-free ELF64 validator строит bounded load plan.
- Проверяются ELF magic/class/endian/type/machine, header bounds, PT_LOAD ranges и entry point.
- W^X, user-region bounds, segment overlap и BSS (`memsz - filesz`) входят в validation policy.

## Текущий этап: M1 Userland

ELF parsing и безопасный load plan готовы. Следующий пакет реально мапит PT_LOAD pages в Process, копирует file bytes, обнуляет BSS и запускает ELF entry; затем process table и fault isolation.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
