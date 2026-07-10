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
- Assembly-only trampoline сохраняет persistent IST frame, CR3, FS/GS и XSAVE state.
- Round-robin подключён за lazy runtime gate и по умолчанию выключен.

## Scheduler status

Hardware smoke на Windows/QEMU пройден: после `sched on` обе задачи дошли до 588 context switches, worker heartbeat вырос до 588, shell остался жив, `ls /` и `cat hello.txt` продолжили работать. Следующий барьер: часовой stress-test без Double Fault, зависаний и порчи FAT32.

## Windows

```powershell
git fetch origin
git reset --hard origin/brain/m0-context-switch
.\scripts\run-qemu.ps1
```

Проверка: `tasks`, затем `sched on`, снова `tasks`, `ls /`, `cat hello.txt`. Gate намеренно не переживает reboot.

## Дальше

- Автоматизировать длительный scheduler stress-test.
- Часовой прогон без Double Fault.
- После проверки включить round-robin по умолчанию.
- IRQ-safe synchronization и frame deallocation.

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
