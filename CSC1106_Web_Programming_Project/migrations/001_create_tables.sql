create table users (
    id serial primary key,
    username varchar(50) unique not null,
    first_name varchar(50) not null,
    last_name varchar(50) not null,
    name varchar(100) not null,
    email varchar(150) unique not null,
    email_verified boolean not null default false,
    email_verified_at timestamp,
    password_hash text not null,
    phone_number varchar(20) unique not null,
    role varchar(20) not null default 'customer',
    created_at timestamp default (now() at time zone 'Asia/Singapore')
);

create table bank_accounts (
    id serial primary key,
    user_id int not null references users(id) on delete cascade,
    account_number varchar(30) unique not null,
    balance numeric(12, 2) not null default 0.00,
    created_at timestamp default (now() at time zone 'Asia/Singapore')
);

create table transactions (
    id serial primary key,
    from_account_id int references bank_accounts(id),
    to_account_id int references bank_accounts(id),
    transaction_type varchar(30) not null,
    amount numeric(12, 2) not null,
    description text,
    created_at timestamp default (now() at time zone 'Asia/Singapore')
);

create table loans (
    id serial primary key,
    user_id int not null references users(id) on delete cascade,
    amount numeric(12, 2) not null,
    status varchar(20) not null default 'pending',
    reason text,
    created_at timestamp default (now() at time zone 'Asia/Singapore')
);

create table audit_logs (
    id serial primary key,
    user_id int references users(id),
    action text not null,
    created_at timestamp default (now() at time zone 'Asia/Singapore')
);

create table fixed_deposits (
    id serial primary key,
    user_id int not null references users(id) on delete cascade,
    account_id int not null references bank_accounts(id) on delete cascade,
    principal_amount numeric(12, 2) not null,
    interest_rate numeric(5, 2) not null,
    interest_amount numeric(12, 2) not null,
    total_return numeric(12, 2) not null,
    duration_days int not null,
    maturity_seconds int not null,
    status varchar(20) not null default 'active',
    created_at timestamp default (now() at time zone 'Asia/Singapore'),
    maturity_at timestamp not null,
    claimed_at timestamp
);

create table risk_investments (
    id serial primary key,
    user_id int not null references users(id) on delete cascade,
    account_id int not null references bank_accounts(id) on delete cascade,
    amount numeric(12, 2) not null,
    risk_level varchar(20) not null,
    result varchar(20) not null,
    return_amount numeric(12, 2) not null,
    profit_loss numeric(12, 2) not null,
    created_at timestamp default (now() at time zone 'Asia/Singapore')
);

create table email_verification_otps (
    id serial primary key,
    user_id int not null references users(id) on delete cascade,
    otp_hash text not null,
    expires_at timestamp not null,
    used_at timestamp,
    created_at timestamp default (now() at time zone 'Asia/Singapore')
);

create index idx_email_verification_otps_user_id
on email_verification_otps(user_id);
