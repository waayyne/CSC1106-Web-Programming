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
├── setup_db.bat
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
- pgAdmin 4 (optional, for GUI access)

---

## Setup

### 1. Clone the repo

```bash
git clone <repo-link>
cd CSC1106_Web_Programming_Project
```

### 2. Set up the database

**Option A — Automated (Windows, recommended)**

```bash
setup_db.bat
```

This will drop and recreate the `banking_system` database, then run `migrations/001_create_tables.sql`.

> ⚠️ Running this resets the database and deletes all existing data.

**Option B — Manual via pgAdmin 4**

1. Open pgAdmin 4
2. Create the database:
```sql
CREATE DATABASE banking_system;
```
3. Open the Query Tool under `banking_system` and paste + run the contents of `migrations/001_create_tables.sql`

### 3. Configure environment variables

Copy `.env.example` and rename it to `.env`:

```bash
cp .env.example .env
```

Fill in your values:

```env
DATABASE_URL=postgres://postgres:1234@localhost:5432/banking_system
SESSION_KEY=0123456701234567012345670123456701234567012345670123456701234567
APP_BASE_URL=http://127.0.0.1:8080
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

Open your browser at [http://127.0.0.1:8080](http://127.0.0.1:8080)

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
| `setup_db.bat` | Database reset script (Windows) |

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
setup_db.bat
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