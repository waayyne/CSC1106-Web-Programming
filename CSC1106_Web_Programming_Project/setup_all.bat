@echo off
echo ========================================
echo Starting Full Project Setup...
echo ========================================

:: 1. Database Setup
call batScripts\setup_db.bat

:: 2. Nginx Setup
call batScripts\setup_nginx.bat

echo.
echo ========================================
echo All tasks completed successfully!
echo 1. Start backend: cargo run
echo 2. Start Nginx: start nginx (or nginx -s reload)
echo ========================================
pause