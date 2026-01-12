# Stop all running processes
Write-Host "Stopping all processes..." -ForegroundColor Yellow
Get-Process -Name "gui" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "privacy_suite" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 3

# Verify processes are stopped
$retries = 0
while ((Get-Process -Name "privacy_suite" -ErrorAction SilentlyContinue) -and $retries -lt 5) {
    Write-Host "Waiting for privacy_suite to stop..." -ForegroundColor Yellow
    Start-Sleep -Seconds 1
    $retries++
}

if (Get-Process -Name "privacy_suite" -ErrorAction SilentlyContinue) {
    Write-Host "Error: Could not stop privacy_suite.exe. Please close it manually." -ForegroundColor Red
    exit 1
}

# Build with Tauri CLI (builds frontend + bundles it properly) in RELEASE mode
Write-Host "Building Tauri app (release mode)..." -ForegroundColor Cyan
Set-Location gui
npm run tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tauri build failed!" -ForegroundColor Red
    Set-Location ..
    exit 1
}

Set-Location ..

# Copy backend to GUI directory
Write-Host "Copying backend..." -ForegroundColor Cyan
Copy-Item "target\release\privacy_suite.exe" "gui\src-tauri\target\release\privacy_suite.exe" -Force -ErrorAction SilentlyContinue

Write-Host "`nBuild complete!" -ForegroundColor Green
Write-Host "Launch with: gui\src-tauri\target\release\gui.exe" -ForegroundColor Yellow
Write-Host "Or run: .\launch_standalone.bat" -ForegroundColor Yellow
