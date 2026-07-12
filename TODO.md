# AlohaOS Roadmap

Цель: стабильная x86_64 ОС с изолированным user space, VFS, IPC, networking и desktop applications.

## M0 Kernel Stable

- [x] Context switching, scheduler, synchronization, memory reclamation, panic diagnostics и smoke tests.

## M1 Userland

### Process memory

- [x] Ring 3 descriptors и TSS `RSP0`.
- [x] Per-process PML4, USER/NX W^X mappings и owned-frame cleanup.
- [x] Interrupt-safe CR3 guard, Process lifecycle и validated multi-page user copies.

### Первый Ring 3 round-trip

- [x] Bootstrap user image в RX mapping.
- [x] Отдельный 4-page NX user stack; соседние страницы остаются guard gaps.
- [x] `iretq` trampoline с RPL3 CS/SS, user RIP/RSP и IF.
- [x] DPL3 software trap `int 0x80` с hardware switch на TSS `RSP0`.
- [x] Контролируемый возврат в suspended kernel frame и restore kernel CR3.
- [x] Отдельный QEMU `ring3-smoke` CI job.

### Production syscall ABI

- [ ] Настроить EFER.SCE, STAR, LSTAR и FMASK.
- [ ] Per-process kernel entry stack вместо bootstrap global return slot.
- [ ] Assembly entry сохраняет user RSP/RCX/R11 и callee-saved registers.
- [ ] Canonical RIP/RSP checks; безопасный fallback на `iretq` вместо опасного `sysretq`.
- [ ] Syscalls `write`, `exit`, `sleep` с errno и ABI version.
- [ ] Затем `read/open/close/stat/mmap/spawn/wait` и handles.

### ELF и isolation

- [ ] ELF64 validation, PT_LOAD W^X mappings, BSS zeroing и overlap rejection.
- [ ] Process table, parent/child, waiters и cleanup.
- [ ] User faults завершают процесс, не kernel.
- [ ] Negative CI: bad pointer, NX execution, write-to-code, stack overflow, malformed ELF.
- [ ] Rust user runtime, syscall wrappers, app Cargo build и user-space shell.

## M2 IPC, VM и storage

- [ ] Channels/message queues, pipes, shared memory + `MAP_SHARED`, затем Unix sockets и signals.
- [ ] Slab allocator, VM areas, mmap/mprotect, demand paging, file mappings и OOM policy.
- [ ] VFS, writable FAT32, page cache, `/tmp`, permissions/capabilities и persistent settings.

## M3 Hardware, network и security

- [ ] ACPI/APIC/HPET/SMP/PCIe, power management и device manager.
- [ ] VirtIO-net, ARP/IPv4/ICMP, UDP/TCP, sockets, DHCP, DNS и HTTP.
- [ ] SMEP/SMAP, canaries, ASLR/KASLR, Secure Boot, watchdog и recovery.

## M4-M5 Graphics, desktop и distribution

- [ ] VirtIO GPU, input events, compositor, toolkit, desktop shell, apps и audio.
- [ ] libc/runtime/toolchain, package manager, signed repositories и updates.
- [ ] GDB stub, tracing, core dumps, profiler, fuzzing, regression CI и docs.

## Следующий пакет

1. Syscall MSRs + hardened assembly entry/return.
2. `write/exit/sleep` и syscall smoke tests.
3. ELF loader + process table + fault isolation.
4. Channels + shared memory как первый IPC слой.
