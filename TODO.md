# AlohaOS Roadmap

Цель: стабильная x86_64 ОС с изолированным user space, VFS, драйверами, графическим сервером и desktop applications.

## M0 Kernel Stable

- [x] Full x86_64 context, lifecycle, guarded stacks и preemptive scheduler.
- [x] IRQ-safe synchronization, frame reclamation и shared-state cleanup.
- [x] COM1 logging, panic backtrace и QEMU smoke tests.

## M1 Userland: процессы, Ring 3 и syscalls

### Memory и process foundation

- [x] Ring 3 code/data descriptors и TSS `RSP0`.
- [x] Отдельный PML4 root и выделенный user virtual region.
- [x] USER mappings с W^X: executable code read-only, writable data/stack NX.
- [x] Interrupt-safe CR3 activation guard с гарантированным возвратом в kernel CR3.
- [x] Проверка canonical user ranges, overflow, page presence, USER и writable flags.
- [x] `copy_from_user`/`copy_to_user` через несколько страниц без прямого dereference user pointer.
- [x] Базовая Process structure: PID, lifecycle, CR3/address space, entry, user stack и exit code.

### Первый запуск user mode

- [ ] User image builder: скопировать минимальный machine code в read-only executable mapping.
- [ ] Подготовить aligned user stack, argc/env ABI пока не нужен.
- [ ] Реализовать `iretq` trampoline с CS/SS RPL3, RFLAGS.IF, user RIP/RSP.
- [ ] Добавить контролируемый software trap для первого возврата user to kernel.
- [ ] Проверить, что kernel entry реально использует TSS `RSP0`, а user stack остаётся изолирован.
- [ ] Добавить QEMU `ring3-smoke`: user code выполняется, печатает marker через trap и завершается.

### Syscall path

- [ ] Настроить EFER.SCE, STAR, LSTAR и FMASK.
- [ ] Assembly entry должен сохранить user RSP/RCX/R11 и перейти на process kernel stack до Rust.
- [ ] Проверять canonical RIP/RSP перед `sysretq`; при сомнении возвращаться через `iretq`.
- [ ] Реализовать `write`, `exit`, `sleep`, затем `read/open/close/stat/mmap/spawn/wait`.
- [ ] Запретить kernel pointers, integer overflow, unmapped ranges и cross-page partial access.
- [ ] Добавить syscall rate/error accounting и понятные errno values.

### ELF и isolation

- [ ] ELF64 loader: validate magic/class/machine/type/program headers и bounds.
- [ ] Мапить PT_LOAD, enforce W^X, zero BSS и отвергать overlapping/malformed segments.
- [ ] Process table: PID allocation, parent/child, handles, waiters и cleanup после exit.
- [ ] User page fault/invalid opcode должны завершать только процесс.
- [ ] Добавить negative tests: bad pointer, NX execution, write-to-code, stack overflow, malformed ELF.
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

## Следующий пакет работ

1. User image + stack mappings и `iretq` Ring 3 trampoline.
2. Контролируемый trap обратно в kernel и `ring3-smoke`.
3. Syscall MSRs, entry stack и `write/exit/sleep`.
4. Process table + ELF loader + crash isolation.
