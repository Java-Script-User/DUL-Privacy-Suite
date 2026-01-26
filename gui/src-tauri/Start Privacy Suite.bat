@echo off
title Privacy Suite - Starting Services
echo Starting Privacy Suite with Administrator privileges...
echo.

:: Elevation

net session >nul 2>&1
if %errorLevel% == 0 (
    goto :start_services
) else (
    echo Requesting Administrator privileges...
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit
)

:start_services
echo Running with Administrator privileges
echo.

echo Stopping any existing Privacy Suite processes...
taskkill /F /IM privacy_suite.exe >nul 2>&1
taskkill /F /IM "Privacy Suite.exe" >nul 2>&1

timeout /t 1 /nobreak >nul

echo Starting Privacy Suite backend...
cd /d "%~dp0"
start /B "" "%~dp0privacy_suite.exe"

echo Initializing network security layers...
timeout /t 5 /nobreak >nul

netstat -ano | findstr :3030 >nul
if %errorLevel% == 0 (
    echo Backend started successfully on port 3030
) else (
    echo WARNING: Backend may not have started properly
)

echo.
echo Starting GUI...
start "" "%~dp0Privacy Suite.exe"

echo.
echo Privacy Suite is now running!
echo This window will close in 2 seconds...
timeout /t 2 /nobreak >nul

exit
