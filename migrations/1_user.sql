create table users (
    id              uuid primary key,
    username        text,
    avatar_url      text,
    email           text,
    access_token    bytea,
    refresh_token   bytea,
    bot_id          text,
    workspace_id    text,
    workspace_name  text,
    workspace_icon  text
);
