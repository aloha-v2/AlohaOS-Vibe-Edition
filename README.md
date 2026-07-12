# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable: scheduler, memory, synchronization, diagnostics и QEMU regression suite.
- Ring 3, per-process address spaces, W^X, safe user copies и process-owned kernel entry stacks.
- Versioned syscall ABI и dispatcher для `write`, `exit`, `sleep`.
- EFER.SCE, STAR, LSTAR и FMASK настраиваются и проверяются readback.
- Формализован `SyscallFrame` с шестью аргументами, user RIP/RFLAGS/RSP и result.
- Active process устанавливает собственный kernel entry stack до будущего assembly entry.
- LSTAR пока fail-closed: ранний `syscall` останавливает CPU, а не выполняет Rust на user stack.

## Текущий этап: M1 Userland

MSR contract и register context готовы. Следующий пакет заменит fail-closed LSTAR stub полноценным assembly trampoline: сохранить user state, перейти на process stack, вызвать dispatcher и вернуть через проверенный SYSRET/IRET path.

Подробный статус: [TODO.md](TODO.md).

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
