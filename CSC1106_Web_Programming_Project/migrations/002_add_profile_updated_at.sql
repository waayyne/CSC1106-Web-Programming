alter table users
add column if not exists updated_at timestamp not null default current_timestamp;
