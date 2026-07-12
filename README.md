# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: собственный AlohaBoot UEFI bootloader и `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и UEFI memory map handoff.
- GDT, TSS, IDT, ISR, отдельные IST stacks и kernel panic screen.
- Physical frame allocator с возвратом одиночных frames, 4-level paging и higher-half direct map.
- 1 MiB bump heap для ранних kernel objects, `Box`, `Vec` и `String`.
- PIC 8259, PIT 100 Hz, uptime и PS/2 keyboard.
- Полный x86_64 task context: GPR/IRET frame, CR3, FS/GS и XSAVE/XRSTOR.
- Task lifecycle: Ready, Running, Blocked, Sleeping и Dead.
- Отдельные 16 KiB kernel stacks с unmapped guard pages.
- Preemptive round-robin включён по умолчанию и прошёл часовой QEMU stress без Double Fault.
- IRQ-safe spinlock, scheduler-aware mutex, semaphore и wait queue.
- Allocation-free COM1 logger с уровнями DEBUG, INFO и ERROR.
- Bounded panic backtrace с офлайн-символизацией через `scripts/symbolize-backtrace.py`.
- Горячие подсистемы и descriptor tables больше не используют `static mut`.
- Legacy VirtIO Block сериализует DMA queue и request state через scheduler-aware mutex.
- Read-only FAT32: `ls /`, `cat hello.txt`; mount state защищён IRQ-safe lock.
- Shell: history, `help`, `clear`, `meminfo`, `uptime`, `tasks`, `sched`, `ls`, `cat`, `reboot`.
- GitHub Actions: release build, boot/storage, heap/memory/keyboard, exception и scheduler smoke tests.

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

`tasks` показывает lifecycle, число context switches и worker heartbeat. `sched on|off` оставлен как диагностический аварийный переключатель, обычная загрузка сразу включает preemption.

## Текущий этап: M1 Userland

**M0 Kernel Stable завершён.** Context switching, task lifecycle, guarded stacks, synchronization, frame reclamation, panic backtrace и пять CI jobs подтверждены. Scheduler прошёл часовой QEMU stress без Double Fault.

Следующий шаг: Ring 3 descriptors и TSS `RSP0`, отдельные user address spaces, USER/NX mappings и минимальный безопасный syscall path. Desktop пока не трогаем, ядро сначала должно научиться изолировать приложения.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0: использование, изменение и распространение разрешены только в некоммерческих целях. См. [LICENSE.md](LICENSE.md).
