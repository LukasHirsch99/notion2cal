use crate::{
    components::{datasource_search::search, theme_button::theme_button, welcome::welcome},
    config::{Config, Keyring},
    db::Repo,
    middleware::current_user,
};
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::{env, path::Path};
use topcoat::{
    Result, asset::{Asset, AssetBundle, RouterBuilderAssetExt, asset}, context::Cx, cookie::RouterBuilderCookieExt, font::fontsource::fontsource_font, router::{RouterBuilderDiscoverExt, Slot, layout, module_router, page}, runtime::RouterBuilderRuntimeExt, session::{RouterBuilderSessionExt, SessionConfig}, tailwind, view::{View, view},
};

mod auth;
mod calendar;
mod components;
mod config;
mod db;
mod dto;
mod event_collector;
mod middleware;
mod notion_api;

const FAVICON: Asset = asset!("assets/favicon.svg");

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("install rustls ring provider");

    // initialize tracing
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();

    let config = Config::from_env().map_err(topcoat::Error::msg)?;
    let keyring = Keyring::load(Path::new(&config.notion_key_dir), config.notion_key_current)
        .map_err(topcoat::Error::msg)?;

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(config.database_url.expose_secret())
        .await?;
    sqlx::migrate!().run(&db).await?;
    let repo = Repo::new(db);

    let base_url = app_base_url(&config.redirect_uri);

    let router = module_router!()
        .discover()
        .cookies()
        .sessions(SessionConfig::default())
        .assets(AssetBundle::load()?)
        .base_url(base_url)
        .runtime()
        .app_context(repo)
        .app_context(config)
        .app_context(keyring)
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
                <script>
                    "(function () {
                          try {
                            var saved = localStorage.getItem('theme');
                            var dark = saved
                              ? saved === 'dark'
                              : window.matchMedia('(prefers-color-scheme: dark)').matches;
                            document.documentElement.classList.toggle('dark', dark);
                          } catch (e) {}
                        })();"
                </script>
                topcoat::font::link(font: fontsource_font!(GEIST))
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                <link rel="icon" type="image/svg+xml" href=(FAVICON)>
            </head>
            <body class="mx-auto flex w-full max-w-5xl px-6 flex-col">
                <header class="py-12 flex justify-between">
                    <span class="text-sm font-semibold tracking-tight">
                        "Notion2Cal"
                    </span>
                    theme_button()
                </header>
                (slot)
            </body>
        </html>
    })
}

#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    let logged_in = current_user(cx).await.is_ok();
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
