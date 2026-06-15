# WIVAH Bank — Banking System

CSC1106 Web Programming Project

Built with **Rust**, **Actix Web**, **Tera Templates**, **PostgreSQL**, and **Nginx**.

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

## How It Works

The Actix server runs on port `8080`. Nginx listens on port `80` and proxies requests to it.

- Direct: `http://localhost:8080`
- Via Nginx: `http://localhost` ← use this for the final setup

---

## Quick Start

### 1. Install Prerequisites

- **Rust**: https://www.rust-lang.org/tools/install
- **PostgreSQL**: https://www.postgresql.org/download/
- **Nginx**:
  - Windows: download the ZIP from https://nginx.org/en/download.html, extract it, and rename the folder to `nginx`, then move it to `C:\`. The final path must be `C:\nginx` and it must directly contain `nginx.exe`, `conf\`, `html\`, `logs\` — not another nested folder like `C:\nginx\nginx-1.31.1\`.
  - macOS: `brew install nginx`
  - Linux: `sudo apt install nginx`

### 2. Clone the Repo

```bash
git clone <repo-link>
cd CSC1106_Web_Programming_Project
```

### 3. Configure Environment

```bash
copy .env.example .env   # Windows
cp .env.example .env     # macOS/Linux
```

Edit `.env` with your values:

```env
DATABASE_URL=postgres://postgres:<your-password>@localhost:5432/banking_system
SESSION_KEY=<64-char hex string>
APP_BASE_URL=http://localhost
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-gmail-app-password
SMTP_FROM=your-email@gmail.com
```

> For Gmail, `SMTP_PASSWORD` must be a [Gmail App Password](https://myaccount.google.com/apppasswords), not your regular password.

### 4. Set Up the Database

**Windows:**

```bat
setup_all.bat
```

This creates `banking_system`, runs all migrations, copies `WIVAHbank.conf` into Nginx, tests the Nginx config, then starts Nginx (or reloads it if already running). ⚠️ It resets the database.

> Nginx must be at `C:\nginx` for the script to work. If the config test fails, it will print an error and stop before starting Nginx.


**macOS/Linux:**

```bash
createdb banking_system
psql -U postgres -d banking_system -f migrations/001_create_tables.sql
psql -U postgres -d banking_system -f migrations/002_add_profile_updated_at.sql
psql -U postgres -d banking_system -f migrations/003_perma_admin.sql
psql -U postgres -d banking_system -f migrations/004_password_reset_tokens.sql
```

---

### 5. Configure Nginx

#### Windows

**Step 1 — Replace the main Nginx config**

Open `C:\nginx\conf\nginx.conf` and replace its contents with everything in `nginxConfFileSetup.md` from this project. This adds the following line inside the `http { }` block, which tells Nginx to load project-specific configs:

```nginx
include C:/nginx/conf/sites-enabled/*.conf;
```

**Step 2 — Copy the project config**

`setup_all.bat` handles this automatically. It creates `C:\nginx\conf\sites-enabled\` if it doesn't exist and copies `deployment\WIVAHbank.conf` there.

> Note: `setup_all.bat` does **not** replace `nginx.conf` — you must do Step 1 manually first.

**Step 3 — Test and start Nginx**

```bat
cd /d C:\nginx
nginx.exe -t
start nginx.exe
```

To reload after config changes:

```bat
nginx.exe -s reload
```

#### macOS (Homebrew)

Nginx config is at `/opt/homebrew/etc/nginx/` (Apple Silicon) or `/usr/local/etc/nginx/` (Intel).

```bash
mkdir -p /opt/homebrew/etc/nginx/servers
cp deployment/WIVAHbank.conf /opt/homebrew/etc/nginx/servers/WIVAHbank.conf
```

Make sure the main `nginx.conf` has this inside the `http { }` block:

```nginx
include servers/*;
```

Test and restart:

```bash
nginx -t
brew services restart nginx
```

#### Linux (Ubuntu/Debian)

```bash
sudo cp deployment/WIVAHbank.conf /etc/nginx/sites-available/WIVAHbank.conf
sudo ln -s /etc/nginx/sites-available/WIVAHbank.conf /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 6. Run

**Windows:**

1. Run `setup_all.bat` (sets up the database, configures and starts Nginx)
2. Run the Rust server:

```bat
cargo run
```

3. Open `http://localhost`

**macOS/Linux:**

1. Start Nginx:

```bash
brew services start nginx      # macOS
sudo systemctl start nginx     # Linux
```

2. Run the Rust server:

```bash
cargo run
```

3. Open `http://localhost`

### 7. Stop

- Rust server: `Ctrl+C`
- Nginx on Windows: `cd /d C:\nginx && nginx.exe -s stop` or run `stop_all.bat`
- Nginx on macOS: `brew services stop nginx`
- Nginx on Linux: `sudo systemctl stop nginx`

---

## Pages

| Page | URL |
|---|---|
| Homepage | `/` |
| Login | `/login` |
| Register | `/register` |
| Customer Dashboard | `/dashboard` |
| Account / ATM / Transfer | `/account`, `/atm`, `/transfer` |
| Transaction History | `/transactions` |
| Loan Application | `/loans` |
| Profile Settings | `/profile` |
| Admin Dashboard | `/admin` |
| Staff Dashboard | `/staff` |
| Audit Logs | `/audit-logs` |

---

## User Roles

### Customer
- Manage personal banking activities
- Submit loan applications
- View loan status

### Staff
- Review customer loan applications
- Approve or reject loans

### Administrator
- Manage users and staff
- View audit logs
- Monitor system activities

---

## Common Issues

**`http://localhost` doesn't work but `:8080` does** — Nginx isn't running or the config isn't loaded. Check that `WIVAHbank.conf` is in the right folder and Nginx has been reloaded.

**Port 80 already in use** — find and stop the conflicting process:
```bat
netstat -ano | findstr :80       # Windows
sudo lsof -i :80                 # macOS/Linux
```

**SQLx `.env` error** — make sure there are no spaces around `=` and no stray characters in `.env`.

**Database connection failed** — confirm PostgreSQL is running, the `banking_system` database exists, and `DATABASE_URL` credentials are correct.

---

## .gitignore

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

Do not commit `.env`. Commit `.env.example`.