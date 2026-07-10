# AlohaOS Roadmap

Цель: стабильная x86_64 ОС с user space, графическим сервером, рабочим столом, GUI-приложениями и настройками.

Порядок важен. Не начинайте desktop до завершения этапов 1-5.

## 1. Стабильность ядра

- [ ] Полный x86_64 context switch: GPR, RIP, RSP, RFLAGS, CR3, FS/GS, XSAVE/XRSTOR. **Реализован, automated QEMU smoke пройден, ожидает ручную проверку.**
- [x] Lifecycle задач: Ready, Running, Blocked, Sleeping, Dead.
- [x] Отдельный kernel stack и guard page для background task.
- [ ] Стабильный preemptive round-robin и stress-test. **Smoke test пройден, длительный stress-test впереди.**
- [ ] Spinlock, mutex, semaphore, wait queue и IRQ-safe locking.
- [ ] Убрать глобальные `static mut` из горячих подсистем.
- [ ] Освобождение физических фреймов.
- [x] Kernel log, COM1 и severity.
- [ ] Symbol table/backtrace для panic screen.
- [ ] QEMU tests: exceptions, heap, scheduler, disk, keyboard. **Boot/scheduler smoke добавлен.**

**Готово, когда:** несколько задач работают час без Double Fault, утечек и зависаний.

## 2. ACPI, APIC и современное железо

- [ ] RSDP в `BootInfo`, RSDT/XSDT, MADT, FADT, HPET и MCFG.
- [ ] Local APIC + I/O APIC, APIC timer/HPET.
- [ ] SMP bootstrap, ACPI reboot/shutdown, PCIe enumeration.

## 3. Ring 3 и syscalls

- [ ] Ring-3 descriptors, TSS RSP0 и отдельный PML4.
- [ ] USER/NX pages, independent address spaces.
- [ ] `syscall/sysret`, user-pointer validation и базовые syscalls.
- [ ] File descriptors, ELF loader и user-space shell.

## 4. VFS и storage

- [ ] VFS API, FAT32 driver, LFN и подкаталоги.
- [ ] FAT32 write/create/truncate/delete/rename.
- [ ] Block cache, flushing, crash-safe writes и `/tmp` RAM FS.

## 5. Devices

- [ ] Device manager, improved VirtIO Block, mouse/input.
- [ ] VirtIO GPU, double buffering, EDID, RTC.

## 6-11

После этапов 1-5: graphics, compositor, IPC, GUI toolkit, desktop, apps, settings и security.

## Ближайшие задачи

1. Ручная проверка `tasks` и merge PR #4.
2. Часовой scheduler stress-test.
3. IRQ-safe synchronization primitives.
4. Physical frame deallocation.
