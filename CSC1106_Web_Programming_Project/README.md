# WIVAH Bank — Banking System

CSC1106 Web Programming Project

WIVAH Bank is a banking web application built using **Rust**, **Actix Web**, **Tera Templates**, **PostgreSQL**, and **Nginx**.

The application includes user authentication, customer banking features, staff/admin features, transactions, loan management, password reset, and audit logs.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| Web Framework | Actix Web |
| Template Engine | Tera Templates |
| Database | PostgreSQL |
| Frontend | HTML, CSS, JavaScript |
| Reverse Proxy | Nginx |

---

## Important URL Explanation

The Rust Actix Web server runs on port `8080`.

Direct Actix access:

```text
http://localhost:8080
```

Nginx reverse proxy access:

```text
http://localhost
```

Nginx listens on port `80` and forwards the request to the Actix server at:

```text
http://127.0.0.1:8080
```

For the final project setup, use:

```text
http://localhost
```

---

## Project Structure

```text
CSC1106_Web_Programming_Project/
├── .env
├── .env.example
├── Cargo.toml
├── Cargo.lock
├── README.md
├── setup_all.bat
├── stop_all.bat
├── nginxConfFileSetup.md
├── batScripts/
│   ├── setup_db.bat
│   └── setup_nginx.bat
├── deployment/
│   └── WIVAHbank.conf
├── migrations/
│   ├── 001_create_tables.sql
│   ├── 002_add_profile_updated_at.sql
│   ├── 003_perma_admin.sql
│   └── 004_password_reset_tokens.sql
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── db.rs
│   ├── middleware/
│   ├── models/
│   ├── routes/
│   └── services/
├── static/
│   ├── css/
│   └── js/
└── templates/
```

---

# Windows Installation

## 1. Install Rust

Download and install Rust from:

```text
https://www.rust-lang.org/tools/install
```

After installation, check that Rust is installed:

```bat
rustc --version
cargo --version
```

---

## 2. Install PostgreSQL

Download and install PostgreSQL from:

```text
https://www.postgresql.org/download/windows/
```

During installation, remember your PostgreSQL password. The default PostgreSQL username is usually:

```text
postgres
```

This project’s Windows setup script assumes:

```text
PostgreSQL version: 18
PostgreSQL path: C:\Program Files\PostgreSQL\18\bin\psql.exe
Username: postgres
Password: 1234
Database name: banking_system
```

If your PostgreSQL password is not `1234`, edit this file before running the setup script:

```text
batScripts\setup_db.bat
```

Change this line:

```bat
set PGPASSWORD=1234
```

If your PostgreSQL version or installation folder is different, also update this line:

```bat
set PSQL="C:\Program Files\PostgreSQL\18\bin\psql.exe"
```

---

## 3. Install Nginx on Windows

Download Nginx from the official website:

```text
https://nginx.org/en/download.html
```

Use the Windows mainline version.

After downloading, you will get a `.zip` file, for example:

```text
nginx-1.31.1.zip
```

### 3.1 Extract the ZIP file

Right-click the `.zip` file and choose:

```text
Extract All...
```

After extracting, you may get a folder like:

```text
nginx-1.31.1
```

### 3.2 Rename and move the Nginx folder

The final Nginx path must be:

```text
C:\nginx
```

Inside `C:\nginx`, you should directly see:

```text
nginx.exe
conf
html
logs
```

Correct structure:

```text
C:\nginx\nginx.exe
C:\nginx\conf
C:\nginx\html
C:\nginx\logs
```

Wrong structure:

```text
C:\nginx\nginx-1.31.1\nginx.exe
```

If your folder looks like the wrong structure, open:

```text
C:\nginx\nginx-1.31.1
```

Then move everything inside it up one level into:

```text
C:\nginx
```

After moving, delete the empty `nginx-1.31.1` folder.


Then try moving the files again.

### 3.3 Test the Nginx folder

Open Command Prompt and run:

```bat
cd /d C:\nginx
nginx.exe -v
nginx.exe -t
```

If the test is successful, Nginx is in the correct location.

---

## 4. Clone the Repository

```bat
git clone <repo-link>
cd CSC1106_Web_Programming_Project
```

---

## 5. Configure Nginx

### 5.1 Replace the main Nginx config file

Open:

```text
C:\nginx\conf\nginx.conf
```

Then open this file from the project folder:

```text
CSC1106_Web_Programming_Project\nginxConfFileSetup.md
```

Copy everything from `nginxConfFileSetup.md` and paste it into:

```text
C:\nginx\conf\nginx.conf
```

Save and close the file.

This is needed because the project Nginx setup uses this line:

```nginx
include C:/nginx/conf/sites-enabled/*.conf;
```

This allows Nginx to load project-specific config files from:

```text
C:\nginx\conf\sites-enabled
```

### 5.2 Project Nginx config

The project-specific Nginx config file is already included in the project at:

```text
deployment\WIVAHbank.conf
```

You do not need to copy this file manually if you are using the automated setup. The `setup_all.bat` script will run `batScripts\setup_nginx.bat`, which creates the `sites-enabled` folder and copies `WIVAHbank.conf` into the correct Nginx folder.

---

## 6. Configure Environment Variables

Copy `.env.example` and rename the copy to `.env`.

Windows Command Prompt:

```bat
copy .env.example .env
```

Open `.env` and update the values.

Example:

```env
DATABASE_URL=postgres://postgres:1234@localhost:5432/banking_system
SESSION_KEY=0123456701234567012345670123456701234567012345670123456701234567
APP_BASE_URL=http://localhost
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-gmail-app-password
SMTP_FROM=your-email@gmail.com
```

Replace `1234` with your PostgreSQL password.

Because Nginx is used, keep:

```env
APP_BASE_URL=http://localhost
```

Do not commit `.env` to GitHub.

For Gmail SMTP, `SMTP_PASSWORD` must be a Gmail App Password, not your normal Gmail password.

---

## 7. Run the Automated Windows Setup

After Nginx is extracted to `C:\nginx`, `nginx.conf` is replaced, and `.env` is created, run:

```bat
setup_all.bat
```

This runs:

```bat
batScripts\setup_db.bat
batScripts\setup_nginx.bat
```

The setup script will:

1. Drop and recreate the `banking_system` database.
2. Run the migration files.
3. Create `C:\nginx\conf\sites-enabled` if it does not exist.
4. Copy `deployment\WIVAHbank.conf` to `C:\nginx\conf\sites-enabled\WIVAHbank.conf`.

Warning: this resets the database and deletes existing data.

Important: `setup_all.bat` handles the project-specific Nginx config, but it does not replace the main Nginx config file at `C:\nginx\conf\nginx.conf`. You must copy the content from `nginxConfFileSetup.md` into `nginx.conf` first.

---

## 8. Run the Project on Windows

### 8.1 Start the Rust server

In the project folder, run:

```bat
cargo run
```

If successful, the terminal should show something similar to:

```text
Connected to PostgreSQL
Server running at http://localhost:8080
```

Test direct Actix access first:

```text
http://localhost:8080
```

### 8.2 Start Nginx

Open another Command Prompt as Administrator:

```bat
cd /d C:\nginx
start nginx.exe
```

Then open:

```text
http://localhost
```

Nginx runs in the background. You only need to start it again after restarting your computer or after stopping Nginx.

If you change `nginx.conf` or `WIVAHbank.conf`, reload Nginx:

```bat
cd /d C:\nginx
nginx.exe -s reload
```

---

## 9. Stop the Project on Windows

Stop the Rust server by pressing:

```text
Ctrl + C
```

Stop Nginx:

```bat
cd /d C:\nginx
nginx.exe -s stop
```

If Nginx does not stop:

```bat
taskkill /F /IM nginx.exe
```

Alternatively, run:

```bat
stop_all.bat
```

---

# macOS / Linux Installation

The Windows `.bat` setup files are only for Windows. On macOS/Linux, set up PostgreSQL and Nginx manually.

## 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Restart the terminal, then check:

```bash
rustc --version
cargo --version
```

---

## 2. Install PostgreSQL

### macOS using Homebrew

```bash
brew install postgresql
brew services start postgresql
```

### Ubuntu/Debian Linux

```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

---

## 3. Install Nginx

### macOS using Homebrew

```bash
brew install nginx
brew services start nginx
```

### Ubuntu/Debian Linux

```bash
sudo apt update
sudo apt install nginx
sudo systemctl start nginx
sudo systemctl enable nginx
```

---

## 4. Clone the Repository

```bash
git clone <repo-link>
cd CSC1106_Web_Programming_Project
```

---

## 5. Configure Environment Variables

```bash
cp .env.example .env
```

Open `.env` and update the values.

Example:

```env
DATABASE_URL=postgres://postgres:1234@localhost:5432/banking_system
SESSION_KEY=0123456701234567012345670123456701234567012345670123456701234567
APP_BASE_URL=http://localhost
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-gmail-app-password
SMTP_FROM=your-email@gmail.com
```

Update the PostgreSQL username and password based on your local setup.

Because Nginx is used, keep:

```env
APP_BASE_URL=http://localhost
```

---

## 6. Set Up the Database

Create the database:

```bash
createdb banking_system
```

If `createdb` does not work, use `psql`:

```bash
psql -U postgres
```

Then run:

```sql
CREATE DATABASE banking_system;
\q
```

Run the migration files in order:

```bash
psql -U postgres -d banking_system -f migrations/001_create_tables.sql
psql -U postgres -d banking_system -f migrations/002_add_profile_updated_at.sql
psql -U postgres -d banking_system -f migrations/003_perma_admin.sql
psql -U postgres -d banking_system -f migrations/004_password_reset_tokens.sql
```

---

## 7. Configure Nginx on macOS/Linux

The project config file is:

```text
deployment/WIVAHbank.conf
```

It should forward:

```text
http://localhost
```

to:

```text
http://127.0.0.1:8080
```

### macOS Homebrew Nginx

Apple Silicon Homebrew Nginx is usually located at:

```text
/opt/homebrew/etc/nginx/
```

Intel Mac Homebrew Nginx is usually located at:

```text
/usr/local/etc/nginx/
```

Create a `servers` folder if needed:

```bash
mkdir -p /opt/homebrew/etc/nginx/servers
```

Copy the project config:

```bash
cp deployment/WIVAHbank.conf /opt/homebrew/etc/nginx/servers/WIVAHbank.conf
```

Make sure the main Nginx config includes this line inside the `http { ... }` block:

```nginx
include servers/*;
```

Test and restart Nginx:

```bash
nginx -t
brew services restart nginx
```

### Ubuntu/Debian Linux Nginx

Copy the project config:

```bash
sudo cp deployment/WIVAHbank.conf /etc/nginx/sites-available/WIVAHbank.conf
```

Enable it:

```bash
sudo ln -s /etc/nginx/sites-available/WIVAHbank.conf /etc/nginx/sites-enabled/WIVAHbank.conf
```

Test and reload Nginx:

```bash
sudo nginx -t
sudo systemctl reload nginx
```

---

## 8. Run the Project on macOS/Linux

Start the Rust server:

```bash
cargo run
```

Test direct Actix access:

```text
http://localhost:8080
```

Then test Nginx access:

```text
http://localhost
```

---

## 9. Stop the Project on macOS/Linux

Stop the Rust server by pressing:

```text
Ctrl + C
```

Stop Nginx on macOS:

```bash
brew services stop nginx
```

Stop Nginx on Linux:

```bash
sudo systemctl stop nginx
```

---

# Main Pages

| Page | URL |
|---|---|
| Homepage | `/` |
| Login | `/login` |
| Register | `/register` |
| Customer Dashboard | `/dashboard` |
| Account Details | `/account` |
| ATM | `/atm` |
| Transfer Money | `/transfer` |
| Transaction History | `/transactions` |
| Loan Application | `/loans` |
| Profile Settings | `/profile` |
| Admin Dashboard | `/admin` |
| Staff Dashboard | `/staff` |
| Audit Logs | `/audit-logs` |

---

# Key Files

| File | Purpose |
|---|---|
| `src/main.rs` | Starts the Actix Web server |
| `src/db.rs` | PostgreSQL connection setup |
| `src/config.rs` | Loads environment variables |
| `src/routes/` | Web route handlers |
| `src/services/` | Business logic |
| `src/models/` | Data structs and models |
| `templates/` | Tera HTML templates |
| `static/css/` | CSS styling |
| `static/js/` | JavaScript files |
| `migrations/` | Database migration scripts |
| `deployment/WIVAHbank.conf` | Project-specific Nginx reverse proxy config copied by `setup_all.bat` |
| `nginxConfFileSetup.md` | Main Windows Nginx config content |
| `setup_all.bat` | Automated Windows setup script |
| `stop_all.bat` | Stops project-related processes on Windows |

---

# Common Issues and Fixes

## 1. `http://localhost:8080` works, but `http://localhost` does not

This means the Rust server is working, but Nginx is not forwarding correctly.

Check that:

1. Nginx is running.
2. `WIVAHbank.conf` is in the correct Nginx config folder.
3. The main `nginx.conf` includes the project config.
4. Nginx was reloaded after config changes.

Windows reload:

```bat
cd /d C:\nginx
nginx.exe -s reload
```

macOS reload:

```bash
brew services restart nginx
```

Linux reload:

```bash
sudo systemctl reload nginx
```

---

## 2. `http://localhost` does not work, but `http://localhost:8080` works

Use this order to test:

1. Start the Rust server:

```bat
cargo run
```

2. Open:

```text
http://localhost:8080
```

3. Start Nginx:

```bat
cd /d C:\nginx
start nginx.exe
```

4. Open:

```text
http://localhost
```

If step 2 works but step 4 fails, the issue is Nginx, not Rust.

---

## 3. `nginx.exe` is not recognized on Windows

This means you are not inside the Nginx folder.

Run:

```bat
cd /d C:\nginx
nginx.exe -t
```

If this still fails, check whether your Nginx files are nested incorrectly, for example:

```text
C:\nginx\nginx-1.31.1\nginx.exe
```

The expected path is:

```text
C:\nginx\nginx.exe
```

---

## 4. Nginx folder is nested after extraction

Wrong:

```text
C:\nginx\nginx-1.31.1\nginx.exe
```

Correct:

```text
C:\nginx\nginx.exe
```

Move the contents of `C:\nginx\nginx-1.31.1` into `C:\nginx`.

If you cannot move it, close any File Explorer windows inside that folder and stop Nginx:

```bat
taskkill /F /IM nginx.exe
```

Then try again.

---

## 5. Installed Nginx using Winget instead of the website

If Nginx was installed using Winget, the folder may be somewhere like:

```text
C:\Users\USER\AppData\Local\Microsoft\WinGet\Packages\nginxinc.nginx_Microsoft.Winget.Source_8wekyb3d8bbwe\nginx-1.31.1
```

This README assumes the official Nginx ZIP method and this path:

```text
C:\nginx
```

For this project, use the official ZIP method so the instructions match correctly.

---

## 6. Nginx is still running and cannot be moved or uninstalled

Stop it first:

```bat
taskkill /F /IM nginx.exe
```

Then check:

```bat
tasklist | findstr nginx
```

If nothing appears, Nginx is stopped.

---

## 7. Port 80 is already in use

Another program may already be using port `80`.

Windows:

```bat
netstat -ano | findstr :80
```

macOS/Linux:

```bash
sudo lsof -i :80
```

Close the program using port `80`, then restart Nginx.

---

## 8. SQLx `.env` parsing error

Make sure `.env` has no random text or broken lines.

Correct:

```env
DATABASE_URL=postgres://postgres:1234@localhost:5432/banking_system
```

Wrong:

```env
DATABASE_URL = postgres://postgres:1234@localhost:5432/banking_system
```

Wrong:

```env
W
DATABASE_URL=postgres://postgres:1234@localhost:5432/banking_system
```

---

## 9. Database connection failed

Check that:

1. PostgreSQL is running.
2. The database `banking_system` exists.
3. The PostgreSQL username and password in `.env` are correct.
4. `DATABASE_URL` is correct.

Example:

```env
DATABASE_URL=postgres://postgres:1234@localhost:5432/banking_system
```

---

# Git Ignore

Recommended `.gitignore`:

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
setup_all.bat
stop_all.bat
batScripts/
deployment/
migrations/
src/
templates/
static/
nginxConfFileSetup.md
```

Do not commit:

```text
.env
target/
```

---

# Features

- User registration
- User login
- Customer dashboard
- Account details
- ATM deposit and withdrawal
- Bank transfer
- Transaction history
- Transaction statement
- Loan application
- Profile settings
- Admin dashboard
- Staff dashboard
- Audit logs
- Password reset
- PostgreSQL database integration
- Nginx reverse proxy for `http://localhost`
