# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: AlohaBoot UEFI bootloader и `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и memory map handoff.
- GDT, TSS, IDT, ISR и panic screen.
- Physical allocator, paging, higher-half direct map и reclaiming heap.
- PIC, PIT 100 Hz, uptime и PS/2 keyboard.
- Legacy VirtIO Block + FAT32: `ls /` и `cat hello.txt` проверены на Windows/QEMU.
- Shell, COM1 logging, task lifecycle и guarded task stacks.
- Отдельный 20 KiB scheduler/timer IST stack проверен на Windows/QEMU без Double Fault.
- Assembly-only trampoline сохраняет CR3, FS/GS и XSAVE state; round-robin пока gated off.

## Scheduler status

Стабильная база подтверждена: shell, PIT timer, dedicated timer IST и FAT32 работают. Новый low-level trampoline не вызывает Rust между сохранением и восстановлением extended context. Следующий шаг: подключить persistent IST frames, включить round-robin за runtime gate и провести stress-test на Windows/QEMU.

## Windows

```powershell
git fetch origin
git reset --hard origin/brain/m0-context-switch
.\scripts\run-qemu.ps1
```

Проверка: `tasks`, `ls /`, `cat hello.txt`.

## Дальше

- Persistent per-task frames для timer IST.
- Включить round-robin за runtime gate.
- Часовой stress-test без Double Fault.
- IRQ-safe synchronization и frame deallocation.

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
