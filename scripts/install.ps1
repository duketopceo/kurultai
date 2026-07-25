# One-line PowerShell installer for Kurultai pre-compiled binary
# irm https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.ps1 | iex
$ErrorActionPreference = 'Stop'

$Repo = if ($env:KURULTAI_REPO) { $env:KURULTAI_REPO } else { 'duketopceo/kurultai' }
$InstallDir = if ($env:KURULTAI_INSTALL_DIR) { $env:KURULTAI_INSTALL_DIR } else { "$env:LOCALAPPDATA\Programs\kurultai" }

Write-Host "Installing Kurultai…" -ForegroundColor Cyan

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$ReleaseApi = "https://api.github.com/repos/$Repo/releases/latest"
try {
    $Release = Invoke-RestMethod -Uri $ReleaseApi
} catch {
    Write-Error "No GitHub release yet. Install with: cargo install --git https://github.com/$Repo --locked"
    exit 1
}

$Asset = $Release.assets | Where-Object { $_.name -like '*windows-amd64.zip' } | Select-Object -First 1
if (-not $Asset) {
    Write-Error "Could not find a Windows binary asset. Use: cargo install --git https://github.com/$Repo --locked"
    exit 1
}

$ZipPath = "$env:TEMP\kurultai-windows-amd64.zip"
Write-Host "Downloading $($Asset.name)…" -ForegroundColor Yellow
Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $ZipPath

Write-Host "Extracting to $InstallDir…" -ForegroundColor Yellow
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force
Remove-Item $ZipPath -Force

$UserPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable('PATH', "$UserPath;$InstallDir", 'User')
    Write-Host "Added $InstallDir to User PATH." -ForegroundColor Green
}

Write-Host "Kurultai installed. Open a new terminal and run kurultai --help." -ForegroundColor Green
