# CSC1106 Web Programming Project

## Project Name
Banking System

## Tech Stack
- Rust
- Actix Web
- Tera Templates
- PostgreSQL 18
- HTML/CSS

---

## Database Requirement

This project **must use PostgreSQL 18**.

The database scripts and Rust database connection are written for PostgreSQL.

This project is **not designed for MySQL, MariaDB, SQLite, or MongoDB** unless the database code and SQL syntax are rewritten.

Required database setup:

```text
Database system: PostgreSQL 18
Database tool: pgAdmin 4
Database name: banking_system
Username: postgres
Password: 1234
Port: 5432
```

---

## Current Features
- Homepage
- Register page
- Login page
- PostgreSQL database connection
- User registration
- Auto bank account creation after registration
- Login validation using email and password

---

## Current Note About Passwords

Password hashing is currently removed because another teammate will be handling it.

For now, the password is stored inside the `password_hash` column as plain text.

Before final submission, password hashing should be added back using bcrypt.

The file to update later is:

```text
src/services/auth_service.rs
```

---

# Project Structure

```text
CSC1106_Web_Programming_Project/
│
├── Cargo.toml
├── Cargo.lock
├── .env.example
├── README.md
├── setup_db.bat
│
├── migrations/
│   └── 001_create_tables.sql
│
├── static/
│   ├── css/
│   │   └── style.css
│   ├── js/
│   │   └── main.js
│   └── images/
│
├── templates/
│   ├── layout.html
│   ├── login.html
│   ├── register.html
│   └── customer_dashboard.html
│
└── src/
    ├── main.rs
    ├── db.rs
    ├── config.rs
    │
    ├── models/
    │   ├── mod.rs
    │   └── user.rs
    │
    ├── routes/
    │   ├── mod.rs
    │   └── auth_routes.rs
    │
    ├── services/
    │   ├── mod.rs
    │   └── auth_service.rs
    │
    └── middleware/
        ├── mod.rs
        └── auth_middleware.rs
```

---

# Setup Instructions

## 1. Clone the project

```bash
git clone <repo-link>
cd CSC1106_Web_Programming_Project
```

---

## 2. Install requirements

Make sure these are installed:

```text
Rust
PostgreSQL 18
pgAdmin 4
```

---

# Database Setup

## Option 1: Use setup_db.bat

This is the recommended method for Windows.

Run:

```text
setup_db.bat
```

The batch file will:

```text
1. Drop the old banking_system database if it exists
2. Create a new banking_system database
3. Run migrations/001_create_tables.sql
4. Create all required tables
```

Important:

```text
Running setup_db.bat will reset the database and delete existing test data.
```

---

## setup_db.bat content

The batch file uses PostgreSQL 18:

```bat
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
%PSQL% -U %DB_USER% -h localhost -p 5432 -d postgres -c "DROP DATABASE IF EXISTS %DB_NAME%;"

echo.
echo Creating database...
%PSQL% -U %DB_USER% -h localhost -p 5432 -d postgres -c "CREATE DATABASE %DB_NAME%;"

echo.
echo Running migration file...
%PSQL% -U %DB_USER% -h localhost -p 5432 -d %DB_NAME% -f migrations/001_create_tables.sql

echo.
echo ========================================
echo Database setup completed.
echo ========================================
pause
```

---

## Option 2: Manual database setup using pgAdmin 4

Open pgAdmin 4.

Create the database manually:

```sql
CREATE DATABASE banking_system;
```

Then open Query Tool under the `banking_system` database.

Copy and run the SQL from:

```text
migrations/001_create_tables.sql
```

This SQL file is written for **PostgreSQL 18**.

---

# Environment Setup

## 1. Create `.env`

Copy `.env.example` and rename the copy to:

```text
.env
```

The `.env` file should contain:

```env
DATABASE_URL=postgres://postgres:1234@localhost:5432/banking_system
SESSION_KEY=0123456701234567012345670123456701234567012345670123456701234567
```

Do not commit `.env` to GitHub.

---

# Run the Project

Run:

```bash
cargo run
```

If successful, you should see:

```text
Connected to PostgreSQL
Server running at http://127.0.0.1:8080
```

Open in browser:

```text
http://127.0.0.1:8080/
```

---

# Current Pages

| Page | URL |
|---|---|
| Homepage | `/` |
| Login | `/login` |
| Register | `/register` |
| Customer Dashboard | `/dashboard` |

---

# Login/Register Notes

## Login UI

Edit this file:

```text
templates/login.html
```

The login form must keep:

```html
<form method="post" action="/login">
```

The input names must stay as:

```html
<input type="email" name="email">
<input type="password" name="password">
```

---

## Register UI

Edit this file:

```text
templates/register.html
```

The register form must keep:

```html
<form method="post" action="/register">
```

The input names must stay as:

```html
<input type="text" name="name">
<input type="email" name="email">
<input type="text" name="phone_number">
<input type="password" name="password">
```

If these names are changed, Rust will not receive the form data correctly.

---

# Main Files and Purpose

| File | Purpose |
|---|---|
| `src/main.rs` | Starts the Actix Web server |
| `src/db.rs` | Connects Rust to PostgreSQL |
| `src/routes/auth_routes.rs` | Handles homepage, login, register, dashboard routes |
| `src/services/auth_service.rs` | Handles register/login database logic |
| `src/models/user.rs` | Stores register/login form structs |
| `templates/login.html` | Login page UI |
| `templates/register.html` | Register page UI |
| `templates/customer_dashboard.html` | Customer dashboard UI |
| `migrations/001_create_tables.sql` | Creates PostgreSQL 18 database tables |
| `setup_db.bat` | Recreates the PostgreSQL 18 database |

---

# GitHub Rules

Do not commit:

```text
.env
target/
```

Make sure `.gitignore` contains:

```gitignore
/target/
.env
*.pdb
.vscode/
.idea/
Thumbs.db
.DS_Store
*.log
```

Commit these files:

```text
Cargo.toml
Cargo.lock
README.md
.env.example
setup_db.bat
migrations/
src/
templates/
static/
```

---

# Next Features To Build

```text
1. Customer dashboard with real account balance
2. Deposit money
3. Withdraw money
4. Transfer money by account number
5. Transfer money by PayNow phone number
6. Transaction history
7. Loan application
8. Admin dashboard
9. Staff dashboard
10. Audit logs
```