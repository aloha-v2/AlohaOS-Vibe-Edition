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
- Горячие подсистемы и descriptor tables больше не используют `static mut`.
- Legacy VirtIO Block сериализует DMA queue и request state через scheduler-aware mutex.
- Read-only FAT32: `ls /`, `cat hello.txt`; mount state защищён IRQ-safe lock.
- Shell: history, `help`, `clear`, `meminfo`, `uptime`, `tasks`, `sched`, `ls`, `cat`, `reboot`.
- GitHub Actions: release build, QEMU boot/storage smoke и scheduler stress.

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

## Текущий этап: M0 Kernel Stable

Scheduler, memory reclamation и shared-state cleanup готовы. До закрытия M0 осталось добавить panic backtrace и расширить QEMU smoke tests на exceptions, heap и keyboard.

После M0: Ring 3, отдельные address spaces, минимальные syscalls и перенос shell в user space. Не начинаем desktop поверх нестабильного ядра, это путь в цирк с Double Fault.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0: использование, изменение и распространение разрешены только в некоммерческих целях. См. [LICENSE.md](LICENSE.md).
