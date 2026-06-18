# WIVAH Bank — CSC1106 Web Programming Project

WIVAH Bank is a banking web app we built for CSC1106. It has normal banking features such as login, deposits, withdrawals, transfers, loans, fixed deposits, risk investment simulation, and separate dashboards for customers, staff, and admins.

Built with Rust, Actix Web, Tera Templates, PostgreSQL, and Nginx.

---

## Roles

**Customer** — register/login, verify email, deposit/withdraw/transfer money, view transaction history, export statement, apply for loans, use fixed deposit and risk investment simulation, update profile

**Staff** — review and manage customer loan applications

**Admin** — manage users and accounts, view audit logs

---

## How transfers work

Transfers use a database transaction with row-level locking on the sender's account so two transfers cannot read the same balance at the same time. If anything fails during the transfer, it rolls back.

---

## Installation and Running Instructions

Follow the steps below in order to install and run the project.

### 1. Install these first

* Rust → https://www.rust-lang.org/tools/install
* PostgreSQL → https://www.postgresql.org/download/
* Nginx → https://nginx.org/en/download.html

For Windows, extract Nginx and put it at:

```text
C:\nginx
```

Make sure these are directly inside `C:\nginx`:

```text
nginx.exe
conf\
html\
logs\
```

Do not leave them inside another nested folder like:

```text
C:\nginx\nginx-1.31.1\
```

### 2. Open the project folder

Extract the project ZIP, then open the project folder:

```bash
cd CSC1106_Web_Programming_Project
```

### 3. Set up `.env`

Copy the example environment file.

Windows:

```bat
copy .env.example .env
```

Mac/Linux:

```bash
cp .env.example .env
```

Fill in `.env`:

```env
DATABASE_URL=postgres://postgres:your_password@localhost:5432/banking_system
SESSION_KEY=your_64_character_session_key
APP_BASE_URL=http://localhost
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_gmail_app_password
SMTP_FROM=your_email@gmail.com
TURNSTILE_SITE_KEY=your_turnstile_site_key
TURNSTILE_SECRET_KEY=your_turnstile_secret_key
```

For Gmail, `SMTP_PASSWORD` has to be an App Password, not your actual password:

```text
https://myaccount.google.com/apppasswords
```

If email or Turnstile is not being tested, dummy values can be used, but the related features may not work.

### 4. Database

#### Windows

Run:

```bat
setup_all.bat
```

This creates the database, runs all migration files, sets up Nginx, and starts it.

Note: this resets the database if it already exists.

#### Mac/Linux

Create the database:

```bash
createdb banking_system
```

Run the migration files:

```bash
psql -U postgres -d banking_system -f migrations/001_create_tables.sql
psql -U postgres -d banking_system -f migrations/002_add_profile_updated_at.sql
psql -U postgres -d banking_system -f migrations/003_perma_users.sql
psql -U postgres -d banking_system -f migrations/004_password_reset_tokens.sql
psql -U postgres -d banking_system -f migrations/005_email_verification_otps.sql
psql -U postgres -d banking_system -f migrations/006_add_daily_transfer_limit.sql
```

### 5. Nginx

#### Windows

A full Nginx config file is provided in:

```text
nginxConfFilesetup.md
```

Open `nginxConfFilesetup.md`, copy everything inside it, and paste it into:

```text
C:\nginx\conf\nginx.conf
```

Save the file.

Then run:

```bat
-----------------------------------------------------------
Sets up Nginx and runs Nginx while also setting up database
-----------------------------------------------------------
setup_all.bat
```

The script copies `WIVAHbank.conf` to the correct Nginx folder and starts Nginx.

To test/start/stop manually:

```bat
cd /d C:\nginx
nginx.exe -t
start nginx.exe
nginx.exe -s stop
```

#### Mac

```bash
mkdir -p /opt/homebrew/etc/nginx/servers
cp deployment/WIVAHbank.conf /opt/homebrew/etc/nginx/servers/WIVAHbank.conf
nginx -t && brew services restart nginx
```

Make sure `nginx.conf` has this inside the `http { }` block:

```nginx
include servers/*;
```

#### Linux

```bash
sudo cp deployment/WIVAHbank.conf /etc/nginx/sites-available/WIVAHbank.conf
sudo ln -s /etc/nginx/sites-available/WIVAHbank.conf /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

### 6. Run

Start the Rust server:

```bash
cargo run
```
or

```text
Run the start_all.bat
```

Open:

```text
http://localhost
```

If Nginx is not set up yet, use:

```text
http://localhost:8080
```

### 7. Stop

Rust server:

```text
Ctrl+C
```

Nginx on Windows:

```bat
nginx.exe -s stop
```

Or run:

```bat
stop_all.bat
```

Nginx on Mac:

```bash
brew services stop nginx
```

Nginx on Linux:

```bash
sudo systemctl stop nginx
```

### 8. Starting again

```text
- After closing the server with Cntrl + C and stopping ngix server
- To start the server again without starting the database again

1) Run start_all.bat to initiate rust and nginx
2) "Terminate batch job (Y/N) will appear after pressing Cntrl + C, press yes
3) Run stop_all.bat to close nginx as well as any other processes still running

```


---


## Pages

```text
/                  Homepage
/login             Login
/register          Register
/verify-email      Email verification
/forgot-password   Forgot password
/dashboard         Customer dashboard
/account           Account info
/atm               Deposit / withdraw
/transfer          Transfer money
/transactions      Transaction history
/loans             Loans
/profile           Profile settings
/admin             Admin dashboard
/staff             Staff dashboard
/audit-logs        Audit logs
```

---

## Default User accounts

Created by the migration file:

```text
Admin:
Username: BankAdmin
Email: admin@bank.com
Password: check migrations/003_perma_users.sql
```

```text
Staff:
Username: BankStaff
Email: staff@bank.com
Password: check migrations/003_perma_users.sql
```

```text
Customer:
Username: BankUser
Email: user@bank.com
Password: check migrations/003_perma_users.sql
```

You can also change the passwords of the demo users in 003_perma_users.sql before running the project.

---

## Common issues

### `localhost` does not work but `localhost:8080` works

Nginx is not running or the config is not loaded correctly.

Check that `WIVAHbank.conf` is in the correct folder and reload Nginx.

### Database connection error

Make sure PostgreSQL is running and `DATABASE_URL` in `.env` is correct.

### Nginx config error

Run this on Windows:

```bat
cd /d C:\nginx
nginx.exe -t
```

Run this on Mac/Linux:

```bash
nginx -t
```

### Port 80 or 8080 already in use

Something else may already be using the port.

Windows:

```bat
netstat -ano | findstr :80
netstat -ano | findstr :8080
```

Mac/Linux:

```bash
sudo lsof -i :80
sudo lsof -i :8080
```

### SQLx `.env` error

Check that there are no spaces around `=` in the `.env` file.

Correct:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/banking_system
```

Wrong:

```env
DATABASE_URL = postgres://postgres:password@localhost:5432/banking_system
```

---

## `.gitignore` reminder

Do not commit `.env`.

Commit `.env.example`.
