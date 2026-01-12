@echo off
echo Starting Privacy Suite...
echo.

:: Check for admin rights
net session >nul 2>&1
if %errorLevel% == 0 (
    echo Running with Administrator privileges
) else (
    echo Requesting Administrator privileges...
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b
)

:: Kill any existing backend processes
taskkill /F /IM privacy_suite.exe >nul 2>&1

:: Copy backend to GUI directory so it can be found
echo Preparing files...
copy /Y "%~dp0target\release\privacy_suite.exe" "%~dp0gui\src-tauri\target\release\privacy_suite.exe" >nul 2>&1

:: Start the GUI application (it will auto-start the backend)
echo Starting Privacy Suite...
start "" "%~dp0gui\src-tauri\target\release\gui.exe"

:: Wait a moment for GUI to start
timeout /t 1 /nobreak >nul

:: Close this command window
exit /b
