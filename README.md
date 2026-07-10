# AlohaOS — Vibe Edition

Миниатюрная операционная система на Rust (с щепоткой ассемблера там, где иначе никак).
Пока что это загрузочный скелет: при старте на экране появляется надпись **AlohaOS**.

## Архитектура

| Компонент     | Значение                                   |
| ------------- | ------------------------------------------ |
| Bootloader    | **AlohaBoot** (собственный UEFI-загрузчик) |
| Firmware      | UEFI                                       |
| Architecture  | x86_64                                     |
| Language      | Rust                                       |
| Kernel        | Hybrid                                     |

### Как это работает

1. **AlohaBoot** (`boot/`, таргет `x86_64-unknown-uefi`) — UEFI-приложение. Оно:
   - получает линейный фреймбуфер через GOP (Graphics Output Protocol);
   - читает ядро `\alohaos\kernel.elf` с загрузочного тома (ESP);
   - раскладывает LOAD-сегменты ELF по их физическим адресам;
   - вызывает `ExitBootServices` и прыгает в точку входа ядра (ABI `sysv64`),
     передавая указатель на `BootInfo` в `rdi`.
2. **kernel** (`kernel/`, таргет `x86_64-unknown-none`, `no_std`) — получает
   `BootInfo`, заливает экран фоном, рисует «AlohaOS» встроенным 8x8 битмап-шрифтом
   и уходит в `hlt`.
3. **common** (`common/`) — общий контракт `BootInfo` между загрузчиком и ядром.

```
┌─────────────┐   GOP fb + kernel ELF    ┌────────────┐
│  AlohaBoot  │ ───────────────────────▶ │   kernel   │  →  "AlohaOS"
│ (UEFI .efi) │   jmp entry(&BootInfo)   │ (bare ELF) │
└─────────────┘                          └────────────┘
```

## Что понадобится

```sh
rustup target add x86_64-unknown-uefi x86_64-unknown-none
# + qemu-system-x86_64 и прошивка OVMF (UEFI для QEMU)
```

## Сборка и запуск

```sh
# Собрать загрузчик и ядро, разложить ESP и запустить в QEMU:
make run OVMF=/путь/к/OVMF_CODE.fd

# Только собрать артефакты:
make boot kernel

# Собрать ESP-каталог без запуска:
make esp
```

Если `make` недоступен, то же самое руками:

```sh
cargo build -p alohaboot --target x86_64-unknown-uefi
cargo build -p kernel    --target x86_64-unknown-none

mkdir -p esp/EFI/BOOT esp/alohaos
cp target/x86_64-unknown-uefi/debug/alohaboot.efi esp/EFI/BOOT/BOOTX64.EFI
cp target/x86_64-unknown-none/debug/kernel        esp/alohaos/kernel.elf

qemu-system-x86_64 \
  -machine q35 -m 256M \
  -drive if=pflash,format=raw,readonly=on,file=/путь/к/OVMF_CODE.fd \
  -drive format=raw,file=fat:rw:esp
```

## Дорожная карта

- [x] Загрузка и вывод сплэша «AlohaOS»
- [ ] GDT / IDT, обработка исключений
- [ ] Пейджинг и физический аллокатор
- [ ] Ввод с клавиатуры, простой шелл
