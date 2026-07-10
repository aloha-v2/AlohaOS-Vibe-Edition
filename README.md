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

`brain/m0-context-switch` содержит preemptive two-task round-robin. Timer frame сохраняет GPR/RIP/RSP/RFLAGS, extended context хранит CR3, FS/GS base и x87/SSE/AVX через XSAVE/XRSTOR. Bootstrap shell пока остаётся на проверенном boot stack; background task использует отдельный guarded stack. Это осознанный staging шаг, чтобы не смешивать миграцию bootstrap stack с проверкой переключателя.

Команда `tasks` показывает context switches и worker heartbeat. До merge обязательны зелёный QEMU smoke test и ручная проверка.

## Windows

```powershell
git fetch origin
git switch brain/m0-context-switch
git pull
.\scripts\run-qemu.ps1
```

В shell: `tasks`, `uptime`, `ls /`, `cat hello.txt`, `meminfo`.

## Дальше

- Стабилизировать round-robin без Double Fault.
- Мигрировать bootstrap task на guarded stack отдельным изменением.
- Добавить IRQ-safe synchronization и frame deallocation.
- Ring 3, syscalls и user shell.

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
