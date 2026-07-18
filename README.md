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
- Per-process handle table и file syscalls `open/read/close/stat` поверх read-only FAT32.
- Protection, suspended-resume, spawn-rollback и handle-table suites проходят в CI.

## Текущий этап: M1 Userland

Spawn ownership и file handles покрыты end-to-end. Следующий пакет: `mmap`, затем Rust user runtime и user-space shell.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
