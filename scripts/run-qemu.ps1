param([string]$Qemu="qemu-system-x86_64",[string]$OvmfCode)
$ErrorActionPreference="Stop"
$Root=Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$FirmwareDir=Join-Path $Root "firmware";$ProjectOvmf=Join-Path $FirmwareDir "OVMF_CODE.fd";$OvmfUrl="https://raw.githubusercontent.com/retrage/edk2-nightly/master/bin/RELEASEX64_OVMF.fd"
& (Join-Path $Root "scripts\build.ps1")
if(-not $OvmfCode){$OvmfCode=$ProjectOvmf}
if(-not(Test-Path $OvmfCode)){if($OvmfCode-ne$ProjectOvmf){throw "OVMF not found: $OvmfCode"};New-Item -ItemType Directory -Force $FirmwareDir|Out-Null;$Temp="$ProjectOvmf.download";[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12;$Client=New-Object System.Net.WebClient;$Client.Headers.Add("User-Agent","AlohaOS-Build");$Client.DownloadFile($OvmfUrl,$Temp);$Client.Dispose();if((Get-Item $Temp).Length-lt 1MB){throw "Downloaded OVMF is too small"};Move-Item $Temp $ProjectOvmf -Force}
$MappedDrive=$null;foreach($Letter in @("Z","Y","X","W","V","U","T","S","R","Q","P")){if(-not(Test-Path "${Letter}:\")){$MappedDrive="${Letter}:";break}}
if(-not $MappedDrive){throw "No free drive letter"};& subst.exe $MappedDrive $Root;if($LASTEXITCODE-ne 0){throw "Could not map repository"}
try{$Esp="$MappedDrive\esp";$Firmware=if($OvmfCode-eq$ProjectOvmf){"$MappedDrive\firmware\OVMF_CODE.fd"}else{$OvmfCode};$Disk="$MappedDrive\disk\aloha-fat32.img";$Log="$MappedDrive\qemu.log";Write-Host "Starting AlohaOS with VirtIO FAT32 disk..." -ForegroundColor Cyan;& $Qemu -machine q35 -m 256M -bios $Firmware -drive "format=raw,file=fat:rw:$Esp" -drive "if=none,id=alohadisk,format=raw,file=$Disk" -device "virtio-blk-pci,drive=alohadisk,disable-modern=on" -net none -no-shutdown -d "guest_errors,cpu_reset" -D $Log;if($LASTEXITCODE-ne 0){throw "QEMU exited with code $LASTEXITCODE"}}finally{& subst.exe $MappedDrive /D|Out-Null}
