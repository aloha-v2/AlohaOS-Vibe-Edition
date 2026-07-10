# AlohaOS Vibe Edition

Экспериментальная x86_64 операционная система на Rust с небольшими ассемблерными ISR там, где без них нельзя.

## Архитектура

| Компонент | Реализация |
| --- | --- |
| Bootloader | AlohaBoot, собственный UEFI-загрузчик |
| Firmware | UEFI |
| Architecture | x86_64 |
| Language | Rust + x86_64 assembly |
| Kernel | Hybrid, `no_std` |

## Уже работает

- AlohaBoot загружает отдельный ELF ядра и передаёт framebuffer + UEFI memory map.
- GDT, TSS с IST-стеком, IDT и panic-экраны для CPU exceptions.
- Физический frame allocator, собственный PML4 и higher-half direct map.
- Reclaiming linked-list heap с рабочими `alloc`, `Box`, `Vec`, `String` и `dealloc`.
- PIC 8259, PIT 100 Hz, PS/2 keyboard IRQ1 и lock-free ring buffer.
- Shell с историей на 16 команд и навигацией стрелками Up/Down.
- Команды: `help`, `clear`, `meminfo`, `uptime`, `reboot`.

## Windows: сборка и запуск

Установи Rust и QEMU, затем открой PowerShell в корне репозитория:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\run-qemu.ps1
```

Скрипт сам:

1. добавит Rust targets `x86_64-unknown-uefi` и `x86_64-unknown-none`;
2. соберёт AlohaBoot и kernel;
3. создаст `esp\EFI\BOOT\BOOTX64.EFI` и `esp\alohaos\kernel.elf`;
4. скачает и закеширует OVMF в `firmware\OVMF_CODE.fd`, если файла ещё нет;
5. обойдёт проблемы QEMU с кириллицей в Windows-пути через временный диск;
6. запустит QEMU.

Только сборка:

```powershell
.\scripts\build.ps1
```

## Shell

```text
help       список команд
clear      очистить экран
meminfo    физическая память и статистика heap
uptime     время с запуска по PIT
reboot     reset через 8042, затем chipset fallback
Up/Down    история команд
```

## Следующие этапы

- VirtIO Block driver, VFS и FAT32.
- Preemptive round-robin scheduler и context switch.
- Ring 3, user address spaces и syscalls.
- ACPI shutdown и APIC.

## Лицензия

MIT
