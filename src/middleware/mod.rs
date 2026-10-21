use crate::{
    AppConfig,
    notion_api::{
        NotionApi,
        token::{
            Owner::{self},
            User,
        },
    },
};
use sqlx::{PgPool, types::time::OffsetDateTime};
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

pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub workspace_id: String,
    pub workspace_name: Option<String>,
    pub workspace_icon: Option<String>,
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
    let session = session::start(cx).await?;
    let token_hash = session.token_hash;
    let expires_at = session.expires_at;

    let Owner::User { user } = owner else {
        return Err(topcoat::Error::msg("Invalid owner"));
    };

    let create_user = match user {
        User::Person {
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
        User::AnonymPerson { id } => CreateUser {
            id,
            name: None,
            avatar_url: None,
            email: None,
        },
    };

    println!("User: {:?}", create_user);

    let db: &PgPool = app_context(cx);

    if let Some(_) = sqlx::query!(
        r#"
        select id from users where id =  $1
        "#,
        create_user.id,
    )
    .fetch_optional(db)
    .await?
    {
        sqlx::query!(
            r#"
            update users set username = $1, avatar_url = $2,
            email = $3, access_token = $4, refresh_token = $5,
            bot_id = $6, workspace_id = $7, workspace_name = $8, workspace_icon = $9
            "#,
            create_user.name,
            create_user.avatar_url,
            create_user.email,
            access_token,
            refresh_token,
            bot_id,
            workspace_id,
            workspace_name,
            workspace_icon,
        )
        .execute(db)
        .await?;
    } else {
        sqlx::query!(
            r#"
        insert into users values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
            create_user.id,
            create_user.name,
            create_user.avatar_url,
            create_user.email,
            access_token,
            refresh_token,
            bot_id,
            workspace_id,
            workspace_name,
            workspace_icon,
        )
        .execute(db)
        .await?;
    }

    println!(
        "Creating session for {}, hash: {:?}",
        create_user.id, token_hash
    );
    sqlx::query!(
        r#"
        insert into sessions values ($1, $2, $3)
        "#,
        token_hash.to_vec(),
        create_user.id,
        Into::<OffsetDateTime>::into(expires_at)
    )
    .execute(db)
    .await?;
    println!("Created session");

    Ok(())
}

pub async fn current_user(cx: &Cx) -> Option<&AuthenticatedUser> {
    let token = session::token_hash(cx).await.ok().flatten()?;
    fetch_user_by_hash(cx, &token).await
}

#[memoize(as_ref)]
async fn fetch_user_by_hash(cx: &Cx, token_hash: &TokenHash) -> Option<AuthenticatedUser> {
    println!("Fetching user by hash: {:?}", token_hash);
    let db: &PgPool = app_context(cx);

    let user = sqlx::query!(
        r#"
        select u.* from users u
        join sessions s on s.user_id = u.id
        where s.token_hash = $1
        "#,
        token_hash.to_vec()
    )
    .fetch_optional(db)
    .await
    .inspect_err(|e| println!("Error: {:?}", e))
    .ok()
    .flatten()?;
    println!("Found user: {}", &user.id);

    Some(AuthenticatedUser {
        id: user.id,
        username: user.username,
        email: user.email,
        avatar_url: user.avatar_url,
        access_token: user.access_token,
        refresh_token: user.refresh_token,
        workspace_id: user.workspace_id,
        workspace_name: user.workspace_name,
        workspace_icon: user.workspace_icon,
    })
}

#[memoize(as_ref)]
pub async fn current_api(cx: &Cx) -> Option<NotionApi> {
    let user = current_user(cx).await?;
    let config: &AppConfig = app_context(cx);

    Some(NotionApi::new(&user.access_token, &config.api_version))
}

pub async fn require_auth(cx: &Cx) -> Result<&AuthenticatedUser, UnauthorizedError> {
    current_user(cx).await.ok_or_unauthorized()
}
