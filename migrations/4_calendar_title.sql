alter table calendars
    add column title text not null default '',
    add column created_at timestamptz not null default now();
