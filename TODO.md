# AlohaOS Roadmap

Цель: стабильная x86_64 ОС с user space, графическим сервером, рабочим столом, GUI-приложениями и настройками.

Порядок важен. Не начинайте desktop до завершения этапов 1-5: иначе GUI будет работать поверх нестабильного ядра и любое приложение сможет обрушить всю ОС.

## 1. Стабильность ядра

- [x] Реализовать полный x86_64 context switch: GPR/IRET, CR3, FS/GS base, FPU/SSE/AVX через XSAVE/XRSTOR.
- [x] Реализовать lifecycle задач: Ready, Running, Blocked, Sleeping, Dead.
- [x] Добавить отдельный kernel stack для каждой задачи и guard page от переполнения.
- [x] Сделать стабильный preemptive round-robin scheduler и stress-test переключений: hardware smoke, 60s CI и 3600s QEMU stress пройдены без Double Fault.
- [ ] Включить preemptive round-robin по умолчанию.
- [ ] Добавить spinlock, mutex, semaphore, wait queue и IRQ-safe locking. IRQ-safe spinlock и миграция COM1 уже готовы.
- [ ] Мигрировать heap/device shared state на IRQ-safe primitives.
- [ ] Убрать глобальные `static mut` из горячих подсистем.
- [ ] Реализовать нормальное освобождение физических фреймов.
- [x] Добавить kernel log, serial output в COM1 и уровни log severity.
- [ ] Добавить symbol table/backtrace для panic screen.
- [ ] Создать полный QEMU test suite: boot, exceptions, heap, scheduler, disk, keyboard. Build, boot/timer/FAT32 smoke и scheduler stress уже работают.

**Готово, когда:** несколько задач работают час без Double Fault, утечек и зависаний. Часовой scheduler stress пройден; проверки утечек и остальные тесты ещё впереди.

### Выполнено

- COM1 logger работает без heap, пишет boot/panic с severity и защищён IRQ-safe spinlock.
- Lifecycle хранится атомарно; поддержаны block, wake, timed sleep и exit.
- PIT переводит Sleeping в Ready при достижении wake tick.
- Для каждой задачи мапится отдельный guarded kernel stack.
- Dedicated scheduler/timer IST stack проверен на Windows/QEMU.
- Assembly-only trampoline сохраняет persistent GPR/IRET frame, CR3, FS/GS и XSAVE state.
- Runtime gate `sched on|off` работает.
- Hardware smoke: обе задачи по 588 switches, worker heartbeat 588, shell и FAT32 живы.
- Автоматические 60s и 3600s scheduler stress-тесты прошли без Double Fault, panic и storage errors.

## 2. ACPI, APIC и современное железо

- [ ] Передавать RSDP из AlohaBoot в `BootInfo`.
- [ ] Парсить RSDT/XSDT, MADT, FADT, HPET и MCFG.
- [ ] Перейти с PIC 8259 на Local APIC + I/O APIC.
- [ ] Добавить APIC timer или HPET и точные monotonic clocks.
- [ ] Реализовать SMP bootstrap для дополнительных CPU.
- [ ] Реализовать ACPI reboot и shutdown.
- [ ] Добавить PCI/PCIe enumeration через MCFG.

**Готово, когда:** таймер, IRQ routing, reboot и shutdown работают без legacy-костылей.

## 3. Процессы, Ring 3 и системные вызовы

- [ ] Добавить ring-3 code/data descriptors и TSS `RSP0`.
- [ ] Создавать отдельный PML4 для каждого процесса.
- [ ] Мапить user pages с флагами USER и NX.
- [ ] Реализовать copy-on-write или хотя бы независимые address spaces.
- [ ] Настроить `syscall/sysret` с безопасной проверкой адресов.
- [ ] Начальные syscalls: `exit`, `write`, `read`, `open`, `close`, `stat`, `mmap`, `sleep`, `spawn`, `wait`.
- [ ] Добавить handles/file descriptors и таблицу ресурсов процесса.
- [ ] Реализовать ELF loader для user-space программ.
- [ ] Перенести shell из Ring 0 в user space.
- [ ] Изолировать падение приложения от ядра и других процессов.

**Готово, когда:** user-программа печатает текст, читает файл, падает и не валит kernel.

## 4. VFS и постоянная файловая система

- [ ] Создать VFS API: inode, file, directory, mount point и path resolver.
- [ ] Подключить FAT32 как VFS driver.
- [ ] Добавить чтение подкаталогов и Long File Names.
- [ ] Реализовать FAT32 write, create, truncate, delete и rename.
- [ ] Добавить block cache и dirty-page flushing.
- [ ] Сделать защиту от повреждения при сбое записи.
- [ ] Добавить RAM filesystem для `/tmp`.
- [ ] Определить дерево: `/bin`, `/apps`, `/system`, `/users`, `/tmp`, `/devices`.
- [ ] Добавить права доступа и владельцев хотя бы в минимальном виде.
- [ ] В перспективе перейти на журналируемую собственную FS или ext2/ext4 read/write.

**Готово, когда:** файлы и настройки переживают reboot.

## 5. Драйверная модель и базовые устройства

- [ ] Создать device manager и единый driver API.
- [ ] Улучшить VirtIO Block: interrupts, несколько запросов, write и flush.
- [ ] Добавить VirtIO Input или полноценный PS/2 mouse driver.
- [ ] Добавить клавиатурные layouts, modifiers, repeat и Unicode input.
- [ ] Добавить VirtIO GPU для QEMU вместо зависимости только от UEFI framebuffer.
- [ ] Реализовать double buffering и page flipping.
- [ ] Добавить EDID, выбор resolution и display modes.
- [ ] Добавить RTC и wall-clock time.
- [ ] Добавить VirtIO Network и базовый network stack позже.
- [ ] Добавить audio driver позже, лучше начать с Intel HDA или VirtIO Sound.

**Готово, когда:** мышь, клавиатура и framebuffer/GPU доступны через стабильные kernel APIs.

## 6. Графический фундамент

- [ ] Создать 2D graphics library: pixel, line, rectangle, rounded rectangle, image blit.
- [ ] Перейти с 8x8 bitmap font на TrueType/OpenType rasterizer.
- [ ] Добавить UTF-8, Unicode, font fallback и text measurement.
- [ ] Реализовать RGBA surfaces и alpha blending.
- [ ] Добавить clipping regions и damage tracking.
- [ ] Реализовать compositor с back buffer.
- [ ] Добавить hardware/software cursor.
- [ ] Вынести display server/compositor в отдельный user-space процесс.
- [ ] Определить IPC-протокол окон: create, resize, draw, input, close.
- [ ] Не давать приложениям прямой доступ к framebuffer.

**Готово, когда:** два изолированных процесса рисуют окна, перемещают их мышью и получают собственные события.

## 7. IPC и GUI toolkit

- [ ] Реализовать kernel IPC: message queues или channels.
- [ ] Добавить shared memory для передачи оконных buffers без копирования.
- [ ] Создать event loop для user-space приложений.
- [ ] Сделать GUI toolkit: Window, View, Label, Button, TextBox, Checkbox, Slider, List, Menu.
- [ ] Добавить layout engine: row, column, grid, padding, alignment.
- [ ] Добавить focus, tab navigation, shortcuts и clipboard.
- [ ] Определить theme API: colors, typography, spacing, radii, icons.
- [ ] Добавить accessibility metadata хотя бы на уровне semantic roles.
- [ ] Сделать стабильный application ABI/API и SDK crate.

**Готово, когда:** одно приложение собирается отдельно от kernel и использует toolkit без прямой работы с пикселями.

## 8. Рабочий стол

- [ ] Реализовать login/session manager или пока автологин одного пользователя.
- [ ] Сделать desktop shell отдельным user-space процессом.
- [ ] Добавить wallpaper, desktop icons и context menu.
- [ ] Добавить panel/taskbar, launcher, список окон, tray и clock.
- [ ] Реализовать window decorations: title bar, minimize, maximize, close, resize.
- [ ] Добавить virtual desktops после стабилизации одного desktop.
- [ ] Сделать notifications service.
- [ ] Добавить file picker и common dialogs.
- [ ] Реализовать drag-and-drop.
- [ ] Сохранять layout и desktop preferences в `/users/default/settings`.

**Готово, когда:** пользователь запускает приложение, переключает окна и перезагружает ОС без потери настроек.

## 9. Базовые GUI-приложения

- [ ] Settings: display, theme, keyboard, mouse, time, storage, system info.
- [ ] File Manager: folders, copy, move, rename, delete, properties.
- [ ] Terminal: запуск shell в user space, history и scrollback.
- [ ] Text Editor: open/save, selection, clipboard, undo/redo.
- [ ] System Monitor: CPU, RAM, processes, uptime и disks.
- [ ] Image Viewer.
- [ ] Calculator.
- [ ] About AlohaOS.
- [ ] App launcher и application manifests.
- [ ] Package/install format только после стабилизации VFS и process ABI.

## 10. Настройки и конфигурация

- [ ] Определить versioned settings format, например UTF-8 key/value или компактный binary format.
- [ ] Создать settings service вместо прямой записи каждым приложением.
- [ ] Разделить system settings и per-user settings.
- [ ] Добавить атомарную запись через temporary file + rename.
- [ ] Добавить defaults, validation и migration между версиями.
- [ ] Сделать live notifications об изменении theme/language/display.
- [ ] Добавить reset-to-default и recovery mode.

## 11. Безопасность и надёжность

- [ ] Включить NX, SMEP, SMAP и write-protect там, где CPU поддерживает.
- [ ] Сделать W^X: страница не должна быть одновременно writable и executable.
- [ ] Проверять все user pointers в syscalls.
- [ ] Добавить process capabilities/permissions.
- [ ] Разделить drivers, services и applications по привилегиям.
- [ ] Добавить watchdog и crash reports.
- [ ] Реализовать safe mode и recovery shell.
- [ ] Сделать fuzz tests для ELF, FAT32, path parser и IPC messages.

## 12. Рекомендуемые milestone-релизы

- [ ] **M0 Kernel Stable:** exceptions, memory, full scheduler, tests. Scheduler готов; остальные M0 проверки ещё впереди.
- [ ] **M1 Userland:** Ring 3, syscalls, ELF apps, user shell.
- [ ] **M2 Storage:** VFS, writable FAT32, persistent settings.
- [ ] **M3 Graphics:** mouse, VirtIO GPU, compositor, windows.
- [ ] **M4 Desktop:** panel, launcher, file manager, terminal.
- [ ] **M5 Daily Usable Demo:** settings, editor, networking basics, installer/image builder.

## Ближайшие задачи

1. Включить preemptive round-robin по умолчанию.
2. Мигрировать heap lock на общий IRQ-safe primitive.
3. Добавить mutex, semaphore и wait queue.
4. Реализовать physical frame deallocation.
5. Расширить QEMU tests: exceptions, heap, disk и keyboard.
