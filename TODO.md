# AlohaOS Roadmap

Цель: стабильная x86_64 ОС с изолированным user space, VFS, драйверами, графическим сервером и desktop applications.

## M0 Kernel Stable

- [x] Full x86_64 context, lifecycle, guarded stacks и preemptive scheduler.
- [x] IRQ-safe synchronization, frame reclamation и shared-state cleanup.
- [x] COM1 logging, panic backtrace и QEMU smoke tests.

## M1 Userland: процессы, Ring 3 и syscalls

- [x] Ring 3 code/data descriptors и TSS `RSP0`.
- [x] Отдельный PML4 root и выделенный user virtual region для процесса.
- [x] USER mappings с W^X: executable code read-only, writable data/stack NX.
- [ ] Безопасное переключение CR3 между kernel task и process address space.
- [ ] Первый вход в Ring 3 через `iretq` и возврат в kernel только через контролируемый trap.
- [ ] Настроить `syscall/sysret`: STAR/LSTAR/FMASK, canonical address checks и отдельный kernel entry stack.
- [ ] Минимальные syscalls: `write`, `exit`, `sleep`; затем `read`, `open`, `close`, `stat`, `mmap`, `spawn`, `wait`.
- [ ] Проверять каждый user pointer, длину, overflow и границы mappings до чтения/записи kernel.
- [ ] Process structure: PID, state, CR3, kernel stack, user stack, handles и exit code.
- [ ] ELF64 loader: validate headers/segments, enforce W^X, zero BSS, reject malformed binaries.
- [ ] Изолировать user page fault/invalid opcode: завершать только процесс, не kernel.
- [ ] Перенести shell из Ring 0 в user space.

**M1 готов, когда:** user ELF печатает текст, читает файл, sleep/wait работает, а его crash не валит kernel.

## M2 Storage

- [ ] VFS: inode/file/directory/mount/path resolver.
- [ ] FAT32 VFS driver: subdirectories, LFN, write/create/truncate/delete/rename.
- [ ] Block cache, flush, crash-safe metadata updates и RAM filesystem `/tmp`.
- [ ] File descriptors, permissions и дерево `/bin /apps /system /users /tmp /devices`.

## M3 Hardware and graphics foundation

- [ ] RSDP/ACPI tables, APIC/HPET, SMP, reboot/shutdown и PCIe MCFG.
- [ ] Device manager, VirtIO Block async/write/flush, mouse/input, VirtIO GPU, RTC.
- [ ] Double buffering, modes/EDID, networking basics и позже audio.

## M4-M5 Desktop

После стабильных process ABI, VFS и drivers: IPC/shared memory, compositor, GUI toolkit, desktop shell, apps, settings, package format и hardening.

## Ближайшие задачи

1. Добавить safe CR3 activation guard и проверить возврат в kernel address space.
2. Подготовить минимальные user code/stack mappings и `iretq` trampoline.
3. Настроить syscall entry с `write`, `exit`, `sleep` и user-pointer validation.
4. Добавить Process/ELF loader и crash isolation tests.
5. Перенести shell в user space, затем начать VFS.
