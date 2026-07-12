# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable и полный QEMU regression suite.
- Ring 3, real syscalls, ELF loading/execution и process table.
- Existing Process objects можно регистрировать с parent metadata.
- `getpid`, `sleep`, `exit`, `wait` синхронизируют Process lifecycle с registry.
- `wait` возвращает Busy для живого child, exit status после завершения и атомарно reaps запись.
- Exit reparent-ит детей к PID 0.

## Текущий этап: M1 Userland

Process lifecycle теперь проходит через syscall dispatcher и registry. Следующий пакет: blocking wait queue и scheduler-backed sleep deadline, затем user fault isolation.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
