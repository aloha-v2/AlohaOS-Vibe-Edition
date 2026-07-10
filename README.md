# AlohaOS Vibe Edition

AlohaOS is an experimental x86_64 hybrid-kernel operating system written in
Rust. **AlohaBoot** is its custom UEFI bootloader.

## Current milestone: stage 0

The repository currently builds one UEFI image. AlohaBoot initializes the UEFI
runtime, enters the Rust kernel bootstrap and prints:

```text
AlohaOS
```

For stage 0 the `aloha-kernel` crate is linked into AlohaBoot. This keeps the
first boot deterministic and establishes a clean crate boundary. The next boot
milestone is a separate ELF kernel: AlohaBoot will load it from the EFI system
partition, acquire the framebuffer and memory map, exit UEFI boot services,
and transfer control to the kernel. Assembly will only be introduced for CPU
entry, interrupt stubs and context switching where Rust cannot express the ABI.

## Windows prerequisites

1. Install [Rust](https://rustup.rs/).
2. Install [QEMU for Windows](https://www.qemu.org/download/#windows) and add its
   installation directory to `PATH`.
3. Ensure QEMU includes an x86_64 EDK2/OVMF firmware file. Typical locations are
   `C:\Program Files\qemu\share\edk2-x86_64-code.fd` and
   `C:\Program Files\qemu\share\OVMF_CODE.fd`.

## Build

Open PowerShell in the repository root:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\build.ps1
```

The boot image is copied to `esp\EFI\BOOT\BOOTX64.EFI`.

## Run in QEMU

```powershell
.\scripts\run-qemu.ps1
```

If firmware auto-detection fails:

```powershell
.\scripts\run-qemu.ps1 -OvmfCode "C:\Program Files\qemu\share\edk2-x86_64-code.fd"
```

If QEMU is not in `PATH`:

```powershell
.\scripts\run-qemu.ps1 `
  -Qemu "C:\Program Files\qemu\qemu-system-x86_64.exe" `
  -OvmfCode "C:\Program Files\qemu\share\edk2-x86_64-code.fd"
```

QEMU mounts the generated `esp` directory as a FAT EFI system partition. UEFI
finds `EFI\BOOT\BOOTX64.EFI`, starts AlohaBoot, and the display shows `AlohaOS`.
