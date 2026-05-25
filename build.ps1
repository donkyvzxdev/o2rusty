$projectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$releasesDir = Join-Path $projectDir "releases"
$exePath = Join-Path $projectDir "target\release\o2jam-rust.exe"

if (-not (Test-Path $releasesDir)) {
    New-Item -ItemType Directory -Path $releasesDir -Force | Out-Null
}

# Backup existing executable if it exists
if (Test-Path $exePath) {
    $timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm"
    $backupName = "o2jam-rust_$timestamp.exe"
    $backupPath = Join-Path $releasesDir $backupName
    Copy-Item -Path $exePath -Destination $backupPath -Force
    Write-Host "Backup saved: $backupName"
}

# Build release
Write-Host "Building release..."
Set-Location -Path $projectDir
cargo build --release 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful! Executable: $exePath"
} else {
    Write-Host "Build failed!" -ForegroundColor Red
}
