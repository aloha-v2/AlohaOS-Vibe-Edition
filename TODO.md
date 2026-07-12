# AlohaOS Roadmap

## M0 Kernel Stable

- [x] Scheduler, memory, synchronization, diagnostics и QEMU regression suite.

## M1 Userland

### Ring 3 и process memory

- [x] Ring 3 descriptors, TSS `RSP0`, per-process PML4, USER/NX W^X и safe user copies.
- [x] Process lifecycle, CR3 guard, owned kernel entry stacks и Ring 3 `iretq` round-trip.

### Production syscall path

- [x] Versioned ABI, errno и dispatcher для `write/exit/sleep`.
- [x] EFER.SCE, STAR, LSTAR, FMASK и readback validation.
- [x] Assembly LSTAR entry сохраняет syscall number, 6 args, user RIP/RFLAGS/RSP.
- [x] Переключение с user RSP на active process kernel stack до Rust.
- [x] Dispatcher bridge и validated SYSRET fast path.
- [x] `exit` возвращает управление suspended kernel frame и сохраняет exit code.
- [x] QEMU real-syscall smoke: user `write`, SYSRET continuation, user `exit`.
- [ ] Реализовать IRET fallback для допустимых, но небезопасных для SYSRET frames.
- [ ] Подключить `sleep` к scheduler deadline и возобновлению процесса.
- [ ] Добавить `read/open/close/stat/mmap/spawn/wait` и handle table.

### ELF, process table и isolation

- [ ] ELF64 validation: magic/class/machine/type/header bounds.
- [ ] PT_LOAD mappings, W^X, BSS zeroing и overlap rejection.
- [ ] Process table: PID allocation, parent/child, waiters и cleanup.
- [ ] User page fault/invalid opcode завершают процесс, не kernel.
- [ ] Negative CI: bad pointer, NX execution, write-to-code, stack overflow, malformed ELF.
- [ ] Rust user runtime, wrappers, app Cargo build и user-space shell.

## M2 IPC, VM и storage

- [ ] Channels, pipes, shared memory/MAP_SHARED, Unix sockets и signals.
- [ ] Slab, VM areas, demand paging, file mappings и OOM policy.
- [ ] VFS, writable FAT32, page cache, permissions/capabilities и settings.

## M3-M5

- [ ] ACPI/APIC/SMP, networking/security, graphics/desktop, packages/tooling/docs.

## Следующий пакет

1. IRET fallback + real scheduler-backed sleep.
2. ELF64 parser/loader + malformed-image tests.
3. Process table, wait/cleanup и user-fault isolation.
4. Channels + shared memory.
