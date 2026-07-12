# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: собственный AlohaBoot UEFI bootloader и `no_std` kernel.

## Работает сейчас

- M0 Kernel Stable: scheduler, synchronization, frame reclamation, panic backtrace и QEMU smoke tests.
- Ring 3 code/data descriptors и отдельный 32 KiB TSS `RSP0` stack.
- Per-process PML4 roots с выделенным user region и USER/NX page mappings.
- UEFI loader, framebuffer console, PS/2 keyboard, VirtIO Block и read-only FAT32.
- COM1 logging, shell и пять CI/QEMU jobs.

## Текущий этап: M1 Userland

Готовы Ring 3 descriptors, `RSP0` и базовый `AddressSpace`: новый PML4 наследует kernel mappings, но получает отдельный user PML4 slot. User code pages read-only executable, data/stack pages writable NX; owned page-table/data frames освобождаются вместе с address space.

Следующее: переключение CR3 для процесса, вход в Ring 3 через `iretq`, затем минимальный безопасный syscall path. Desktop всё ещё ждёт, и правильно делает.

Подробный статус: [TODO.md](TODO.md).

## Windows

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\run-qemu.ps1
```

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
