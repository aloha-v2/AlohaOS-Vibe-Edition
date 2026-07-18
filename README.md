# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable, Ring 3, real syscalls, ELF execution и process scheduling.
- User #UD и #PF изолируются от kernel.
- W^X реально проверяется: запись в executable read-only code page завершает только Process.
- Unmapped stack guard реально проверяется user write fault.
- Bad syscall pointer возвращает EFAULT, user process продолжает работу и cleanly exits.
- Negative protection suite запускается отдельной CI matrix.

## Текущий этап: M1 Userland

Базовая memory protection и fault isolation покрыты end-to-end. Следующий пакет: suspended syscall resume для sleep/wait, затем spawn ownership/rollback и handle table.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
