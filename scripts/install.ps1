# Kurultai Windows installer — uses a GitHub Release when present.
# Usage: irm https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.ps1 | iex
$ErrorActionPreference = 'Stop'

$Repo = if ($env:KURULTAI_REPO) { $env:KURULTAI_REPO } else { 'duketopceo/kurultai' }
$InstallDir = if ($env:KURULTAI_INSTALL_DIR) { $env:KURULTAI_INSTALL_DIR } else { "$env:LOCALAPPDATA\Programs\kurultai" }

Write-Host "Installing kurultai…"

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$ReleaseApi = "https://api.github.com/repos/$Repo/releases/latest"
try {
    $Release = Invoke-RestMethod -Uri $ReleaseApi
} catch {
    Write-Host "No GitHub Release yet. Install with:"
    Write-Host "  cargo install --git https://github.com/$Repo --locked"
    Write-Host "Rust: https://rustup.rs"
    exit 1
}

$Asset = $Release.assets | Where-Object { $_.name -like '*windows-amd64.zip' } | Select-Object -First 1
if (-not $Asset) {
    Write-Host "No Windows binary in latest release. Install with:"
    Write-Host "  cargo install --git https://github.com/$Repo --locked"
    exit 1
}

$ZipPath = "$env:TEMP\kurultai-windows-amd64.zip"
Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force
Remove-Item $ZipPath -Force

$UserPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable('PATH', "$UserPath;$InstallDir", 'User')
}

Write-Host "Installed. Open a new terminal and run: kurultai --help"
