# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: собственный AlohaBoot UEFI bootloader и hybrid `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и UEFI memory map handoff.
- GDT, TSS, IDT, ISR и kernel panic screen.
- Physical frame allocator, 4-level paging и higher-half direct map.
- Reclaiming linked-list heap, `Box`, `Vec`, `String` и `dealloc`.
- PIC 8259, PIT 100 Hz, uptime и PS/2 keyboard.
- Task lifecycle: Ready, Running, Blocked, Sleeping и Dead; timer автоматически будит sleeping tasks.
- Для каждой задачи выделен отдельный guarded kernel stack.
- Dedicated 20 KiB scheduler/timer IST stack.
- Assembly-only context-switch trampoline сохраняет persistent GPR/IRET frame, CR3, FS/GS base и FPU/SSE/AVX через XSAVE/XRSTOR.
- Preemptive two-task round-robin доступен через runtime gate `sched on|off`.
- Legacy VirtIO Block и read-only FAT32: `ls /`, `cat hello.txt`.
- Shell: history, `help`, `clear`, `meminfo`, `uptime`, `tasks`, `sched on|off`, `ls`, `cat`, `reboot`.
- Allocation-free COM1 kernel log с уровнями DEBUG, INFO и ERROR, включая panic output.
- IRQ-safe spinlock; COM1 logger уже переведён на него.
- GitHub Actions проверяет release build, boot/timer/FAT32 smoke и scheduler stress.

## Прогресс по roadmap

- Полный x86_64 context-switch механизм готов.
- Hardware smoke на Windows/QEMU пройден: обе задачи дошли до 588 switches, worker heartbeat вырос до 588, shell и FAT32 остались рабочими.
- Автоматический 60-секундный scheduler stress прошёл без Double Fault.
- Полный часовой QEMU stress прошёл 3600 секунд без Double Fault, kernel panic и storage errors.
- Изменения влиты в `main`; build, QEMU smoke и scheduler stress на `main` зелёные.
- Следующий этап: включить round-robin по умолчанию, мигрировать heap/device shared state на IRQ-safe primitives и расширить QEMU tests.

Подробный порядок работ и статусы: [TODO.md](TODO.md).

## Windows

Установи Rust, QEMU и Python, затем из корня репозитория:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
git switch main
git pull origin main
.\scripts\run-qemu.ps1
```

## Проверка

```text
tasks
sched on
tasks
uptime
ls /
cat hello.txt
meminfo
```

После `sched on` команда `tasks` должна показывать растущие context switches у обеих задач и ненулевой worker heartbeat. Runtime gate пока намеренно выключен после reboot; включение round-robin по умолчанию остаётся следующим отдельным шагом.

## Дальше

- Включить preemptive round-robin по умолчанию.
- Мигрировать heap и device shared state на IRQ-safe primitives.
- Добавить mutex, semaphore и wait queue.
- Реализовать physical frame deallocation.
- Расширить QEMU tests: exceptions, heap, disk и keyboard.
- Затем Ring 3, user address spaces, syscalls и user-space shell.

## Лицензия

PolyForm Noncommercial License 1.0.0: использование, изменение и распространение разрешены только в некоммерческих целях. См. [LICENSE.md](LICENSE.md).
