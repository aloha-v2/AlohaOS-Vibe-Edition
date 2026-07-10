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

if (-not (Test-Path $BootBinary)) {
    throw "UEFI binary was not produced: $BootBinary"
}
if (-not (Test-Path $KernelBinary)) {
    throw "Kernel ELF was not produced: $KernelBinary"
}

$Esp = Join-Path $Root "esp"
$BootDir = Join-Path $Esp "EFI\BOOT"
$KernelDir = Join-Path $Esp "alohaos"
New-Item -ItemType Directory -Force $BootDir | Out-Null
New-Item -ItemType Directory -Force $KernelDir | Out-Null

Copy-Item $BootBinary (Join-Path $BootDir "BOOTX64.EFI") -Force
Copy-Item $KernelBinary (Join-Path $KernelDir "kernel.elf") -Force

Write-Host "Built AlohaBoot: $BootDir\BOOTX64.EFI" -ForegroundColor Green
Write-Host "Built kernel:    $KernelDir\kernel.elf" -ForegroundColor Green
