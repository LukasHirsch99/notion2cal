use sqlx::{PgPool, types::time::OffsetDateTime};
use uuid::Uuid;

pub async fn create_session(
    db: &PgPool,
    token_hash: &[u8],
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
