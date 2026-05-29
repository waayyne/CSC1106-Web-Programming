create table if not exists password_reset_tokens (
    id serial primary key,
    user_id int not null references users(id) on delete cascade,
    token_hash text unique not null,
    expires_at timestamp not null,
    used_at timestamp,
    created_at timestamp default current_timestamp
);

create index if not exists idx_password_reset_tokens_token_hash
on password_reset_tokens(token_hash);

create index if not exists idx_password_reset_tokens_user_id
on password_reset_tokens(user_id);
