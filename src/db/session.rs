use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

use crate::db::Repo;

impl Repo {
    pub async fn create_session(
        &self,
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
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn clear_stale_sessions(&self) -> sqlx::Result<()> {
        sqlx::query!("delete from sessions where expires_at < now()")
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete_session(&self, token_hash: &[u8]) -> sqlx::Result<bool> {
        let result = sqlx::query!("delete from sessions where token_hash = $1", token_hash)
            .execute(&self.db)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
