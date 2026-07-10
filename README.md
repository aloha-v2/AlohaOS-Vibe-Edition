# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: собственный AlohaBoot UEFI bootloader и hybrid `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и UEFI memory map handoff.
- GDT, TSS, IDT, ISR и kernel panic screen.
- Physical frame allocator, 4-level paging и higher-half direct map.
- Reclaiming linked-list heap, `Box`, `Vec`, `String` и `dealloc`.
- PIC 8259, PIT 100 Hz, uptime и PS/2 keyboard.
- Task lifecycle: Ready, Running, Blocked, Sleeping и Dead.
- Отдельные 16 KiB kernel stacks с unmapped 4 KiB guard pages.
- Allocation-free COM1 log с DEBUG, INFO, ERROR и panic output.
- Legacy VirtIO Block и read-only FAT32: `ls /`, `cat hello.txt`.
- Shell: history, `help`, `clear`, `meminfo`, `uptime`, `tasks`, `ls`, `cat`, `reboot`.
- CI проверяет release-сборку AlohaBoot и kernel.

## Ветка M0 context switch

В `brain/m0-context-switch` реализован preemptive round-robin для двух kernel tasks. Timer frame переключает GPR, RIP, RSP и RFLAGS; task context сохраняет CR3, FS/GS base и x87/SSE/AVX через XSAVE/XRSTOR. Shell и worker работают на отдельных guarded stacks.

Это low-level изменение пока не готово к merge: сначала нужны зелёный CI и runtime-проверка в QEMU. Команда `tasks` должна показывать растущие `CONTEXT SWITCHES` у обеих задач и `WORKER HEARTBEAT`.

Подробный порядок работ и статусы: [TODO.md](TODO.md).

## Windows

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

## Дальше

- QEMU stress-test round-robin без Double Fault.
- IRQ-safe synchronization primitives.
- Освобождение физических frames.
- Ring 3, syscalls и user shell.

## Лицензия

PolyForm Noncommercial License 1.0.0: только некоммерческое использование, изменение и распространение. См. [LICENSE.md](LICENSE.md).
