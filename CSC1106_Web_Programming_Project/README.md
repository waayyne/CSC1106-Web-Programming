# Banking System — Enterprise Web Application (Rust + Actix Web)

## Project Overview

This project is a modern enterprise banking web application developed using:

- Rust
- Actix Web
- Object Oriented Programming (OOP) concepts
- Server Side Rendering (SSR)
- Relational Database Integration

The system simulates a real world banking platform commonly used in enterprise environments. It focuses on secure transaction handling, scalable backend architecture, maintainable code structure, and business workflow implementation.

The application demonstrates:

- scalable architecture
- maintainable modular design
- authentication and authorization
- relational database persistence
- dynamic HTML rendering
- responsive frontend UI
- enterprise business logic

---

# Technologies Used

## Backend
- Rust
- Actix Web

## Frontend (SSR)
- Askama or Tera Template Engine
- HTML
- CSS
- Bootstrap/Tailwind (optional)

## Database
- PostgreSQL / MySQL / SQLite

## Other Concepts
- MVC or layered architecture
- Traits and modular Rust design
- Session authentication
- Role Based Access Control (RBAC)

---

# System Features

## Core Modules

### 1. User Account Management
Features:
- user registration
- login/logout
- password hashing
- profile management
- role based access control

### 2. Money Transfer System
Features:
- transfer funds between accounts
- balance validation
- transaction confirmation
- concurrency safe transfers

### 3. Transaction History
Features:
- transaction records
- transaction filtering
- account statements
- audit logging

### 4. Loan Management
Features:
- loan application
- approval/rejection workflow
- repayment tracking
- interest calculation

### 5. Fixed Deposit System
Features:
- fixed deposit creation
- maturity calculation
- withdrawal tracking
- interest management

### 6. Admin Dashboard
Features:
- manage users
- monitor transactions
- fraud detection rules
- analytics dashboard

---

# Advanced Features

- transaction audit logging
- concurrency safe transfers
- OTP simulation
- fraud detection rules
- dashboard analytics
- account activity monitoring
- responsive UI design

---

# Server Side Rendering (SSR)

The frontend uses Server Side Rendering (SSR) for dynamic web pages.

Supported template engines:
- Askama
- Tera

The system:
- dynamically renders data from database
- supports reusable layouts/components
- separates frontend templates from backend logic
- provides responsive UI design

---

# Project Architecture

The system follows enterprise backend architecture principles.

Example structure:

src/
│
├── controllers/
├── models/
├── services/
├── repositories/
├── middleware/
├── routes/
├── templates/
├── database/
├── utils/
└── main.rs

Architecture principles:
- separation of concerns
- modular code organization
- reusable services
- maintainable business logic
- scalable backend structure

---

# Database Design

The system integrates with a relational database.

Possible tables:
- users
- bank_accounts
- transactions
- loans
- fixed_deposits
- audit_logs
- roles

Relationships:
- one-to-many
- many-to-many
- foreign key constraints

---

# Security Features

- password hashing
- session authentication
- role based authorization
- input validation
- CSRF protection
- secure transaction handling

---

# CRUD Operations

The system supports full CRUD operations for:
- users
- bank accounts
- transactions
- loans
- deposits

---

# Setup Instructions

## 1. Clone Repository

```bash
git clone <repository-url>
cd banking-system