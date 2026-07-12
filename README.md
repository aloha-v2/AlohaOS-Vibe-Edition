# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: собственный AlohaBoot UEFI bootloader и `no_std` kernel.

## Готово

- **M0 Kernel Stable:** полный context switch, scheduler, synchronization, frame reclamation, panic backtrace и пять CI/QEMU checks.
- Ring 3 code/data descriptors и отдельный TSS `RSP0` stack.
- Per-process PML4 roots, USER/NX W^X mappings и автоматический возврат owned frames.
- Interrupt-safe CR3 activation guard с обязательным возвратом в kernel address space.
- Минимальная Process model: PID, lifecycle, entry, user stack и exit code.
- Полная проверка user ranges: canonical addresses, overflow, mappings, USER и writable permissions.
- `copy_from_user`/`copy_to_user` работают через page translation и поддерживают переход через границу страницы.
- UEFI loader, framebuffer console, PS/2 keyboard, VirtIO Block, read-only FAT32, COM1 logger и shell.

## Текущий этап: M1 Userland

За один пакет закрыты три основания M1: безопасное переключение CR3, Process ownership/lifecycle и validated user-memory copy. Следующий пакет: минимальный user image, `iretq` entry и контролируемый trap обратно в kernel, затем syscall MSRs.

Desktop пока не трогаем. Окна без изоляции процессов выглядят красиво ровно до первого page fault.

Подробный статус: [TODO.md](TODO.md).

## Windows

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\run-qemu.ps1
```

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
