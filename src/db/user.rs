use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub bot_id: String,
    pub workspace_id: String,
    pub workspace_name: Option<String>,
    pub workspace_icon: Option<String>,
}

pub async fn upsert_user(db: &PgPool, user: &User) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        r#"
        insert into users values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        on conflict (id)
        do update set username = $2, avatar_url = $3,
        email = $4, access_token = $5, refresh_token = $6,
        bot_id = $7, workspace_id = $8, workspace_name = $9,
        workspace_icon = $10
        "#,
        user.id,
        user.username,
        user.avatar_url,
        user.email,
        user.access_token,
        user.refresh_token,
        user.bot_id,
        user.workspace_id,
        user.workspace_name,
        user.workspace_icon,
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_user(db: &PgPool, id: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"
        select * from users where id = $1
        "#,
        id
    )
    .fetch_optional(db)
    .await
}

pub async fn get_user_by_session(db: &PgPool, token_hash: &Vec<u8>) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"
        select u.* from users u
        join sessions s on s.user_id = u.id
        where s.token_hash = $1
        "#,
        token_hash
    )
    .fetch_optional(db)
    .await
}
