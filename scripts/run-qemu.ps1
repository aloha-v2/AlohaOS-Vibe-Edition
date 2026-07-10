param(
    [string]$Qemu = "qemu-system-x86_64",
    [string]$OvmfCode
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Esp = Join-Path $Root "esp"
$FirmwareDir = Join-Path $Root "firmware"
$ProjectOvmf = Join-Path $FirmwareDir "OVMF_CODE.fd"
$OvmfUrl = "https://raw.githubusercontent.com/retrage/edk2-nightly/master/bin/RELEASEX64_OVMF.fd"

& (Join-Path $Root "scripts\build.ps1")

# Prefer an explicitly supplied firmware path. Otherwise keep a cached OVMF
# image inside the project and download it on the first run.
if (-not $OvmfCode) {
    $OvmfCode = $ProjectOvmf
}

if (-not (Test-Path $OvmfCode)) {
    if ($OvmfCode -ne $ProjectOvmf) {
        throw "The specified OVMF firmware does not exist: $OvmfCode"
    }

    New-Item -ItemType Directory -Force $FirmwareDir | Out-Null
    $TemporaryFile = "$ProjectOvmf.download"
    Remove-Item $TemporaryFile -Force -ErrorAction SilentlyContinue

    Write-Host "OVMF is missing. Downloading it to firmware\OVMF_CODE.fd..." -ForegroundColor Cyan
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $Client = New-Object System.Net.WebClient
        $Client.Headers.Add("User-Agent", "AlohaOS-Build")
        $Client.DownloadFile($OvmfUrl, $TemporaryFile)
        $Client.Dispose()

        if (-not (Test-Path $TemporaryFile) -or (Get-Item $TemporaryFile).Length -lt 1MB) {
            throw "Downloaded firmware is missing or unexpectedly small."
        }

        Move-Item $TemporaryFile $ProjectOvmf -Force
        Write-Host "OVMF downloaded: $ProjectOvmf" -ForegroundColor Green
    }
    catch {
        Remove-Item $TemporaryFile -Force -ErrorAction SilentlyContinue
        throw "Could not download OVMF from $OvmfUrl. $($_.Exception.Message)"
    }
}
else {
    Write-Host "Using cached OVMF: $OvmfCode" -ForegroundColor DarkGray
}

& $Qemu `
    -machine q35 `
    -m 256M `
    -bios $OvmfCode `
    -drive "format=raw,file=fat:rw:$Esp" `
    -net none `
    -no-reboot

if ($LASTEXITCODE -ne 0) {
    throw "QEMU exited with code $LASTEXITCODE"
}
