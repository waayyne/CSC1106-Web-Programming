# WIVAH Bank
This project is configured for Windows.
## Description
WIVAH Bank is a CSC1106 Web Programming project using Rust, Actix Web, Tera, PostgreSQL, and Nginx.
It is a simple banking web app with customer, staff, and admin pages. It shows routes, forms, templates, sessions, database queries, and role-based pages.
## Features
- Register, login, logout
- Email verification with OTP
- Forgot password and reset password
- Customer dashboard
- Deposit and withdraw money
- Transfer money with daily transfer limit
- Transaction history and statement page
- Loan application and staff loan approval
- Fixed deposit demo
- Risk investment demo
- Profile settings
- Admin user management
- Audit logs
## Apps Used
- Rust
- Actix Web
- Tera templates
- PostgreSQL
- SQLx
- Nginx
- HTML, CSS, JavaScript
## Requirements
- Rust: https://www.rust-lang.org/tools/install
- PostgreSQL: https://www.postgresql.org/download/
- Nginx: https://nginx.org/en/download.html
## Setup
Open the project folder:
```bat
cd CSC1106_Web_Programming_Project
```
Copy the environment file:
```bat
copy .env.example .env
```
Edit `.env`:
```env
DATABASE_URL=postgres://postgres:your_password@localhost:5432/banking_system
SESSION_KEY=your_64_character_session_key
APP_BASE_URL=http://localhost
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_gmail_app_password
SMTP_FROM=your_email@gmail.com
```
For Gmail, use a Gmail App Password for `SMTP_PASSWORD`. If email is not tested, dummy SMTP values can be used, but OTP and reset email may not work.
## Database Setup
For Windows, the easiest way is:
```bat
setup_all.bat
```
This creates or resets the database, runs migrations, and sets up or reloads Nginx.
After it finishes, run `cargo run` to start the Rust backend. It also resets the database if it already exists.
Manual database setup:
```bat
createdb banking_system
psql -U postgres -d banking_system -f migrations/001_create_tables.sql
psql -U postgres -d banking_system -f migrations/002_add_profile_updated_at.sql
psql -U postgres -d banking_system -f migrations/003_perma_users.sql
psql -U postgres -d banking_system -f migrations/004_password_reset_tokens.sql
psql -U postgres -d banking_system -f migrations/005_email_verification_otps.sql
psql -U postgres -d banking_system -f migrations/006_add_daily_transfer_limit.sql
```
## Nginx Setup
Nginx lets the project open at `http://localhost`.
On Windows, extract Nginx to:
```text
C:\nginx
```
The folder should contain:
```text
C:\nginx\nginx.exe
C:\nginx\conf\
C:\nginx\html\
C:\nginx\logs\
```
It should not be nested like:
```text
C:\nginx\nginx-1.xx.x\
```
If it is nested, move the files inside `nginx-1.xx.x` directly into `C:\nginx`.
Edit:
```text
C:\nginx\conf\nginx.conf
```
The simple way is to open `nginxConfFilesetup.md`, copy everything inside it, remove the old content in `C:\nginx\conf\nginx.conf`, and paste the copied content there.
That setup already includes this line inside the `http { }` block:
```nginx
include C:/nginx/conf/sites-enabled/*.conf;
```
The project has this Nginx config:
```text
deployment/WIVAHbank.conf
```
`setup_all.bat` or `setup_nginx.bat` copies it to:
```text
C:\nginx\conf\sites-enabled\WIVAHbank.conf
```
`WIVAHbank.conf` proxies localhost port 80 to the Rust backend at:
```text
http://127.0.0.1:8080
```
Test Nginx:
```bat
cd /d C:\nginx
nginx.exe -t
```
Start Nginx manually:
```bat
start nginx.exe
```
Stop Nginx manually:
```bat
nginx.exe -s stop
```
## Running the Project
Normal way:
```bat
start_all.bat
```
Then open:
```text
http://localhost
```
If not using Nginx:
```bat
cargo run
```
Then open:
```text
http://127.0.0.1:8080
```
## Stopping the Project
Stop the Rust server with `Ctrl+C`.
Stop Nginx:
```bat
cd /d C:\nginx
nginx.exe -s stop
```
Or run:
```bat
stop_all.bat
```
## Default Accounts
Default accounts are created in `migrations/003_perma_users.sql`.
```text
Admin:
Username: BankAdmin
Password: SpideyBank

Staff:
Username: BankStaff
Password: Staff123

Customer:
Username: BankUser
Password: Guest123
```
## Main Pages
```text
/                  Home
/login             Login
/register          Register
/verify-email      Email verification
/forgot-password   Forgot password
/reset-password    Reset password
/dashboard         Customer dashboard
/atm               Deposit and withdraw
/transfer          Transfer money
/transactions      Transaction history
/loans             Loans
/fixed-deposit     Fixed deposit
/risk-investment   Risk investment
/profile           Profile settings
/staff/dashboard   Staff dashboard
/staff/loans       Staff loans
/admin/dashboard   Admin dashboard
/admin/logs        Audit logs
```
## Notes
- Fixed deposit uses seconds so it can be tested during demo.
- Risk investment is a simple percentage-based profit or loss simulation.
- Transfers use PostgreSQL transactions and row locking.
- If email is not configured, email-related features may not work.
## Troubleshooting
If `http://localhost` does not work, test Nginx:
```bat
cd /d C:\nginx
nginx.exe -t
```
Check that this file exists:
```text
C:\nginx\conf\sites-enabled\WIVAHbank.conf
```
If `http://127.0.0.1:8080` works but `http://localhost` does not, the Rust app is running but Nginx is probably not started or not loading the config.
If the database does not connect, check PostgreSQL and `DATABASE_URL` in `.env`. There should be no spaces around `=`.
Correct:
```env
DATABASE_URL=postgres://postgres:password@localhost:5432/banking_system
```
Wrong:
```env
DATABASE_URL = postgres://postgres:password@localhost:5432/banking_system
```