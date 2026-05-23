# CSC1106 Web Programming Project

## Project Name
Banking System

## Tech Stack
- Rust
- Actix Web
- Tera Templates
- PostgreSQL
- HTML/CSS

## Current Features
- Homepage
- Register page
- Login page
- PostgreSQL database connection
- User registration
- Auto bank account creation after registration
- Login validation using email and password

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
├── .env
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

## 3. PostgreSQL password

For this project, the PostgreSQL password is assumed to be:

```text
1234
```

The default PostgreSQL user is:

```text
postgres
```

The database name is:

```text
banking_system
```

---

# Database Setup Using Batch File

## 1. Create `setup_db.bat`

Create a file in the project root:

```text
setup_db.bat
```

Paste this inside:

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

## 2. Run the batch file

Double-click:

```text
setup_db.bat
```

Or run in terminal:

```bash
setup_db.bat
```

This will:

```text
1. Drop the old banking_system database if it exists
2. Create a new banking_system database
3. Run migrations/001_create_tables.sql
4. Create all required tables
```

Important:

```text
Running setup_db.bat will delete existing test users and reset the database.
```

---

# If setup_db.bat does not work

If you see this error:

```text
psql is not recognized
```

Check your PostgreSQL folder:

```text
C:\Program Files\PostgreSQL\
```

If your version is not `18`, change this line in `setup_db.bat`:

```bat
set PSQL="C:\Program Files\PostgreSQL\18\bin\psql.exe"
```

Example for PostgreSQL 17:

```bat
set PSQL="C:\Program Files\PostgreSQL\17\bin\psql.exe"
```

---

# Environment Setup

## 1. Create `.env.example`

Create this file:

```text
.env.example
```

Paste this inside:

```env
DATABASE_URL=postgres://postgres:1234@localhost:5432/banking_system
SESSION_KEY=0123456701234567012345670123456701234567012345670123456701234567
```

---

## 2. Create `.env`

Copy `.env.example` and rename the copy to:

```text
.env
```

Your `.env` should contain:

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
| `migrations/001_create_tables.sql` | Creates database tables |

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

# Current Team Task Split

## Login/Register teammate

Edit:

```text
templates/login.html
templates/register.html
src/routes/auth_routes.rs
src/services/auth_service.rs
```

Main job:

```text
Improve login/register UI
Add password hashing back later
Add validation/error messages
```

---

## Customer banking teammate

Next features:

```text
Customer dashboard
Balance display
Deposit money
Withdraw money
```

---

## Transfer teammate

Next features:

```text
Transfer by bank account number
Transfer by PayNow phone number
Transaction history
```

---

## Admin/Staff teammate

Next features:

```text
Admin dashboard
Staff dashboard
Manage users
Manage accounts
Manage loans
Audit logs
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