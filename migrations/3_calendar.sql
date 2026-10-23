create table calendars (
    id            uuid primary key default gen_random_uuid(),
    title         text not null default '',
    user_id       uuid not null references users(id),
    datasource_id uuid not null,
    config        jsonb not null,
    created_at    timestamptz not null default now()
);
