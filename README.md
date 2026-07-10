# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: собственный AlohaBoot UEFI bootloader и hybrid `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и UEFI memory map handoff.
- GDT, TSS, IDT, ISR и kernel panic screen.
- Physical frame allocator, 4-level paging и higher-half direct map.
- Reclaiming linked-list heap, `Box`, `Vec`, `String` и `dealloc`.
- PIC 8259, PIT 100 Hz, uptime и PS/2 keyboard.
- Timer scheduler hook и сохранение полного GPR interrupt frame; cross-task switch временно отключён до реализации полного x86_64 context (FPU/SIMD, kernel stacks, lifecycle).
- Legacy VirtIO Block и read-only FAT32: `ls /`, `cat hello.txt`.
- Shell: history, `help`, `clear`, `meminfo`, `uptime`, `tasks`, `ls`, `cat`, `reboot`.

## Windows

Установи Rust, QEMU и Python, затем из корня репозитория:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\run-qemu.ps1
```

## Проверка

```text
tasks
uptime
ls /
cat hello.txt
meminfo
```

`tasks` показывает PIT scheduling ticks. Настоящее переключение между kernel tasks сейчас намеренно staged off: предыдущая неполная реализация сохраняла только GPR/RSP и могла вызвать Double Fault.

## Дальше

- Полный x86_64 task context и стабильный preemptive scheduler.
- Запись FAT32, VFS и подкаталоги.
- Ring 3, user address spaces и syscalls.
- ACPI shutdown и APIC.

## Лицензия

PolyForm Noncommercial License 1.0.0: использование, изменение и распространение разрешены только в некоммерческих целях. См. [LICENSE.md](LICENSE.md).
