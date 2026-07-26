# Kurultai Windows installer.
# Usage: irm https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.ps1 | iex
$ErrorActionPreference = 'Stop'

$Repo = if ($env:KURULTAI_REPO) { $env:KURULTAI_REPO } else { 'duketopceo/kurultai' }
$InstallDir = if ($env:KURULTAI_INSTALL_DIR) { $env:KURULTAI_INSTALL_DIR } else { "$env:LOCALAPPDATA\Programs\kurultai" }
$GitUrl = "https://github.com/$Repo"

Write-Host "Installing kurultai…"

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$ReleaseApi = "https://api.github.com/repos/$Repo/releases/latest"
$Release = $null
try {
    $Release = Invoke-RestMethod -Uri $ReleaseApi
} catch {
    $Release = $null
}

$Tag = if ($Release) { $Release.tag_name } else { $null }
$Asset = if ($Release) {
    $Release.assets | Where-Object { $_.name -eq 'kurultai-windows-amd64.zip' } | Select-Object -First 1
} else {
    $null
}

if (-not $Asset) {
    Write-Host "No Windows binary on latest GitHub Release — use cargo:"
    if ($Tag) {
        Write-Host "  cargo install --git $GitUrl --tag $Tag --locked --force"
    } else {
        Write-Host "  cargo install --git $GitUrl --locked --force"
    }
    Write-Host "Rust: https://rustup.rs"
    exit 1
}

$ZipPath = "$env:TEMP\kurultai-windows-amd64.zip"
Write-Host "Downloading $($Asset.name) ($Tag)…"
Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force
Remove-Item $ZipPath -Force

$UserPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable('PATH', "$UserPath;$InstallDir", 'User')
}

Write-Host "Installed $InstallDir\kurultai.exe"
Write-Host "Open a new terminal, then: kurultai init && kurultai daemon --port 8421"
