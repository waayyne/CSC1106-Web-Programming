@echo off
echo ========================================
echo Starting WIVAH Bank Project
echo ========================================

echo.
echo Checking Nginx...

tasklist /FI "IMAGENAME eq nginx.exe" | find /I "nginx.exe" >nul

if %ERRORLEVEL%==0 (
    echo Nginx is already running.
) else (
    echo Starting Nginx...
    cd /d C:\nginx
    start nginx.exe
    echo Nginx started.
)

echo.
echo Starting Rust backend...

cd /d "%~dp0"

cargo run

pause