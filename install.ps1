#!/usr/bin/env pwsh
# MIT license.
# Deno install script (https://github.com/denoland/deno_install/blob/master/install.ps1)

$ErrorActionPreference = 'Stop'

if ($v) {
  $Version = "v${v}"
}
if ($Args.Length -eq 1) {
  $Version = $Args.Get(0)
}

$DenskyInstall = $env:DENO_INSTALL
$BinDir = if ($DenskyInstall) {
  "${DenskyInstall}\bin"
} else {
  "${Home}\.densky\bin"
}

$DenskyZip = "$BinDir\densky.zip"
$DenskyExe = "$BinDir\densky.exe"
$Target = 'windows-x64'

$DownloadUrl = if (!$Version) {
  "https://github.com/Densky-Framework/densky/releases/latest/download/densky-${Target}.zip"
} else {
  "https://github.com/Densky-Framework/densky/releases/download/${Version}/densky-${Target}.zip"
}

if (!(Test-Path $BinDir)) {
  New-Item $BinDir -ItemType Directory | Out-Null
}

curl.exe -Lo $DenskyZip $DownloadUrl

tar.exe xf $DenskyZip -C $BinDir

Remove-Item $DenskyZip

$User = [System.EnvironmentVariableTarget]::User
$Path = [System.Environment]::GetEnvironmentVariable('Path', $User)
if (!(";${Path};".ToLower() -like "*;${BinDir};*".ToLower())) {
  [System.Environment]::SetEnvironmentVariable('Path', "${Path};${BinDir}", $User)
  $Env:Path += ";${BinDir}"
}

Write-Output "Densky was installed successfully to ${DenskyExe}"
Write-Output "Run 'densky --help' to get started"
Write-Output "Stuck? Open an issue on https://github.com/Densky-Framework/densky"
