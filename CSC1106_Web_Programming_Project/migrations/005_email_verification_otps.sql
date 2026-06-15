alter table users
add column if not exists email_verified boolean not null default false;

alter table users
add column if not exists email_verified_at timestamp;

update users
set email_verified = true,
    email_verified_at = coalesce(email_verified_at, current_timestamp)
where email_verified = false;

create table if not exists email_verification_otps (
    id serial primary key,
    user_id int not null references users(id) on delete cascade,
    otp_hash text not null,
    expires_at timestamp not null,
    used_at timestamp,
    created_at timestamp default (now() at time zone 'Asia/Singapore')
);

create index if not exists idx_email_verification_otps_user_id
on email_verification_otps(user_id);
