# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: AlohaBoot UEFI bootloader и `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и memory map handoff.
- GDT, TSS, IDT, ISR и panic screen.
- Physical allocator, paging, higher-half direct map и reclaiming heap.
- PIC, PIT 100 Hz, uptime, PS/2 keyboard, VirtIO Block и read-only FAT32.
- Shell, COM1 logging, task lifecycle и guarded task stacks.
- CI build и headless QEMU smoke test.

## Scheduler status

Полный XSAVE context и round-robin прототип обнаружил Double Fault на реальном Windows/QEMU запуске. Cross-task switch снова безопасно gated off: PIT, shell и lifecycle продолжают работать, но второй task не запускается. Это лучше, чем делать вид, что scheduler готов и оставлять ОС падающей.

Следующая реализация должна использовать отдельный scheduler interrupt stack, строгий assembly trampoline и аппаратный stress-test до включения по умолчанию.

## Windows

```powershell
git fetch origin
git switch brain/m0-context-switch
git reset --hard origin/brain/m0-context-switch
.\scripts\run-qemu.ps1
```

## Дальше

- Переписать context-switch trampoline без Rust на switch path.
- Добавить dedicated IST/scheduler stack.
- Только после этого включить round-robin и часовой stress-test.

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
