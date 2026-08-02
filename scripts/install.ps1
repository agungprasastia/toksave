# TokSave installer for Windows (PowerShell)
# Usage: irm https://raw.githubusercontent.com/agungprasastia/toksave/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$repo = "agungprasastia/toksave"
$target = "windows-x64"
$asset = "toksave-${target}.zip"
$installDir = "$env:LOCALAPPDATA\Programs\toksave"

Write-Host "Installing toksave ($target)..." -ForegroundColor Cyan

# Download
$url = "https://github.com/$repo/releases/latest/download/$asset"
$tmpDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
$zipPath = Join-Path $tmpDir $asset

Write-Host "  Downloading $url..."
Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

# Extract
Write-Host "  Extracting..."
Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

# Install
New-Item -ItemType Directory -Path $installDir -Force | Out-Null
$exeFile = Get-ChildItem -Path $tmpDir -Filter "toksave.exe" -Recurse | Select-Object -First 1
if (-not $exeFile) {
    throw "toksave.exe not found in downloaded archive"
}
Copy-Item -Path $exeFile.FullName -Destination (Join-Path $installDir "toksave.exe") -Force

# Clean up
Remove-Item -Recurse -Force $tmpDir

Write-Host ""
Write-Host "  ✔ Installed to $installDir\toksave.exe" -ForegroundColor Green

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$installDir;$userPath", "User")
    Write-Host "  ✔ Added $installDir to user PATH" -ForegroundColor Green
    Write-Host "  ⚠ Restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "  Run 'toksave' to get started." -ForegroundColor Cyan
