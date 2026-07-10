$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

rustup target add x86_64-unknown-uefi x86_64-unknown-none
if ($LASTEXITCODE -ne 0) { throw "rustup failed with exit code $LASTEXITCODE" }

cargo build --release --target x86_64-unknown-uefi -p alohaboot
if ($LASTEXITCODE -ne 0) { throw "AlohaBoot build failed with exit code $LASTEXITCODE" }
cargo build --release --target x86_64-unknown-none -p kernel
if ($LASTEXITCODE -ne 0) { throw "Kernel build failed with exit code $LASTEXITCODE" }

$BootBinary = Join-Path $Root "target\x86_64-unknown-uefi\release\alohaboot.efi"
$KernelBinary = Join-Path $Root "target\x86_64-unknown-none\release\kernel"
$Esp = Join-Path $Root "esp"
$BootDir = Join-Path $Esp "EFI\BOOT"
$KernelDir = Join-Path $Esp "alohaos"
New-Item -ItemType Directory -Force $BootDir | Out-Null
New-Item -ItemType Directory -Force $KernelDir | Out-Null
Copy-Item $BootBinary (Join-Path $BootDir "BOOTX64.EFI") -Force
Copy-Item $KernelBinary (Join-Path $KernelDir "kernel.elf") -Force

$Disk = Join-Path $Root "disk\aloha-fat32.img"
if (-not (Test-Path $Disk)) {
    $Python = Get-Command py -ErrorAction SilentlyContinue
    if ($Python) { & py -3 (Join-Path $Root "scripts\create-fat32.py") $Disk }
    else { & python (Join-Path $Root "scripts\create-fat32.py") $Disk }
    if ($LASTEXITCODE -ne 0) { throw "FAT32 image creation failed" }
}

Write-Host "Built AlohaBoot: $BootDir\BOOTX64.EFI" -ForegroundColor Green
Write-Host "Built kernel:    $KernelDir\kernel.elf" -ForegroundColor Green
Write-Host "FAT32 disk:      $Disk" -ForegroundColor Green
