# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable, Ring 3, syscalls, ELF execution и process scheduling foundation.
- Exception stubs различают CPL0 и CPL3 по saved CS.
- User invalid opcode и page fault завершают только active Process.
- Fault status синхронизируется в process registry, дети reparent-ятся, runner получает control обратно.
- Kernel exception path остаётся fatal и сохраняет panic diagnostics.
- Отдельные QEMU tests проверяют user `ud2` и NX instruction-fetch fault без kernel panic.

## Текущий этап: M1 Userland

Базовая crash isolation готова для #UD и #PF. Следующий пакет расширяет negative tests на write-to-code и stack guard, затем spawn ownership/rollback и handles.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
