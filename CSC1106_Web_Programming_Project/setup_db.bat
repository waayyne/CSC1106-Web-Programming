@echo off
echo ========================================
echo Setting up Banking System Database
echo ========================================

set PSQL="C:\Program Files\PostgreSQL\18\bin\psql.exe"
set DB_NAME=banking_system
set DB_USER=postgres
set PGPASSWORD=1234

echo.
echo Dropping old database if it exists...
%PSQL% -U %DB_USER% -h localhost -p 5432 -d postgres -c "DROP DATABASE IF EXISTS %DB_NAME% WITH (FORCE);"

echo.
echo Creating database...
%PSQL% -U %DB_USER% -h localhost -p 5432 -d postgres -c "CREATE DATABASE %DB_NAME%;"

echo.
echo Running main migration file...
%PSQL% -U %DB_USER% -h localhost -p 5432 -d %DB_NAME% -f migrations/001_create_tables.sql

echo.
echo Running profile settings migration file...
%PSQL% -U %DB_USER% -h localhost -p 5432 -d %DB_NAME% -f migrations/002_add_profile_updated_at.sql

echo.
echo Seeding admin user...
%PSQL% -U %DB_USER% -h localhost -p 5432 -d %DB_NAME% -f migrations/003_perma_admin.sql

echo.
echo Running password reset migration file...
%PSQL% -U %DB_USER% -h localhost -p 5432 -d %DB_NAME% -f migrations/004_password_reset_tokens.sql

echo.
echo ========================================
echo Database setup completed.
echo ========================================
pause
