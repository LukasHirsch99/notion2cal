create table sessions (
  token_hash bytea primary key,
  user_id    uuid not null,
  expires_at timestamptz not null
);
