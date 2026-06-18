@echo off
echo ========================================
echo Configuring Nginx for WIVAHbank
echo ========================================

:: Define Nginx path (adjust if your installation is elsewhere)
set NGINX_PATH=C:\nginx

:: Adds sites-enabled directory to allow for different projects to have their own nginx configuration files
if not exist "%NGINX_PATH%\conf\sites-enabled" (
    echo Creating sites-enabled directory...
    mkdir "%NGINX_PATH%\conf\sites-enabled"
)

:: Copies the nginx configuration file for WIVAHbank to the sites-enabled directory
echo Copying configuration file...
copy "%~dp0..\deployment\WIVAHbank.conf" "C:\nginx\conf\sites-enabled\WIVAHbank.conf"

echo.
echo ========================================
echo Nginx configuration completed.
echo ========================================
pause
