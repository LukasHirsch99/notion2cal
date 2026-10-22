create table calendars (
    id            uuid primary key default gen_random_uuid(),
    user_id       uuid not null,
    datasource_id uuid not null,
    config        jsonb not null
);
