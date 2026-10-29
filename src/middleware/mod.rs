use crate::{
    config::{Config, Keyring},
    db::{self, Repo, user::User},
    notion_api::{
        NotionApi,
        token::{
            self,
            Owner::{self},
        },
    },
};
use secrecy::{ExposeSecret, SecretString};
use topcoat::{
    Result,
    context::{Cx, app_context, memoize},
    router::error::{RouterErrorExt, UnauthorizedError},
    session::{self},
};
use uuid::Uuid;

pub mod encryption;

#[derive(Debug)]
struct CreateUser {
    id: Uuid,
    name: Option<String>,
    avatar_url: Option<String>,
    email: Option<String>,
}

pub async fn login_user(
    cx: &Cx,
    owner: Owner,
    access_token: SecretString,
    refresh_token: SecretString,
    bot_id: String,
    workspace_icon: Option<String>,
    workspace_id: String,
    workspace_name: Option<String>,
) -> anyhow::Result<()> {
    let Owner::User { user } = owner else {
        return Err(anyhow::anyhow!("Invalid owner type"));
    };

    let user = match user {
        token::User::Person {
            id,
            name,
            avatar_url,
            person,
        } => CreateUser {
            id,
            name,
            avatar_url,
            email: person.email,
        },
        token::User::AnonymPerson { id } => CreateUser {
            id,
            name: None,
            avatar_url: None,
            email: None,
        },
    };

    let db: &Repo = app_context(cx);
    let keyring: &Keyring = app_context(cx);

    db.upsert_user(
        keyring,
        &db::user::User {
            id: user.id,
            username: user.name,
            avatar_url: user.avatar_url,
            email: user.email,
            bot_id: Some(bot_id),
            workspace_id: Some(workspace_id),
            workspace_name,
            workspace_icon,
        },
        access_token,
        refresh_token,
    )
    .await?;

    if matches!(
        session::token_hash(cx)
            .await
            .map_err(|_| anyhow::anyhow!("Error retrieving session"))?,
        None
    ) {
        let session = session::start(cx)
            .await
            .map_err(|_| anyhow::anyhow!("Could not start session"))?;
        let token_hash = session.token_hash;
        let expires_at = session.expires_at;
        db.create_session(&token_hash.to_vec(), user.id, expires_at.into())
            .await?;
    }

    Ok(())
}

async fn session_hash(cx: &Cx) -> Result<Vec<u8>, UnauthorizedError> {
    Ok(session::token_hash(cx)
        .await
        .ok_or_unauthorized()?
        .ok_or_unauthorized()?
        .to_vec())
}

pub async fn current_user(cx: &Cx) -> Result<&User, UnauthorizedError> {
    let token = session_hash(cx).await?;
    fetch_user_by_session_hash(cx, &token)
        .await
        .map_err(Clone::clone)
}

#[memoize(as_ref)]
async fn fetch_user_by_session_hash(
    cx: &Cx,
    session_hash: &Vec<u8>,
) -> Result<User, UnauthorizedError> {
    let db: &Repo = app_context(cx);
    db.get_user_by_session(session_hash)
        .await
        .ok_or_unauthorized()?
        .ok_or_unauthorized()
}

#[memoize(as_ref)]
async fn fetch_access_token_by_session_hash(
    cx: &Cx,
    session_hash: &Vec<u8>,
) -> Result<SecretString, UnauthorizedError> {
    let db: &Repo = app_context(cx);
    let keyring: &Keyring = app_context(cx);

    db.get_token_by_hash(keyring, session_hash)
        .await
        .ok_or_unauthorized()
}

pub async fn require_api(cx: &Cx) -> Result<NotionApi, UnauthorizedError> {
    let session_hash = session_hash(cx).await?;
    let token = fetch_access_token_by_session_hash(cx, &session_hash)
        .await
        .map_err(Clone::clone)?;
    let token = SecretString::from(token.expose_secret().to_owned());
    let config: &Config = app_context(cx);
    Ok(NotionApi::new(token, &config.api_version))
}
