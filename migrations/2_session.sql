create table sessions (
  token_hash  bytea primary key,
  user_id     uuid not null references users(id) on delete cascade,
  expires_at  timestamptz not null
);
