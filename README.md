# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable и полный QEMU regression suite.
- Ring 3, per-process address spaces, real syscalls и ELF execution.
- Bounded process table с monotonic PID allocation.
- Parent/child links, exit status, wait/reap и orphan reparenting.
- Registry metadata отделена от ownership Process/address-space objects, чтобы cleanup оставался детерминированным.

## Текущий этап: M1 Userland

Process registry и wait semantics готовы. Следующий пакет связывает таблицу с реальным spawn/exit path и изолирует user exceptions: faulted process завершается и reaps, kernel продолжает работу.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
