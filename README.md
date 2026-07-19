# AlohaOS Vibe Edition

Экспериментальная x86_64 ОС на Rust с собственным UEFI bootloader и `no_std` kernel.

## Готово

- M0 Kernel Stable, Ring 3, real syscalls, ELF execution и process scheduling.
- User #UD и #PF изолируются от kernel.
- W^X реально проверяется: запись в executable read-only code page завершает только Process.
- Unmapped stack guard реально проверяется user write fault.
- Bad syscall pointer возвращает EFAULT, user process продолжает работу и cleanly exits.
- Sleep/wait сохраняют syscall frame и возобновляются с результатом в RAX.
- Валидные non-SYSRET frames возвращаются через sanitised `iretq` fallback.
- Spawn владеет ресурсами atomically: registry + address space + ELF, с полным rollback при любом сбое (PID и фреймы не текут).
- Per-process handle table и file syscalls `open/read/close/stat` поверх read-only FAT32.
- Anonymous user `mmap`: zero-filled pages, bounded allocation, NX/W^X validation и rollback.
- Rust `no_std` user runtime `aloha-user` с typed wrappers для ABI v1, errno decoding и entry/panic macros.
- Protection, suspended-resume, spawn-rollback, handle-table и mmap suites проходят в CI.

## Текущий этап: M1 Userland

Базовый memory layer и runtime готовы. Сейчас собираем настоящий user-space shell: запуск ELF из FAT32 через spawn, аргументы командной строки, `ls/cat` и корректный wait/exit flow.

Подробный статус и следующий порядок работ: [TODO.md](TODO.md).

## Сборка user runtime

Runtime лежит в `user/runtime` и собирается как отдельный `no_std` crate для `x86_64-unknown-none`:

```rust
#![no_std]
#![no_main]

use aloha_user::{entry, panic_handler, write};

fn main() -> i32 {
    let _ = write(b"hello from Ring 3\\n");
    0
}

entry!(main);
panic_handler!();
```

Текущий ABI использует syscall v1. Kernel принимает fixed-address ELF64, а load segments обязаны находиться в user region и соблюдать W^X.

## Лицензия

PolyForm Noncommercial License 1.0.0. См. [LICENSE.md](LICENSE.md).
