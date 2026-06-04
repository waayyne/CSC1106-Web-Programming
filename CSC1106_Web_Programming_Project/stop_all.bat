@echo off
echo ========================================
echo Stopping WIVAHbank Development Environment
echo ========================================

:: Stop Nginx gracefully
echo Stopping Nginx...
cd C:\nginx
nginx.exe -s stop 2>nul

:: Then return to your project directory (using the script's original path)
cd %~dp0..

:: Stop Rust backend (by killing the process using port 8080)
:: Token 5 is the PID (Process ID) in the netstat output
echo Stopping Rust backend...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :8080') do (
    taskkill /f /pid %%a >nul 2>&1
)

echo.
echo All processes stopped. You may now close this terminal.
echo ========================================
pause