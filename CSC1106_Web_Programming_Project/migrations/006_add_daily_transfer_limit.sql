alter table users
add column daily_transfer_limit numeric(12, 2) not null default 1000.00;