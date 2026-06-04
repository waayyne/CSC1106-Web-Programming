# 🏦 Banking System

> CSC1106 Web Programming Project

A banking web application built with Rust, Actix Web, Tera Templates, and PostgreSQL 18.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| Web Framework | Actix Web |
| Templating | Tera Templates |
| Database | PostgreSQL 18 |
| Frontend | HTML / CSS |

---

## Project Structure

```
├── .env
├── .env.example
├── Cargo.toml
├── Cargo.lock
├── setup_all.bat
├── stop_all.bat
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
│   │   ├── auth_middleware.rs
│   │   └── mod.rs
│   ├── models/
│   │   ├── account.rs
│   │   ├── admin.rs
│   │   ├── audit_log.rs
│   │   ├── loan.rs
│   │   ├── mod.rs
│   │   ├── profile.rs
│   │   ├── staff.rs
│   │   ├── transaction.rs
│   │   └── user.rs
│   ├── routes/
│   │   ├── account_routes.rs
│   │   ├── admin_routes.rs
│   │   ├── auth_routes.rs
│   │   ├── customer_routes.rs
│   │   ├── loan_routes.rs
│   │   ├── mod.rs
│   │   ├── profile_routes.rs
│   │   ├── staff_routes.rs
│   │   ├── transaction_routes.rs
│   │   └── transfer_routes.rs
│   └── services/
│       ├── account_service.rs
│       ├── admin_service.rs
│       ├── audit_service.rs
│       ├── auth_service.rs
│       ├── loan_service.rs
│       ├── mod.rs
│       ├── profile_service.rs
│       ├── staff_service.rs
│       ├── transaction_service.rs
│       └── transfer_service.rs
├── static/
│   ├── css/
│   │   ├── auth.css
│   │   ├── dashboard.css
│   │   ├── forms.css
│   │   ├── sidebar.css
│   │   ├── statement.css
│   │   ├── style.css
│   │   └── transactions.css
│   └── js/
│       └── main.js
└── templates/
    ├── layout.html
    ├── home.html
    ├── login.html
    ├── register.html
    ├── dashboard.html
    ├── admin_dashboard.html
    ├── staff_dashboard.html
    ├── account_details.html
    ├── manage_accounts.html
    ├── manage_loans.html
    ├── transfer_money.html
    ├── transaction_history.html
    ├── transaction_statement.html
    ├── loan_application.html
    ├── profile_settings.html
    ├── audit_logs.html
    ├── atm.html
    ├── staff_table_view.html
    ├── forgot_password.html
    └── reset_password.html
```

---

## Prerequisites

Make sure these are installed before proceeding:

- [Rust](https://www.rust-lang.org/tools/install)
- [PostgreSQL 18](https://www.postgresql.org/download/)
- [pgAdmin 4] (optional, for GUI access)
- [Nginx](https://nginx.org/en/download.html) 

---

## Setup

### 1. Clone the repo

```bash
git clone <repo-link>
cd CSC1106_Web_Programming_Project
```

### 2. Set up the database and nginx

**Go to website provide on prerequistes tab download windows version at Mainline version tab**
- This must be done before running the setup_all.bat file in your terminal
- Make sure the downloaded nginx folder is inside your C drive (C:\nginx)

**Option A — Automated (Windows, recommended)**
**(HELPS to run querrys, create sites-enbled folder and sends deployment file to nginx\conf\sites-enabled)**

```bash
setup_all.bat
```

**IF failure try running seperate batfiles**

This will drop and recreate the `banking_system` database, then run `migrations/001_create_tables.sql`.

> ⚠️ Running this resets the database and deletes all existing data.

**Option B — Manual via pgAdmin 4**

-- DB manual setup --
1. Open pgAdmin 4
2. Create the database:
```sql
CREATE DATABASE banking_system;
```
3. Open the Query Tool under `banking_system` and paste + run the contents of `migrations/001_create_tables.sql`

-- Nginx manual setup--
1. Make sure nginx is downloaded in your C drive
2. Navigate to conf folder inside your nginx and create a newfolder called sites-enabled
3. Put the WIVAHbank.conf file from the deployment folder of this project to sites-enabled


### 2.5 Configure enginx conf file.

1. Open your C:\nginx\conf\nginx.conf file in a IDE of your choice (make sure u downloaded nginx)
2. Look at project folder find nginxConfFileSetup.md copy everything and put it in
3. Make sure to save and close the file afterwards
4. The reason why we comment the rest of the server function as well as adding the line
    [include        C:/nginx/conf/sites-enabled/*.conf;]
(This allows for customisability of conf files for different projects, so no two conf file will mess with the process of nginx)

### 3. Configure environment variables

Copy `.env.example` and rename it to `.env`:

```bash
cp .env.example .env
```

Fill in your values:

```env
DATABASE_URL=postgres://postgres:1234@localhost:5432/banking_system
SESSION_KEY=0123456701234567012345670123456701234567012345670123456701234567
APP_BASE_URL=http://localhost:8080
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-gmail-app-password
SMTP_FROM=your-email@gmail.com
```

> For Gmail, `SMTP_PASSWORD` must be a **[Gmail App Password](https://myaccount.google.com/apppasswords)**, not your regular password.
> You can also use Brevo, Mailtrap, or any other SMTP provider.

> ❌ Never commit `.env` to GitHub.

### 4. Run the server

```bash
cargo run
```

If everything is set up correctly, you should see:

```
Connected to PostgreSQL
Server running at http://127.0.0.1:8080
```

Open your browser at [https://localhost] (http://127.0.0.1:8080) this server is localhost now
OR
NOT
(http://127.0.0.1:8080) as login cookies may not work, as this ignores Nginx.


### 5. Safely exiting the program
1) Click into the terminal window that is currently running the Rust website and press Ctrl + C to terminate the active process
2) Run the stop_all.bat file. This will safely shut down the Nginx reverse proxy running in the background and clean up any remaining Rust processes

---

## Pages

| Page | URL |
|---|---|
| Homepage | `/` |
| Login | `/login` |
| Register | `/register` |
| Customer Dashboard | `/dashboard` |

---

## Key Files

| File | Purpose |
|---|---|
| `src/main.rs` | Starts the Actix Web server |
| `src/db.rs` | PostgreSQL connection setup |
| `src/routes/auth_routes.rs` | Login, register, dashboard routes |
| `src/services/auth_service.rs` | Register/login business logic |
| `src/models/user.rs` | User form structs |
| `templates/login.html` | Login page UI |
| `templates/register.html` | Register page UI |
| `migrations/001_create_tables.sql` | Database schema |
| `setup_all.bat` | Database reset script (Windows), as well as Nginx setup and reset |

---

## Form Field Reference

### Login — `templates/login.html`

```html
<form method="post" action="/login">
  <input type="email" name="email">
  <input type="password" name="password">
</form>
```

### Register — `templates/register.html`

```html
<form method="post" action="/register">
  <input type="text" name="name">
  <input type="email" name="email">
  <input type="text" name="phone_number">
  <input type="password" name="password">
</form>
```

> ⚠️ Do not rename these input fields — Rust reads them by name.

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

**Commit these:**

```
Cargo.toml
Cargo.lock
README.md
.env.example
setup_all.bat
migrations/
src/
templates/
static/
```

---

## Roadmap

- [ ] Customer dashboard with real account balance
- [ ] Deposit money
- [ ] Withdraw money
- [ ] Transfer money by account number
- [ ] Transfer money by PayNow phone number
- [ ] Transaction history
- [ ] Loan application
- [ ] Admin dashboard
- [ ] Staff dashboard
- [ ] Audit logs