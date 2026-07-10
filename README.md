# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: собственный AlohaBoot UEFI bootloader и hybrid `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и UEFI memory map handoff.
- GDT, TSS, IDT, ISR и kernel panic screen.
- Physical frame allocator, 4-level paging и higher-half direct map.
- Reclaiming linked-list heap, `Box`, `Vec`, `String` и `dealloc`.
- PIC 8259, PIT 100 Hz, uptime и PS/2 keyboard.
- Preemptive round-robin kernel scheduler, отдельный 64-КБ stack фонового task и context switch из timer IRQ каждые 5 тиков.
- Legacy VirtIO Block и read-only FAT32: `ls /`, `cat hello.txt`.
- Shell: history, `help`, `clear`, `meminfo`, `uptime`, `tasks`, `ls`, `cat`, `reboot`.

## Windows

Установи Rust, QEMU и Python, затем из корня репозитория:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\run-qemu.ps1
```

Скрипт собирает AlohaBoot и kernel, скачивает OVMF, создаёт 64-МБ FAT32 image и подключает transitional VirtIO Block device.

## Проверка

```text
tasks
uptime
ls /
cat hello.txt
meminfo
```

Повторный `tasks` должен показывать растущие `SWITCHES` и `WORKER HEARTBEAT`: это подтверждает реальное вытесняющее переключение между shell task и фоновым kernel task.

## Дальше

- Запись FAT32, VFS и подкаталоги.
- Ring 3, user address spaces и syscalls.
- ACPI shutdown и APIC.

## Лицензия

PolyForm Noncommercial License 1.0.0: использование, изменение и распространение разрешены только в некоммерческих целях. См. [LICENSE.md](LICENSE.md).
