# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: собственный AlohaBoot UEFI bootloader и hybrid `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и UEFI memory map handoff.
- GDT, TSS, IDT, ISR и kernel panic screen.
- Physical frame allocator, 4-level paging и higher-half direct map.
- Reclaiming linked-list heap, `Box`, `Vec`, `String` и `dealloc`.
- PIC 8259, PIT 100 Hz, uptime, PS/2 keyboard и command history.
- Legacy VirtIO Block PCI driver с polling virtqueue.
- Read-only FAT32: root directory, чтение cluster chain, `ls /`, `cat hello.txt`.
- Shell: `help`, `clear`, `meminfo`, `uptime`, `ls`, `cat`, `reboot`.

## Windows

Установи Rust, QEMU и Python, затем из корня репозитория:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\run-qemu.ps1
```

Скрипт собирает AlohaBoot и kernel, скачивает OVMF при необходимости, создаёт 64-МБ FAT32 image с `HELLO.TXT`, подключает его как transitional VirtIO Block device и запускает QEMU. Для пересоздания диска удали `disk\aloha-fat32.img`.

## Проверка FAT32

```text
ls /
cat hello.txt
meminfo
uptime
```

## Дальше

- Запись FAT32, VFS и подкаталоги.
- Preemptive round-robin scheduler и context switch.
- Ring 3, user address spaces и syscalls.
- ACPI shutdown и APIC.

## Лицензия

Исходный код доступен для изучения, изменения и распространения в некоммерческих целях по **PolyForm Noncommercial License 1.0.0**. Коммерческое использование без отдельного разрешения запрещено. Полные условия находятся в [LICENSE.md](LICENSE.md).

Важно: из-за запрета коммерческого использования это source-available лицензия, а не OSI-approved open-source лицензия.
