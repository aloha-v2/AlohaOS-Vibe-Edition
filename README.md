# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable, Ring 3, real syscalls, ELF execution и process scheduling.
- User #UD и #PF изолируются от kernel.
- W^X реально проверяется: запись в executable read-only code page завершает только Process.
- Unmapped stack guard реально проверяется user write fault.
- Bad syscall pointer возвращает EFAULT, user process продолжает работу и cleanly exits.
- Sleep/wait сохраняют syscall frame и возобновляются с результатом в RAX.
- Валидные non-SYSRET frames возвращаются через sanitised `iretq` fallback.
- Spawn владеет ресурсами atomically: registry + address space + ELF, с полным rollback при любом сбое (PID и фреймы не текут).
- Protection, suspended-resume и spawn-rollback suites запускаются отдельными CI matrix.

## Текущий этап: M1 Userland

Owned spawn с rollback покрыт end-to-end. Следующий пакет: handle table и базовые file syscalls (`read/open/close/stat/mmap`), затем Rust user runtime и user-space shell.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
