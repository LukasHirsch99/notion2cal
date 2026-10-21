use crate::{
    components::card::{card, card_header, card_title},
    middleware::{current_api, current_user, require_auth},
    notion_api::NotionApi,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::{Cx, app_context},
    cookie::RouterBuilderCookieExt,
    router::{RouterBuilderDiscoverExt, Slot, error::RouterErrorExt, layout, module_router, page},
    runtime::{Event, RouterBuilderRuntimeExt, shard, signal},
    session::{RouterBuilderSessionExt, SessionConfig},
    tailwind,
    view::{View, attributes, component, view},
};

mod auth;
mod calendar;
mod components;
mod datasources;
mod dto;
mod event_collector;
mod middleware;
mod notion_api;
mod query_datasource;

use components::input::input;

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

    let router = module_router!()
        .discover()
        .cookies()
        .sessions(SessionConfig::default())
        .assets(AssetBundle::load()?)
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
    let logged_in = matches!(current_user(cx).await, Some(_));
    println!("logged_in: {}", logged_in);

    Ok(view! {
        if logged_in {
            search()
        } else {
            welcome()
        }
    })
}

#[component]
async fn welcome(cx: &Cx) -> Result<impl View> {
    let secrets: &AppConfig = app_context(cx);
    Ok(view! {
        <h1>"Welcome to Notion2Cal"</h1>
        <a href=(secrets.notion_auth_url.clone())>"Login with Notion"</a>
    })
}

#[component]
async fn search(cx: &Cx) -> Result<impl View> {
    let query = signal(cx, String::new);
    let user = require_auth(cx).await?;

    Ok(view! {
        <h1>"Hello " (user.username.clone().unwrap_or(user.id.to_string())) "!"</h1>
        input(attrs: attributes! {
            @input=$(|e: Event| query.set(e.target.value))
            placeholder="Datasource name..."
            class="mb-4"
        })
        datasource_results(query: $(query.get()))
    })
}

#[shard]
async fn datasource_results(cx: &Cx, query: String) -> Result<impl View> {
    let api = current_api(cx).await.ok_or_unauthorized()?;
    let sources = api
        .search_datasources(&query)
        .await
        .map_err(|e| topcoat::Error::msg(e))?;

    Ok(view! {
        <div class="flex gap-2 flex-col">
            for source in sources {
                    card(
                        attrs: attributes! { class="w-md" },
                        card_header(
                            card_title(
                                attrs: attributes! { class="bold" },
                                (source.title_str())
                            )
                        )
                    )
            }
        </div>
    })
}
