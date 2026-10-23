use crate::{
    AppConfig,
    db::{
        self,
        session::create_session,
        user::{User, get_user_by_session, upsert_user},
    },
    notion_api::{
        NotionApi,
        token::{
            self,
            Owner::{self},
        },
    },
};
use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context, memoize},
    router::error::{RouterErrorExt, UnauthorizedError},
    session::{self, TokenHash},
};
use uuid::Uuid;

#[derive(Debug)]
struct CreateUser {
    id: Uuid,
    name: Option<String>,
    avatar_url: Option<String>,
    email: Option<String>,
}

pub async fn create_user(
    cx: &Cx,
    owner: Owner,
    access_token: String,
    refresh_token: String,
    bot_id: String,
    workspace_icon: Option<String>,
    workspace_id: String,
    workspace_name: Option<String>,
) -> Result<()> {
    let Owner::User { user } = owner else {
        return Err(topcoat::Error::msg("Invalid owner"));
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

    let db: &PgPool = app_context(cx);

    upsert_user(
        db,
        &db::user::User {
            id: user.id,
            username: user.name,
            avatar_url: user.avatar_url,
            email: user.email,
            access_token,
            refresh_token,
            bot_id,
            workspace_id,
            workspace_name,
            workspace_icon,
        },
    )
    .await?;

    let session = session::start(cx).await?;
    let token_hash = session.token_hash;
    let expires_at = session.expires_at;
    create_session(db, &token_hash.to_vec(), user.id, expires_at.into()).await?;

    Ok(())
}

pub async fn current_user(cx: &Cx) -> Option<&User> {
    let token = session::token_hash(cx).await.ok().flatten()?;
    fetch_user_by_hash(cx, &token).await
}

#[memoize(as_ref)]
async fn fetch_user_by_hash(cx: &Cx, token_hash: &TokenHash) -> Option<User> {
    let db: &PgPool = app_context(cx);

    get_user_by_session(db, &token_hash.to_vec())
        .await
        .ok()
        .flatten()
}

pub async fn require_api(cx: &Cx) -> Result<NotionApi, UnauthorizedError> {
    let user = require_auth(cx).await?;
    let config: &AppConfig = app_context(cx);
    Ok(NotionApi::new(&user.access_token, &config.api_version))
}

pub async fn require_auth(cx: &Cx) -> Result<&User, UnauthorizedError> {
    current_user(cx).await.ok_or_unauthorized()
}
