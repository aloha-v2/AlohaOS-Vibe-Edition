# AlohaOS Roadmap

## 1. Стабильность ядра

- [ ] Полный x86_64 context switch: прототип XSAVE/CR3/FS/GS выявил Double Fault и временно gated off.
- [x] Lifecycle задач: Ready, Running, Blocked, Sleeping, Dead.
- [x] Guarded kernel stack infrastructure.
- [ ] Preemptive round-robin и часовой stress-test.
- [ ] Переписать switch path как строгий assembly trampoline на dedicated scheduler/IST stack.
- [ ] Spinlock, mutex, semaphore, wait queue и IRQ-safe locking.
- [ ] Убрать `static mut` из горячих подсистем.
- [ ] Освобождение физических фреймов.
- [x] COM1 kernel log и severity.
- [ ] Backtrace для panic screen.
- [ ] QEMU tests: exceptions, heap, scheduler, disk, keyboard.

**Важно:** hardware test на Windows/QEMU поймал Double Fault. Пункт scheduler не считается выполненным до стабильного stress-test.

## 2. ACPI/APIC

- [ ] RSDP, ACPI tables, Local APIC, I/O APIC, HPET, SMP, reboot/shutdown, PCIe.

## 3. Ring 3

- [ ] User descriptors, TSS RSP0, user PML4, syscalls, ELF loader и user shell.

## 4. VFS

- [ ] VFS API, writable FAT32, LFN, subdirectories, cache и crash-safe writes.

## 5. Devices

- [ ] Device manager, improved VirtIO, mouse, GPU, EDID и RTC.

## Ближайшие задачи

1. Dedicated scheduler interrupt stack.
2. Assembly-only context switch trampoline.
3. Hardware stress-test без Double Fault.
4. IRQ-safe synchronization.
