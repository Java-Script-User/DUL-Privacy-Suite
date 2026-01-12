# DUL Privacy Suite - Release Build Script
# Run as Administrator

Write-Host "DUL Privacy Suite - Release Builder" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan
Write-Host ""

$VERSION = "1.0.0"
$PROJECT_NAME = "DUL-Privacy-Suite"
$RELEASE_DIR = "release"

# Stop any running processes
Write-Host "Stopping running processes..." -ForegroundColor Yellow
Get-Process privacy_suite,gui -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

# Clean previous release
Write-Host "Cleaning previous release..." -ForegroundColor Yellow
if (Test-Path $RELEASE_DIR) {
    Remove-Item $RELEASE_DIR -Recurse -Force
}
New-Item -ItemType Directory -Path $RELEASE_DIR -Force | Out-Null

# Build the application
Write-Host "`nBuilding application..." -ForegroundColor Yellow
cd gui
npm run tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
cd ..

# Check if build succeeded
$GUI_EXE = "gui\src-tauri\target\release\gui.exe"
$BACKEND_EXE = "gui\src-tauri\target\release\privacy_suite.exe"

if (-not (Test-Path $GUI_EXE)) {
    Write-Host "Error: GUI executable not found!" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $BACKEND_EXE)) {
    Write-Host "Error: Backend executable not found!" -ForegroundColor Red
    exit 1
}

# Create portable version
Write-Host "`nCreating portable version..." -ForegroundColor Yellow
$PORTABLE_DIR = "$RELEASE_DIR\$PROJECT_NAME-Portable"
New-Item -ItemType Directory -Path $PORTABLE_DIR -Force | Out-Null

Copy-Item $GUI_EXE -Destination $PORTABLE_DIR\
Copy-Item $BACKEND_EXE -Destination $PORTABLE_DIR\
Copy-Item "README.md" -Destination "$PORTABLE_DIR\README.txt"
Copy-Item "LICENSE" -Destination "$PORTABLE_DIR\LICENSE.txt"

# Create quick start guide
@"
DUL Privacy Suite v$VERSION - Quick Start
==========================================

1. Run gui.exe as Administrator
2. Click the power button to connect
3. Your traffic is now protected!

Features:
- Tor network routing
- Tracker blocking  
- WebRTC leak protection
- IPv6 leak protection
- Kill switch

For more information, see README.txt

Support: https://github.com/yourusername/dul-privacy-suite
"@ | Out-File -FilePath "$PORTABLE_DIR\QUICK_START.txt" -Encoding UTF8

# Create ZIP file
Write-Host "Creating ZIP archive..." -ForegroundColor Yellow
$ZIP_NAME = "$PROJECT_NAME-v$VERSION-Windows-x64.zip"
Compress-Archive -Path "$PORTABLE_DIR\*" -DestinationPath "$RELEASE_DIR\$ZIP_NAME" -Force

# Copy MSI installer if it exists
$MSI_PATH = "gui\src-tauri\target\release\bundle\msi\*.msi"
if (Test-Path $MSI_PATH) {
    Write-Host "Copying MSI installer..." -ForegroundColor Yellow
    $MSI_FILES = Get-ChildItem $MSI_PATH
    foreach ($MSI in $MSI_FILES) {
        Copy-Item $MSI.FullName -Destination "$RELEASE_DIR\$PROJECT_NAME-v$VERSION-Setup.msi"
    }
}

# Generate checksums
Write-Host "`nGenerating checksums..." -ForegroundColor Yellow
$CHECKSUM_FILE = "$RELEASE_DIR\SHA256SUMS.txt"
Get-ChildItem "$RELEASE_DIR\*.zip", "$RELEASE_DIR\*.msi" | ForEach-Object {
    $hash = Get-FileHash -Algorithm SHA256 $_.FullName
    $filename = $_.Name
    "$($hash.Hash)  $filename" | Out-File -FilePath $CHECKSUM_FILE -Append -Encoding UTF8
}

# Display results
Write-Host "`n" -ForegroundColor Green
Write-Host "===========================================" -ForegroundColor Green
Write-Host "   Build Complete!                        " -ForegroundColor Green
Write-Host "===========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Release files created in: $RELEASE_DIR" -ForegroundColor Cyan
Write-Host ""
Get-ChildItem $RELEASE_DIR | ForEach-Object {
    $size = if ($_.PSIsContainer) { "DIR" } else { "{0:N2} MB" -f ($_.Length / 1MB) }
    Write-Host "  $($_.Name.PadRight(50)) $size" -ForegroundColor White
}
Write-Host ""
Write-Host "Checksums:" -ForegroundColor Cyan
Get-Content $CHECKSUM_FILE | ForEach-Object {
    Write-Host "  $_" -ForegroundColor Gray
}
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Test the portable version in: $PORTABLE_DIR" -ForegroundColor White
Write-Host "2. Upload to GitHub Releases or your website" -ForegroundColor White
Write-Host "3. Include SHA256SUMS.txt for verification" -ForegroundColor White
Write-Host ""
