param(
    [string]$Qemu = "qemu-system-x86_64",
    [string]$OvmfCode = $env:OVMF_CODE
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

& (Join-Path $Root "scripts\build.ps1")

if (-not $OvmfCode) {
    $Candidates = @(
        "C:\Program Files\qemu\share\edk2-x86_64-code.fd",
        "C:\Program Files\qemu\share\OVMF_CODE.fd",
        "C:\Program Files\qemu\share\edk2-x86_64-secure-code.fd"
    )
    $OvmfCode = $Candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}

if (-not $OvmfCode -or -not (Test-Path $OvmfCode)) {
    throw "OVMF firmware not found. Pass -OvmfCode 'C:\path\to\OVMF_CODE.fd' or set OVMF_CODE."
}

& $Qemu `
    -machine q35 `
    -m 256M `
    -bios $OvmfCode `
    -drive "format=raw,file=fat:rw:$Root\esp" `
    -net none `
    -no-reboot
