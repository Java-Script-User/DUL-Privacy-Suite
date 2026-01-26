@echo off
echo Starting Privacy Suite...
echo.

:: Startup banner

net session >nul 2>&1
if %errorLevel% == 0 (
    echo Running with Administrator privileges
) else (
    echo Requesting Administrator privileges...
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b
)

taskkill /F /IM privacy_suite.exe >nul 2>&1

echo Building backend (Rust)...
pushd "%~dp0"
cargo build --release
if %errorLevel% neq 0 (
    echo Backend build failed.
    popd
    exit /b 1
)
popd

echo Building GUI (Tauri)...
pushd "%~dp0gui"
npm run tauri build
if %errorLevel% neq 0 (
    echo GUI build failed.
    popd
    exit /b 1
)
popd

echo Preparing files...
copy /Y "%~dp0target\release\privacy_suite.exe" "%~dp0gui\src-tauri\target\release\privacy_suite.exe" >nul 2>&1

echo Starting Privacy Suite...
start "" "%~dp0gui\src-tauri\target\release\gui.exe"

timeout /t 1 /nobreak >nul

exit /b
