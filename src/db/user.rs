use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use crate::{
    config::Keyring,
    db::Repo,
    middleware::encryption::{decrypt_token, encrypt_token},
};

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub bot_id: Option<String>,
    pub workspace_id: Option<String>,
    pub workspace_name: Option<String>,
    pub workspace_icon: Option<String>,
}

impl Repo {
    pub async fn upsert_user(
        &self,
        keyring: &Keyring,
        user: &User,
        access_token: SecretString,
        refresh_token: SecretString,
    ) -> anyhow::Result<bool> {
        let (version, key) = keyring.current();
        let access_token_enc = encrypt_token(
            version,
            key,
            access_token.expose_secret(),
            &user.id.to_string(),
        )?;
        let refresh_token_enc = encrypt_token(
            version,
            key,
            refresh_token.expose_secret(),
            &user.id.to_string(),
        )?;

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
            &access_token_enc,
            &refresh_token_enc,
            user.bot_id,
            user.workspace_id,
            user.workspace_name,
            user.workspace_icon,
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_token_by_hash(
        &self,
        keyring: &Keyring,
        token_hash: &Vec<u8>,
    ) -> anyhow::Result<SecretString> {
        let result = sqlx::query!(
            r#"
            select u.id, u.access_token from users u
            join sessions s on s.user_id = u.id
            where s.token_hash = $1 and s.expires_at > now()
            "#,
            token_hash
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(anyhow::anyhow!(
            "Could not find user for token_hash {token_hash:?}"
        ))?;

        match result.access_token {
            None => Err(anyhow::anyhow!("No token for user")),
            Some(access_token) => decrypt_token(keyring, &access_token, &result.id.to_string()),
        }
    }

    pub async fn get_user(&self, id: Uuid) -> sqlx::Result<Option<User>> {
        sqlx::query_as!(
            User,
            r#"
            select id, username, avatar_url, email, bot_id,
            workspace_id, workspace_name, workspace_icon
            from users
            where id = $1
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn get_user_by_session(&self, token_hash: &Vec<u8>) -> sqlx::Result<Option<User>> {
        sqlx::query_as!(
            User,
            r#"
            select u.id, u.username, u.avatar_url, u.email, u.bot_id,
            u.workspace_id, u.workspace_name, u.workspace_icon
            from users u
            join sessions s on s.user_id = u.id
            where s.token_hash = $1 and s.expires_at > now()
            "#,
            token_hash
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn delete_user_by_id(&self, id: Uuid) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            User,
            r#"
            delete from users where id = $1
            "#,
            id
        )
        .execute(&self.db)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
