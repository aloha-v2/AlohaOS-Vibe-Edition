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
 Write-Host "OVMF is missing. Downloading it to firmware\OVMF_CODE.fd..." -ForegroundColor Cyan
 try {
  [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
  $Client = New-Object System.Net.WebClient
  $Client.Headers.Add("User-Agent", "AlohaOS-Build")
  $Client.DownloadFile($OvmfUrl, $TemporaryFile)
  $Client.Dispose()
  if (-not (Test-Path $TemporaryFile) -or (Get-Item $TemporaryFile).Length -lt 1MB) { throw "Downloaded firmware is missing or unexpectedly small." }
  Move-Item $TemporaryFile $ProjectOvmf -Force
 } catch { Remove-Item $TemporaryFile -Force -ErrorAction SilentlyContinue; throw "Could not download OVMF. $($_.Exception.Message)" }
}

$MappedDrive = $null
foreach ($Letter in @("Z","Y","X","W","V","U","T","S","R","Q","P")) { if (-not (Test-Path "${Letter}:\")) { $MappedDrive = "${Letter}:"; break } }
if (-not $MappedDrive) { throw "No free drive letter is available for QEMU." }
& subst.exe $MappedDrive $Root
if ($LASTEXITCODE -ne 0) { throw "Could not map $Root to $MappedDrive" }
try {
 $QemuEsp = "$MappedDrive\esp"
 $QemuOvmf = if ($OvmfCode -eq $ProjectOvmf) { "$MappedDrive\firmware\OVMF_CODE.fd" } else { $OvmfCode }
 Write-Host "Starting QEMU with XSAVE/AVX CPU features..." -ForegroundColor Cyan
 & $Qemu `
  -machine q35 `
  -cpu max `
  -m 256M `
  -bios $QemuOvmf `
  -drive "format=raw,file=fat:rw:$QemuEsp" `
  -net none `
  -no-reboot
 if ($LASTEXITCODE -ne 0) { throw "QEMU exited with code $LASTEXITCODE" }
} finally { & subst.exe $MappedDrive /D | Out-Null }
