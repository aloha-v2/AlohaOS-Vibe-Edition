param(
 [string]$Qemu = "qemu-system-x86_64",
 [string]$OvmfCode
)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$FirmwareDir = Join-Path $Root "firmware"
$ProjectOvmf = Join-Path $FirmwareDir "OVMF_CODE.fd"
$OvmfUrl = "https://raw.githubusercontent.com/retrage/edk2-nightly/master/bin/RELEASEX64_OVMF.fd"
& (Join-Path $Root "scripts\build.ps1")
if (-not $OvmfCode) { $OvmfCode = $ProjectOvmf }
if (-not (Test-Path $OvmfCode)) {
 if ($OvmfCode -ne $ProjectOvmf) { throw "The specified OVMF firmware does not exist: $OvmfCode" }
 New-Item -ItemType Directory -Force $FirmwareDir | Out-Null
 $TemporaryFile = "$ProjectOvmf.download"
 Remove-Item $TemporaryFile -Force -ErrorAction SilentlyContinue
 [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
 $Client = New-Object System.Net.WebClient
 $Client.Headers.Add("User-Agent", "AlohaOS-Build")
 $Client.DownloadFile($OvmfUrl, $TemporaryFile)
 $Client.Dispose()
 if ((Get-Item $TemporaryFile).Length -lt 1MB) { throw "Downloaded OVMF is unexpectedly small." }
 Move-Item $TemporaryFile $ProjectOvmf -Force
}
$MappedDrive = $null
foreach ($Letter in @("Z","Y","X","W","V","U","T","S","R","Q","P")) { if (-not (Test-Path "${Letter}:\")) { $MappedDrive = "${Letter}:"; break } }
if (-not $MappedDrive) { throw "No free drive letter is available for QEMU." }
& subst.exe $MappedDrive $Root
if ($LASTEXITCODE -ne 0) { throw "Could not map $Root to $MappedDrive" }
$SerialLog = Join-Path $Root "qemu-serial.log"
$DebugLog = Join-Path $Root "qemu-debug.log"
Remove-Item $SerialLog,$DebugLog -Force -ErrorAction SilentlyContinue
try {
 $QemuEsp = "$MappedDrive\esp"
 $QemuOvmf = if ($OvmfCode -eq $ProjectOvmf) { "$MappedDrive\firmware\OVMF_CODE.fd" } else { $OvmfCode }
 $QemuSerial = "$MappedDrive\qemu-serial.log"
 $QemuDebug = "$MappedDrive\qemu-debug.log"
 Write-Host "Starting QEMU with crash logging..." -ForegroundColor Cyan
 & $Qemu `
  -machine q35 `
  -cpu max `
  -m 256M `
  -bios $QemuOvmf `
  -drive "format=raw,file=fat:rw:$QemuEsp" `
  -serial "file:$QemuSerial" `
  -d "guest_errors,cpu_reset" `
  -D $QemuDebug `
  -net none `
  -no-reboot
 $ExitCode = $LASTEXITCODE
 Write-Host "QEMU exit code: $ExitCode" -ForegroundColor Yellow
 if (Test-Path $SerialLog) { Write-Host "`n=== SERIAL ===" -ForegroundColor Cyan; Get-Content $SerialLog }
 if (Test-Path $DebugLog) { Write-Host "`n=== DEBUG TAIL ===" -ForegroundColor Cyan; Get-Content $DebugLog -Tail 80 }
 if ($ExitCode -ne 0) { throw "QEMU crashed. Send qemu-serial.log and qemu-debug.log." }
}
finally { & subst.exe $MappedDrive /D | Out-Null }
