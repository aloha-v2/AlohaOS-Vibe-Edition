# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable, Ring 3, real syscalls, ELF execution и process registry.
- Process sleep хранит monotonic deadline и автоматически становится Ready при `advance_time`.
- Blocking wait регистрирует parent waiter; child exit публикует wake event и будит parent.
- Wake events одноразовые через `take_wake`, без polling состояния.
- `sleep` syscall теперь возвращает абсолютный deadline и обновляет registry.

## Текущий этап: M1 Userland

Готова process-level blocking модель. Следующий пакет связывает monotonic process time с PIT tick и runner resumption, затем fault isolation и spawn ownership.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
