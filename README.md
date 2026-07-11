# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: AlohaBoot UEFI bootloader и `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и memory map handoff.
- GDT, TSS, IDT, ISR и panic screen.
- Physical allocator, paging, higher-half direct map и reclaiming heap.
- PIC, PIT 100 Hz, uptime и PS/2 keyboard.
- Legacy VirtIO Block + FAT32: `ls /` и `cat hello.txt` проверены на Windows/QEMU.
- Shell, COM1 logging, task lifecycle и guarded task stacks.
- Dedicated 20 KiB scheduler/timer IST stack проверен на Windows/QEMU без Double Fault.
- Assembly-only trampoline сохраняет persistent GPR/IRET frame, CR3, FS/GS и XSAVE state.
- Gated preemptive round-robin прошёл hardware smoke, 60s CI stress и полный часовой QEMU stress.
- IRQ-safe spinlock реализован; COM1 logger переведён на него.

## Scheduler status

Готовы lifecycle, guarded stacks, timer IST, assembly-only extended-context trampoline, persistent per-task frames и runtime gate `sched on|off`. Hardware smoke на Windows/QEMU пройден: обе задачи дошли до 588 switches, worker heartbeat вырос до 588, shell и FAT32 остались живы.

Автоматические 60-секундный и часовой scheduler stress-тесты завершились успешно без Double Fault, kernel panic и ошибок storage. Следующий шаг: включить round-robin по умолчанию и продолжить миграцию shared state на IRQ-safe primitives.

## Windows

```powershell
git fetch origin
git reset --hard origin/brain/m0-context-switch
.\scripts\run-qemu.ps1
```

Проверка: `tasks`, `sched on`, снова `tasks`, `ls /`, `cat hello.txt`.

## Дальше

- Включить preemptive round-robin по умолчанию.
- Мигрировать heap/device shared state на IRQ-safe primitives.
- Добавить mutex, semaphore и wait queue.
- Physical frame deallocation.

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
