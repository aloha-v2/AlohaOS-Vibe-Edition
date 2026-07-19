# aloha-user

Minimal `no_std` Rust runtime for AlohaOS Ring 3 programs.

It exposes typed wrappers for ABI v1 syscalls and two macros for freestanding binaries:

```rust
#![no_std]
#![no_main]

use aloha_user::{entry, panic_handler, write};

fn main() -> i32 {
    let _ = write(b"hello from Ring 3\n");
    0
}

entry!(main);
panic_handler!();
```

Build user programs for `x86_64-unknown-none`. The kernel currently accepts fixed-address ELF64 executables whose load segments live inside the AlohaOS user region and obey W^X.
