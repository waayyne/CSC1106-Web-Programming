@echo off
set NGINX_PATH=C:\nginx

echo ========================================
echo Starting Full Project Setup...
echo ========================================

:: 1. Database Setup
call batScripts\setup_db.bat

:: 2. Nginx Setup
call batScripts\setup_nginx.bat

:: 3. Start or Reload Nginx
echo.
echo ========================================
echo Starting Nginx...
echo ========================================

cd /d %NGINX_PATH%

nginx.exe -t

if errorlevel 1 (
    echo.
    echo Nginx config error. Please check %NGINX_PATH%\conf\nginx.conf
    pause
    exit /b
)

tasklist /FI "IMAGENAME eq nginx.exe" | find /I "nginx.exe" >nul

if errorlevel 1 (
    echo Nginx is not running. Starting Nginx...
    start "" "%NGINX_PATH%\nginx.exe"
) else (
    echo Nginx is already running. Reloading Nginx...
    nginx.exe -s reload
)

echo.
echo ========================================
echo All setup tasks completed successfully!
echo.
echo Next step:
echo 1. Run: cargo run
echo 2. Open: http://localhost
echo ========================================
pause