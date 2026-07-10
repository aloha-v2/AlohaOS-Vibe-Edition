# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: AlohaBoot UEFI bootloader и `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и memory map handoff.
- GDT, TSS, IDT, ISR и panic screen.
- Physical allocator, paging, higher-half direct map и reclaiming heap.
- PIC, PIT 100 Hz, uptime и PS/2 keyboard.
- Legacy VirtIO Block + FAT32 image: `ls /`, `cat hello.txt`.
- Shell, COM1 logging, task lifecycle и guarded task stacks.
- Отдельный 20 KiB scheduler/timer IST stack.

## Scheduler status

Полный XSAVE context-switch прототип поймал Double Fault на Windows/QEMU, поэтому cross-task switch безопасно выключен. Timer IRQ уже работает на отдельном IST stack. Следующий шаг: assembly-only trampoline, затем stress-test до включения round-robin.

## Windows

```powershell
git fetch origin
git reset --hard origin/brain/m0-context-switch
.\scripts\run-qemu.ps1
```

После запуска проверь `tasks`, `ls /` и `cat hello.txt`.

## Дальше

- Assembly-only context switch trampoline.
- Часовой scheduler stress-test.
- IRQ-safe synchronization и frame deallocation.

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
