# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable: scheduler, memory, synchronization, diagnostics и QEMU regression suite.
- Ring 3 descriptors, TSS `RSP0`, per-process PML4 и USER/NX W^X mappings.
- Process lifecycle, safe CR3 guard и validated multi-page user copies.
- Реальный `iretq` Ring 3 round-trip через controlled `int 0x80` trap.
- Versioned syscall ABI и единый safe dispatcher для `write`, `exit`, `sleep`.
- Pointer validation и bounded copy выполняются до обработки user buffers.
- UEFI loader, framebuffer, keyboard, VirtIO Block, FAT32, COM1 logger и shell.

## Текущий этап: M1 Userland

Теперь готова независимая syscall semantic layer: numbers, errno encoding, dispatcher и первые три операции. Следующий пакет подключит её к production `SYSCALL/SYSRET` entry через MSR, per-process kernel entry stack и hardened return checks.

Подробный статус: [TODO.md](TODO.md).

## Windows

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\run-qemu.ps1
```

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
