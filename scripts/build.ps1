$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

rustup target add x86_64-unknown-uefi
if ($LASTEXITCODE -ne 0) { throw "rustup failed with exit code $LASTEXITCODE" }

cargo build --release --target x86_64-unknown-uefi -p alohaboot
if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }

$Binary = Join-Path $Root "target\x86_64-unknown-uefi\release\alohaboot.efi"
if (-not (Test-Path $Binary)) {
    throw "UEFI binary was not produced: $Binary"
}

$Esp = Join-Path $Root "esp"
$BootDir = Join-Path $Esp "EFI\BOOT"
New-Item -ItemType Directory -Force $BootDir | Out-Null
Copy-Item $Binary (Join-Path $BootDir "BOOTX64.EFI") -Force

Write-Host "Built: $BootDir\BOOTX64.EFI" -ForegroundColor Green
