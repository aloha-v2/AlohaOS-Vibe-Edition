# AlohaOS Roadmap

Цель: стабильная x86_64 ОС с изолированным user space, VFS, драйверами, графическим сервером и desktop applications.

Порядок важен. Сначала process ABI, memory safety, IPC и storage; затем networking, graphics и desktop. Прямой mmap framebuffer приложениям не даём: окна должны идти через compositor.

## M0 Kernel Stable

- [x] Full x86_64 context, lifecycle, guarded stacks и preemptive scheduler.
- [x] IRQ-safe synchronization, frame reclamation и shared-state cleanup.
- [x] COM1 logging, panic backtrace и QEMU smoke tests.

## M1 Userland: процессы, Ring 3 и syscalls

### Memory и process foundation

- [x] Ring 3 code/data descriptors и TSS `RSP0`.
- [x] Отдельный PML4 root и выделенный user virtual region.
- [x] USER mappings с W^X: executable code read-only, writable data/stack NX.
- [x] Interrupt-safe CR3 activation guard с возвратом в kernel CR3.
- [x] Проверка canonical user ranges, overflow, mappings, USER и writable flags.
- [x] Multi-page `copy_from_user`/`copy_to_user` без прямого dereference user pointer.
- [x] Process foundation: PID, lifecycle, address space, entry, user stack и exit code.

### Первый запуск user mode

- [ ] User image builder: копировать минимальный machine code в RX mapping.
- [ ] Подготовить aligned user stack и guard page.
- [ ] Реализовать `iretq` trampoline с CS/SS RPL3, IF, user RIP/RSP.
- [ ] Добавить контролируемый software trap user to kernel.
- [ ] Проверить реальное использование TSS `RSP0` и изоляцию user stack.
- [ ] QEMU `ring3-smoke`: user code выполняется, отдаёт marker и завершается.

### Syscall path и process ABI

- [ ] Настроить EFER.SCE, STAR, LSTAR и FMASK.
- [ ] Assembly entry сохраняет user RSP/RCX/R11 и переходит на process kernel stack.
- [ ] Проверять canonical RIP/RSP перед `sysretq`; небезопасный возврат делать через `iretq`.
- [ ] Реализовать `write`, `exit`, `sleep`, затем `read/open/close/stat/mmap/spawn/wait`.
- [ ] Ввести стабильные syscall numbers, errno, ABI version и rate/error accounting.
- [ ] Process table: PID allocation, parent/child, handles, waiters и cleanup после exit.
- [ ] User page fault/invalid opcode завершают только процесс.

### ELF, runtime и toolchain

- [ ] ELF64 loader: validate headers, machine/type, program headers и bounds.
- [ ] Мапить PT_LOAD, enforce W^X, zero BSS, reject overlap и malformed binaries.
- [ ] Минимальный Rust user runtime: `_start`, panic/alloc strategy и syscall wrappers.
- [ ] C ABI слой: собственная маленькая libc либо адаптация musl/newlib после стабилизации syscalls.
- [ ] Cross-build workflow для user apps через Cargo и Make, SDK crate и headers.
- [ ] Автоматическая сборка и QEMU CI для отдельных user-space приложений.
- [ ] Перенести shell из Ring 0 в user space.

**M1 готов, когда:** user ELF печатает текст, читает файл, sleep/wait работает, а crash не валит kernel.

## M2 IPC, virtual memory и storage

### IPC

- [ ] Kernel channels/message queues с bounded buffers и blocking wait queues.
- [ ] Анонимные и именованные pipes с EOF/backpressure semantics.
- [ ] Shared memory objects и `mmap(MAP_SHARED)` без копирования.
- [ ] Unix-domain sockets после стабилизации file descriptors и VFS namespace.
- [ ] Process signals: `kill`, masks, pending queue и безопасные user signal frames.
- [ ] Capability/handle transfer через IPC, без передачи raw kernel pointers.
- [ ] Fuzz tests для IPC messages, handle transfer и malformed signal frames.

### Virtual memory

- [ ] Reclaiming kernel allocator: slab caches для task/process/vnode/IPC objects.
- [ ] `mmap/munmap/mprotect`, VM areas и page permissions accounting.
- [ ] Demand paging через recoverable user page fault handler.
- [ ] File-backed mappings и shared dirty pages через VFS page cache.
- [ ] Copy-on-write для process spawn/fork-like semantics, если ABI это потребует.
- [ ] Per-process memory limits, OOM selection и clean process termination.
- [ ] Swap оставить опциональным после стабильного page cache, не тащить раньше времени.

### VFS и filesystem

- [ ] VFS: inode/file/directory/mount/path resolver.
- [ ] FAT32 VFS driver: subdirectories, LFN, write/create/truncate/delete/rename.
- [ ] Block/page cache, flush ordering и crash-safe metadata updates.
- [ ] RAM filesystem `/tmp`, device filesystem `/devices` и proc-like diagnostics.
- [ ] File descriptors, Unix-like permissions и capability-based access для services.
- [ ] Дерево `/bin /apps /system /users /tmp /devices`.
- [ ] Позже: ext2 read/write либо собственная journaled FS; ext4, encryption и FUSE только после VFS maturity.

**M2 готов, когда:** процессы общаются, shared memory работает, файлы и настройки переживают reboot.

## M3 Hardware, networking и platform security

### Modern hardware и power

- [ ] Передавать RSDP из AlohaBoot; парсить XSDT/MADT/FADT/HPET/MCFG.
- [ ] Local APIC + I/O APIC, HPET/APIC timer и SMP bootstrap.
- [ ] ACPI reboot/shutdown, затем C-states, P-states, battery status и suspend/resume.
- [ ] PCI/PCIe enumeration, device manager, driver lifecycle и hot-plug events.
- [ ] USB host stack/hot-plug намного позже базовой VirtIO/PCI стабильности.
- [ ] Loadable kernel modules отложить до появления строгого module ABI и signature policy.

### Networking

- [ ] VirtIO-net driver первым; RTL8139 только как legacy compatibility target.
- [ ] Ethernet framing, ARP, IPv4, ICMP и routing table.
- [ ] UDP, затем TCP state machine с retransmit, windows и timeouts.
- [ ] Socket API: socket/bind/listen/accept/connect/send/recv/poll.
- [ ] DHCP client, DNS resolver и loopback interface.
- [ ] HTTP client после TCP/DNS для package repositories и updates.
- [ ] Network namespace/cgroups считать будущим hardening, не блокером desktop.

### Security hardening

- [ ] Включить CPU NX, write-protect, SMEP и SMAP по CPUID capability.
- [ ] SMAP access только через короткие audited `stac/clac` user-copy sections.
- [ ] Stack canaries для kernel и user runtime.
- [ ] User ASLR после position-independent ELF; KASLR после relocation-ready kernel/bootloader.
- [ ] Secure Boot через подписанный AlohaBoot и проверку kernel image chain.
- [ ] Capability-based service access плюс Unix permissions для файлов.
- [ ] Watchdog, crash reports, safe mode и recovery shell.

## M4 Graphics, input, audio и desktop foundation

### Graphics и input

- [ ] VirtIO GPU, display modes/EDID, double buffering и page flipping.
- [ ] Input event API для keyboard, mouse и позже touch; layouts, repeat, Unicode и clipboard.
- [ ] RGBA surfaces, clipping, damage tracking, alpha blending и image blit.
- [ ] Font rasterizer/fallback/text measurement; начать с bitmap, затем FreeType-like TrueType path.
- [ ] PNG/BMP decoder сначала; JPEG позже и желательно в sandboxed user process.
- [ ] User-space compositor/display server с shared-memory buffers и оконным IPC protocol.
- [ ] Запретить приложениям прямой framebuffer access, кроме отдельного trusted compatibility service.

### Audio

- [ ] VirtIO Sound или Intel HDA driver, PCM playback/recording API.
- [ ] User-space audio server, mixer, per-app volume и device selection.

### Desktop

- [ ] Window API: create/move/resize/focus/close и decorations.
- [ ] Desktop shell: panel/taskbar, launcher, tray, clock, notifications и virtual desktops.
- [ ] GUI toolkit, layout engine, themes, accessibility roles и stable app SDK.
- [ ] File manager, terminal, editor, settings, system monitor, image viewer и calculator.

## M5 Distribution, packages и developer experience

### Packages и updates

- [ ] Package format: compressed archive + signed versioned manifest.
- [ ] Install database, atomic install/uninstall/update и rollback.
- [ ] Dependency resolution только после стабильных app ABI и filesystem layout.
- [ ] HTTP repository metadata, signature verification и offline cache.

### Debugging и observability

- [ ] Kernel GDB stub через serial/QEMU.
- [ ] Syscall tracing (`strace`-like) и structured kernel trace buffer.
- [ ] User debugger primitives: ptrace-like control, breakpoints и register access.
- [ ] Core dumps для user crashes с mappings, registers и process metadata.
- [ ] Sampling profiler/performance counters после APIC timer и SMP.

### Testing и docs

- [ ] Regression suite для syscalls, VM, VFS, IPC, networking и user apps.
- [ ] Fuzzing ELF, FAT32/VFS, path parser, syscalls, IPC и network packets.
- [ ] Host-side property tests; Miri только для portable crates, не bare-metal hardware code.
- [ ] Sanitizer-enabled host harnesses для parsers и protocol logic.
- [ ] rustdoc kernel/SDK API, syscall reference, driver guide, app guide, user manual и ADRs.

## Будущее, не блокирует основные milestones

- Containers: PID/network/mount namespaces, resource groups, chroot/pivot-root.
- Virtualization: расширенные VirtIO devices, QEMU guest agent, nested virtualization только при реальной необходимости.
- Localization: i18n, locales, timezones и расширенные keyboard layouts.
- Advanced storage: disk encryption, FUSE, snapshots и полноценная journaled filesystem.

## Следующий пакет работ

1. User image + guarded stack mappings и `iretq` Ring 3 trampoline.
2. Контролируемый trap обратно в kernel и отдельный `ring3-smoke` CI job.
3. Syscall MSRs, entry stack и пакет `write/exit/sleep`.
4. ELF loader, process table и crash isolation tests.
5. Затем channels + shared memory как первый IPC слой.
