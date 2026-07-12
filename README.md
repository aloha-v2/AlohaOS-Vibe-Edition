# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable и полный QEMU regression suite.
- Ring 3, per-process address spaces, W^X, safe user copies и process-owned kernel stacks.
- Реальный SYSCALL/SYSRET путь для user `write` и `exit`.
- ELF64 validator и реальный PT_LOAD loader.
- Loader мапит user pages с финальными W^X permissions, копирует file bytes через physical ownership и явно обнуляет BSS.
- ELF entry устанавливается в Process; loader smoke проверяет file image и zero-filled tail.

## Текущий этап: M1 Userland

Теперь ELF не только валидируется, но реально загружается в process address space. Следующий пакет запускает ELF entry end-to-end, затем добавляет process table, wait/cleanup и изоляцию user faults.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
