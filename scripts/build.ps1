$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

rustup target add x86_64-unknown-uefi
cargo build --release --target x86_64-unknown-uefi -p aloha-boot

$Esp = Join-Path $Root "esp"
$BootDir = Join-Path $Esp "EFI\BOOT"
New-Item -ItemType Directory -Force $BootDir | Out-Null
Copy-Item `
    (Join-Path $Root "target\x86_64-unknown-uefi\release\aloha-boot.efi") `
    (Join-Path $BootDir "BOOTX64.EFI") `
    -Force

Write-Host "Built: $BootDir\BOOTX64.EFI" -ForegroundColor Green
