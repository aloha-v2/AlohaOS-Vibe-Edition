# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable, Ring 3, real syscalls, ELF execution и process registry.
- PIT timer напрямую двигает process sleep deadlines.
- Expired sleepers получают Ready state и one-shot wake event.
- Process runner reconciliation переносит registry wake в owned Process state.
- Blocking wait после child exit возобновляет parent и завершает reap со status.

## Текущий этап: M1 Userland

Sleep/wait wake path теперь связан с реальным PIT tick и runner state. Следующий пакет: user fault isolation с возвратом в runner, затем spawn ownership и handles.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
