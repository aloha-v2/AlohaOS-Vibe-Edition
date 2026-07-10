# AlohaOS Roadmap

## 1. Стабильность ядра

- [ ] Полный x86_64 context switch: XSAVE/CR3/FS/GS prototype gated off до hardware stress-test.
- [x] Lifecycle задач: Ready, Running, Blocked, Sleeping, Dead.
- [x] Guarded kernel stack infrastructure.
- [ ] Preemptive round-robin и часовой stress-test; runtime gate реализован, hardware test впереди.
- [x] Dedicated scheduler/timer IST stack, проверен на Windows/QEMU.
- [x] Assembly-only extended-context trampoline (CR3, FS/GS, XSAVE/XRSTOR).
- [x] Persistent per-task GPR/IRET frames для переключения с timer IST.
- [x] Runtime gate `sched on|off`, по умолчанию выключен.
- [ ] Spinlock, mutex, semaphore, wait queue и IRQ-safe locking.
- [ ] Убрать `static mut` из горячих подсистем.
- [ ] Освобождение физических фреймов.
- [x] COM1 kernel log и severity.
- [ ] Backtrace для panic screen.
- [ ] Полный QEMU test suite; boot/timer/FAT32 smoke уже работает.

## 2. ACPI/APIC

- [ ] RSDP, ACPI tables, Local APIC, I/O APIC, HPET, SMP, reboot/shutdown и PCIe.

## 3. Ring 3

- [ ] User descriptors, TSS RSP0, user PML4, syscalls, ELF loader и user shell.

## 4. VFS

- [ ] VFS API, writable FAT32, LFN, subdirectories, cache и crash-safe writes.

## 5. Devices

- [ ] Device manager, improved VirtIO, mouse, GPU, EDID и RTC.

## Проверено на Windows/QEMU

- Shell загружается без panic.
- Timer IST стабилен, scheduling ticks растут.
- VirtIO Block и FAT32 online.
- `ls /` видит `HELLO.TXT`, `cat hello.txt` читает файл.

## Ближайшие задачи

1. Hardware smoke: `sched on`, `tasks`, `ls /`, `cat hello.txt`.
2. Часовой scheduler stress-test без Double Fault.
3. IRQ-safe synchronization.
4. Physical frame deallocation.
