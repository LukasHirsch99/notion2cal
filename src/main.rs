use crate::{
    components::{datasource_search::search, welcome::welcome},
    middleware::current_user,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    cookie::RouterBuilderCookieExt,
    router::{RouterBuilderDiscoverExt, Slot, layout, module_router, page},
    runtime::RouterBuilderRuntimeExt,
    session::{RouterBuilderSessionExt, SessionConfig},
    tailwind,
    view::{View, view},
};

mod auth;
mod calendar;
mod components;
mod datasources;
mod db;
mod dto;
mod event_collector;
mod middleware;
mod notion_api;
mod query_datasource;

struct AppConfig {
    oauth_client_id: String,
    oauth_client_secret: String,
    notion_auth_url: String,
    redirect_uri: String,
    api_version: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();

    let api_version = env::var("NOTION_API_VERSION").expect("NOTION_API_VERSION must be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let oauth_client_id = env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID must be set");
    let oauth_client_secret =
        env::var("OAUTH_CLIENT_SECRET").expect("OAUTH_CLIENT_SECRET must be set");
    let notion_auth_url = env::var("NOTION_AUTH_URL").expect("NOTION_AUTH_URL must be set");
    let redirect_uri = env::var("REDIRECT_URI").expect("REDIRECT_URI must be set");

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("failed to connect to DATABASE_URL");

    sqlx::migrate!().run(&db).await?;

    let base_url = app_base_url(&redirect_uri);

    let router = module_router!()
        .discover()
        .cookies()
        .sessions(SessionConfig::default())
        .assets(AssetBundle::load()?)
        .base_url(base_url)
        .runtime()
        // .app_context(api)
        .app_context(db)
        .app_context(AppConfig {
            oauth_client_id,
            oauth_client_secret,
            notion_auth_url,
            redirect_uri,
            api_version,
        })
        .build();

    topcoat::start(router).await?;
    Ok(())
}

#[layout]
async fn layout(slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Notion2Cal"</title>
                <meta name="viewport" content="width=device-width, initial-scale=1">
                topcoat::dev::script()
                topcoat::runtime::script()
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
            </head>
            <body class="p-5">(slot)</body>
        </html>
    })
}

#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    let logged_in = current_user(cx).await.is_some();

    Ok(view! {
        if logged_in {
            search()
        } else {
            welcome()
        }
    })
}

/// The public origin of the app. The OAuth redirect URI points at `/auth` on
/// that origin, so deriving it avoids a second deployment setting.
fn app_base_url(redirect_uri: &str) -> String {
    let trimmed = redirect_uri.trim_end_matches('/');
    trimmed.strip_suffix("/auth").unwrap_or(trimmed).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_the_app_base_url_from_the_redirect_uri() {
        assert_eq!(
            app_base_url("https://n2c.hirsch.engineer/auth"),
            "https://n2c.hirsch.engineer"
        );
        assert_eq!(
            app_base_url("http://localhost:3000/auth/"),
            "http://localhost:3000"
        );
        assert_eq!(
            app_base_url("https://example.com/custom/callback"),
            "https://example.com/custom/callback"
        );
    }
}
