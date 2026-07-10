# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: собственный AlohaBoot UEFI bootloader и hybrid `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и UEFI memory map handoff.
- GDT, TSS, IDT, ISR и kernel panic screen.
- Physical frame allocator, 4-level paging и higher-half direct map.
- Reclaiming linked-list heap, `Box`, `Vec`, `String` и `dealloc`.
- PIC 8259, PIT 100 Hz, uptime и PS/2 keyboard.
- Task lifecycle: Ready, Running, Blocked, Sleeping и Dead; timer автоматически будит sleeping tasks.
- Для каждой задачи выделен отдельный 16 KiB kernel stack с реально unmapped 4 KiB guard page.
- Timer scheduler hook и сохранение полного GPR interrupt frame; cross-task switch временно отключён до реализации FPU/SIMD context.
- Legacy VirtIO Block и read-only FAT32: `ls /`, `cat hello.txt`.
- Shell: history, `help`, `clear`, `meminfo`, `uptime`, `tasks`, `ls`, `cat`, `reboot`.
- Allocation-free COM1 kernel log с уровнями DEBUG, INFO и ERROR, включая panic output.

## Прогресс по roadmap

- Восстановлен полный boot flow ядра после регрессии со splash-only запуском.
- Добавлен COM1 logger без зависимости от heap.
- Добавлена атомарная модель lifecycle задач и timer-driven wakeup.
- Добавлены отдельные kernel stacks в выделенном virtual range; перед каждым стеком оставлена unmapped guard page.
- Следующий этап: полный x86_64 context (CR3, FS/GS, XSAVE/XRSTOR), затем round-robin switch.

Подробный порядок работ и статусы: [TODO.md](TODO.md).

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

`tasks` показывает состояния lifecycle, scheduling ticks и wake deadline. Межзадачное переключение пока намеренно выключено до сохранения extended CPU context.

## Дальше

- Полный x86_64 task context.
- Стабильный preemptive round-robin scheduler и stress test.
- Запись FAT32, VFS и подкаталоги.
- Ring 3, user address spaces и syscalls.

## Лицензия

PolyForm Noncommercial License 1.0.0: использование, изменение и распространение разрешены только в некоммерческих целях. См. [LICENSE.md](LICENSE.md).
