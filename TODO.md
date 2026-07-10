# AlohaOS Roadmap

## 1. Стабильность ядра

- [ ] Полный x86_64 context switch: hardware smoke и автоматический 60s stress пройдены, часовой прогон впереди.
- [x] Lifecycle задач: Ready, Running, Blocked, Sleeping, Dead.
- [x] Guarded kernel stack infrastructure.
- [ ] Preemptive round-robin и часовой stress-test; runtime gate, hardware smoke и 60s CI stress готовы.
- [x] Dedicated scheduler/timer IST stack, проверен на Windows/QEMU.
- [x] Assembly-only extended-context trampoline (CR3, FS/GS, XSAVE/XRSTOR).
- [x] Persistent per-task GPR/IRET frames для переключения с timer IST.
- [x] Runtime gate `sched on|off`, по умолчанию выключен.
- [x] Hardware smoke: обе задачи 588 switches, worker heartbeat 588, shell и FAT32 живы.
- [x] Автоматический 60-секундный QEMU scheduler stress.
- [ ] Ручной workflow `Scheduler one-hour stress` должен пройти 3600 секунд.
- [x] IRQ-safe spinlock primitive и миграция COM1 logger.
- [ ] Mutex, semaphore и wait queue.
- [ ] Мигрировать heap/device shared state на IRQ-safe primitives.
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
- Gated round-robin: task 0 и task 1 по 588 switches, worker heartbeat 588.
- Автоматический 60s stress проходит без Double Fault.
- VirtIO Block и FAT32 online после включения scheduler.
- `ls /` видит `HELLO.TXT`, `cat hello.txt` читает файл.

## Ближайшие задачи

1. Проверить IRQ-safe spinlock через build, smoke и scheduler stress.
2. Мигрировать heap lock на общий primitive.
3. Запустить `Scheduler one-hour stress`.
4. Mutex, semaphore, wait queue и frame deallocation.
