# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust: AlohaBoot UEFI bootloader и `no_std` kernel.

## Работает сейчас

- UEFI ELF loader, framebuffer и memory map handoff.
- GDT, TSS, IDT, ISR и panic screen.
- Physical allocator, paging, higher-half direct map и reclaiming heap.
- PIC, PIT 100 Hz, uptime, PS/2 keyboard, VirtIO Block и read-only FAT32.
- Shell, COM1 logging, task lifecycle и guarded task stacks.
- Отдельный 20 KiB scheduler/timer IST stack; build и QEMU smoke зелёные.

## Scheduler status

Прототип полного XSAVE context switch обнаружил Double Fault на Windows/QEMU, поэтому cross-task switching безопасно gated off. PIT, shell и lifecycle стабильны. Timer IRQ уже перенесён на отдельный IST stack, что изолирует scheduler path от текущего task stack.

Следующий шаг: assembly-only context switch trampoline, затем включение round-robin только после QEMU и аппаратного stress-test.

## Windows

```powershell
git fetch origin
git switch brain/m0-context-switch
git reset --hard origin/brain/m0-context-switch
.\scripts\run-qemu.ps1
```

## Дальше

- Assembly-only switch trampoline.
- Включить round-robin за runtime gate.
- Часовой stress-test без Double Fault.
- IRQ-safe synchronization и frame deallocation.

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
