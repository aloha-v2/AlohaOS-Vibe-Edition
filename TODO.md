# AlohaOS Roadmap

Цель: стабильная x86_64 ОС с user space, графическим сервером, рабочим столом, GUI-приложениями и настройками.

## 1. Стабильность ядра

- [x] Полный x86_64 context switch: GPR, RIP, RSP, RFLAGS, CR3, FS/GS, XSAVE/XRSTOR.
- [x] Lifecycle задач, guarded kernel stacks и preemptive round-robin.
- [x] Spinlock, mutex, semaphore, wait queue и IRQ-safe locking.
- [x] Frame reclamation, COM1 logging, panic backtrace и QEMU smoke tests.
- [x] **M0 Kernel Stable:** build, boot/storage, subsystem, exception и scheduler checks зелёные.

## 2. ACPI, APIC и современное железо

- [ ] Передавать RSDP из AlohaBoot в `BootInfo`.
- [ ] Парсить RSDT/XSDT, MADT, FADT, HPET и MCFG.
- [ ] Перейти с PIC 8259 на Local APIC + I/O APIC.
- [ ] Добавить APIC timer или HPET и точные monotonic clocks.
- [ ] Реализовать SMP bootstrap, ACPI reboot/shutdown и PCIe enumeration.

## 3. Процессы, Ring 3 и системные вызовы

- [x] Добавить ring-3 code/data descriptors и TSS `RSP0`.
- [ ] Создавать отдельный PML4 для каждого процесса.
- [ ] Мапить user pages с флагами USER и NX.
- [ ] Реализовать независимые address spaces, позже copy-on-write.
- [ ] Настроить `syscall/sysret` с безопасной проверкой адресов.
- [ ] Начальные syscalls: `exit`, `write`, `read`, `open`, `close`, `stat`, `mmap`, `sleep`, `spawn`, `wait`.
- [ ] Добавить handles/file descriptors и таблицу ресурсов процесса.
- [ ] Реализовать ELF loader для user-space программ.
- [ ] Перенести shell из Ring 0 в user space.
- [ ] Изолировать падение приложения от ядра и других процессов.

**Готово, когда:** user-программа печатает текст, читает файл, падает и не валит kernel.

## 4. VFS и постоянная файловая система

- [ ] VFS API: inode, file, directory, mount point и path resolver.
- [ ] FAT32 как VFS driver: LFN, write/create/truncate/delete/rename.
- [ ] Block cache, dirty-page flushing, RAM filesystem для `/tmp`.
- [ ] Дерево `/bin`, `/apps`, `/system`, `/users`, `/tmp`, `/devices`.

## 5. Драйверная модель и базовые устройства

- [ ] Device manager и единый driver API.
- [ ] VirtIO Block interrupts, несколько запросов, write и flush.
- [ ] Mouse/input, keyboard layouts/repeat/Unicode и VirtIO GPU.
- [ ] Double buffering, page flipping, EDID, RTC и wall-clock.

## 6-11. Graphics, desktop, apps, settings, security

После стабильных userland, VFS и device APIs: compositor, IPC, GUI toolkit, desktop shell, приложения, настройки, networking и hardening.

## 12. Milestones

- [x] **M0 Kernel Stable**
- [ ] **M1 Userland**
- [ ] **M2 Storage**
- [ ] **M3 Graphics**
- [ ] **M4 Desktop**
- [ ] **M5 Daily Usable Demo**

## Ближайшие задачи

1. Создать отдельный PML4 и USER/NX mappings для первого процесса.
2. Реализовать минимальный безопасный `syscall/sysret` path: `write`, `exit`, `sleep`.
3. Загрузить первую user-mode программу и изолировать её падение.
4. Затем перенести shell из Ring 0 в user space.
