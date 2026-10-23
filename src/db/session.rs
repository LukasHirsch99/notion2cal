use sqlx::{PgPool, types::time::OffsetDateTime};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct Session {
    pub token_hash: Vec<u8>,
    pub user_id: Uuid,
    pub expires_at: OffsetDateTime,
}

pub async fn create_session(
    db: &PgPool,
    token_hash: &Vec<u8>,
    user_id: Uuid,
    expires_at: OffsetDateTime,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        insert into sessions values ($1, $2, $3)
        "#,
        token_hash,
        user_id,
        expires_at
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn clear_stale_sessions(db: &PgPool) -> sqlx::Result<()> {
    sqlx::query!("delete from sessions where expires_at < now()")
        .execute(db)
        .await?;
    Ok(())
}
