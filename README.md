# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- **M0 Kernel Stable:** context switch, scheduler, synchronization, memory reclamation, panic backtrace и QEMU tests.
- Ring 3 descriptors, TSS `RSP0`, per-process PML4, USER/NX W^X mappings и validated user copies.
- Process lifecycle и interrupt-safe CR3 activation.
- Реальный `iretq` вход в Ring 3, отдельный user stack и контролируемый DPL3 trap через vector `0x80` обратно на kernel `RSP0`.
- Отдельный QEMU `ring3-smoke` проверяет полный user/kernel round-trip.
- UEFI loader, framebuffer, keyboard, VirtIO Block, FAT32, COM1 logger и shell.

## Текущий этап: M1 Userland

Сразу закрыт пакет первого запуска user mode: bootstrap RX image, guarded stack mappings, `iretq`, TSS stack transition, controlled trap и CI. Следующий пакет: production syscall MSRs/entry stack и `write/exit/sleep`, затем ELF loader и crash isolation.

Подробный статус: [TODO.md](TODO.md).

## Windows

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\run-qemu.ps1
```

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
