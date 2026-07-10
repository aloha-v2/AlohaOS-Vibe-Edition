# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: AlohaBoot UEFI bootloader и `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и memory map handoff.
- GDT, TSS, IDT, ISR и panic screen.
- Physical allocator, paging, higher-half direct map и reclaiming heap.
- PIC, PIT 100 Hz, uptime, PS/2 keyboard, VirtIO Block и read-only FAT32.
- Shell и allocation-free COM1 logging.
- Task lifecycle и guarded kernel stack для background worker.
- CI build и headless QEMU smoke test.

## M0 context-switch branch

`brain/m0-context-switch` содержит preemptive two-task round-robin. Timer frame сохраняет GPR/RIP/RSP/RFLAGS, extended context хранит CR3, FS/GS base и x87/SSE/AVX через XSAVE/XRSTOR. Bootstrap shell остаётся на проверенном boot stack; background task использует отдельный guarded stack с unmapped lower guard и mapped ABI headroom.

Автоматические release build и QEMU smoke test проходят. Перед merge нужна последняя ручная проверка: команда `tasks` должна показать `XSAVE CONTEXT SWITCH ACTIVE`, растущие context switches у обеих задач и worker heartbeat больше нуля.

## Windows

```powershell
git fetch origin
git switch brain/m0-context-switch
git pull
.\scripts\run-qemu.ps1
```

В shell: `tasks`, `uptime`, `ls /`, `cat hello.txt`, `meminfo`.

## Дальше

- Ручная проверка round-robin, затем merge PR #4.
- Длительный scheduler stress test.
- IRQ-safe synchronization и frame deallocation.
- Ring 3, syscalls и user shell.

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
