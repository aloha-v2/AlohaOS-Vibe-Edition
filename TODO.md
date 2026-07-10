# AlohaOS Roadmap

Цель: стабильная x86_64 ОС с user space, графическим сервером, рабочим столом, GUI-приложениями и настройками.

Порядок важен. Не начинайте desktop до завершения этапов 1-5.

## 1. Стабильность ядра

- [ ] Переписать полный x86_64 context switch: GPR, RIP, RSP, RFLAGS, CR3, FS/GS base, FPU/SSE/AVX через XSAVE/XRSTOR. **Реализовано в `brain/m0-context-switch`, ожидает QEMU stress-test.**
- [x] Реализовать lifecycle задач: Ready, Running, Blocked, Sleeping, Dead.
- [x] Добавить отдельный kernel stack для каждой задачи и guard page от переполнения.
- [ ] Сделать стабильный preemptive round-robin scheduler и stress-test переключений. **Scheduler реализован, runtime stress-test ещё не пройден.**
- [ ] Добавить spinlock, mutex, semaphore, wait queue и IRQ-safe locking.
- [ ] Убрать глобальные `static mut` из горячих подсистем.
- [ ] Реализовать нормальное освобождение физических фреймов.
- [x] Добавить kernel log, serial output в COM1 и уровни log severity.
- [ ] Добавить symbol table/backtrace для panic screen.
- [ ] Создать QEMU smoke tests: boot, exceptions, heap, scheduler, disk, keyboard.

**Готово, когда:** несколько задач работают час без Double Fault, утечек и зависаний.

### Выполнено

- COM1 logger без heap с boot/panic severity.
- Lifecycle: block, wake, timed sleep, exit.
- Отдельные guarded kernel stacks.
- Полный interrupt frame и extended context реализованы в рабочей ветке.
- `tasks` показывает switches и heartbeat фоновой задачи.

## 2. ACPI, APIC и современное железо

- [ ] Передавать RSDP из AlohaBoot в `BootInfo`.
- [ ] Парсить RSDT/XSDT, MADT, FADT, HPET и MCFG.
- [ ] Перейти с PIC 8259 на Local APIC + I/O APIC.
- [ ] Добавить APIC timer или HPET и точные monotonic clocks.
- [ ] Реализовать SMP bootstrap для дополнительных CPU.
- [ ] Реализовать ACPI reboot и shutdown.
- [ ] Добавить PCI/PCIe enumeration через MCFG.

## 3. Процессы, Ring 3 и системные вызовы

- [ ] Добавить ring-3 code/data descriptors и TSS `RSP0`.
- [ ] Создавать отдельный PML4 для каждого процесса.
- [ ] Мапить user pages с флагами USER и NX.
- [ ] Реализовать независимые address spaces или copy-on-write.
- [ ] Настроить `syscall/sysret` с безопасной проверкой адресов.
- [ ] Syscalls: `exit`, `write`, `read`, `open`, `close`, `stat`, `mmap`, `sleep`, `spawn`, `wait`.
- [ ] Handles/file descriptors и таблица ресурсов процесса.
- [ ] ELF loader для user-space программ.
- [ ] Перенести shell из Ring 0 в user space.
- [ ] Изолировать падение приложения от ядра и других процессов.

## 4. VFS и постоянная файловая система

- [ ] VFS API: inode, file, directory, mount point и path resolver.
- [ ] FAT32 как VFS driver, LFN и подкаталоги.
- [ ] FAT32 write/create/truncate/delete/rename.
- [ ] Block cache, dirty-page flushing и crash-safe запись.
- [ ] RAM filesystem для `/tmp`.
- [ ] Дерево `/bin`, `/apps`, `/system`, `/users`, `/tmp`, `/devices`.
- [ ] Минимальные владельцы и права доступа.

## 5. Драйверная модель и базовые устройства

- [ ] Device manager и единый driver API.
- [ ] VirtIO Block interrupts, multiple requests, write и flush.
- [ ] VirtIO Input или полноценный PS/2 mouse.
- [ ] Keyboard layouts, modifiers, repeat и Unicode.
- [ ] VirtIO GPU, double buffering и page flipping.
- [ ] EDID, resolution и display modes.
- [ ] RTC и wall-clock time.
- [ ] Позже: VirtIO Network и audio.

## 6-11. Graphics, IPC, desktop, apps, settings, security

После завершения этапов 1-5: 2D graphics, compositor, IPC, GUI toolkit, desktop shell, приложения, settings service и hardening.

## 12. Milestones

- [ ] **M0 Kernel Stable:** exceptions, memory, full scheduler, tests.
- [ ] **M1 Userland:** Ring 3, syscalls, ELF apps, user shell.
- [ ] **M2 Storage:** VFS, writable FAT32, persistent settings.
- [ ] **M3 Graphics:** mouse, VirtIO GPU, compositor, windows.
- [ ] **M4 Desktop:** panel, launcher, file manager, terminal.
- [ ] **M5 Daily Usable Demo:** settings, editor, networking basics, installer/image builder.

## Ближайшие задачи

1. Пройти QEMU runtime и stress-test нового context switch.
2. Добавить IRQ-safe synchronization primitives.
3. Добавить освобождение физических frames.
4. Перенести shell в Ring 3.
