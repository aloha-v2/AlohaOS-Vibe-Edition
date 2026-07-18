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
- Per-process handle table и file syscalls `open/read/close/stat` поверх read-only FAT32.
- Protection, suspended-resume и handle-table suites запускаются отдельными CI matrix.

## Текущий этап: M1 Userland

Handle table и базовые file syscalls покрыты end-to-end через реальный dispatch против FAT32-тома. Следующий пакет: `mmap`, затем Rust user runtime и user-space shell.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
