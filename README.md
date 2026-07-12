# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable: scheduler, memory, synchronization, diagnostics и QEMU regression suite.
- Ring 3 descriptors, TSS `RSP0`, per-process PML4, USER/NX W^X mappings и validated user copies.
- Реальный `iretq` Ring 3 round-trip и controlled DPL3 trap.
- Versioned syscall ABI и dispatcher для `write`, `exit`, `sleep`.
- Каждый Process теперь владеет отдельным 32 KiB kernel entry stack.
- Syscall return frames классифицируются: safe `sysret`, audited `iret` fallback или reject.
- CPUID capability check и RFLAGS sanitization готовы до включения MSR entry.

## Текущий этап: M1 Userland

Сначала закрепили опасные инварианты entry path: stack ownership, canonical RIP/RSP, forbidden RFLAGS и безопасный выбор return instruction. Следующий PR уже может включать EFER/STAR/LSTAR/FMASK и assembly entry, не смешивая это с непроверенной логикой.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
