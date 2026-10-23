create table users (
    id              uuid primary key,
    username        text,
    avatar_url      text,
    email           text,
    access_token    text not null,
    refresh_token   text not null,
    bot_id          text not null,
    workspace_id    text not null,
    workspace_name  text,
    workspace_icon  text
);
