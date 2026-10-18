use crate::{
    components::card::{card, card_header, card_title},
    notion_api::NotionApi,
};
use std::env;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::{Cx, app_context},
    router::{RouterBuilderDiscoverExt, Slot, layout, module_router, page},
    runtime::{Event, RouterBuilderRuntimeExt, shard, signal},
    tailwind,
    view::{View, attributes, component, view},
};

mod calendar;
mod components;
mod datasources;
mod dto;
mod event_collector;
mod notion_api;
mod query_datasource;

use components::input::input;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();

    let api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY needs to be set");
    let api_version = env::var("NOTION_API_VERSION").expect("NOTION_API_VERSION needs to be set");

    let api = NotionApi::new(&api_key, &api_version);

    let router = module_router!()
        .discover()
        .assets(AssetBundle::load()?)
        .runtime()
        .app_context(api)
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
async fn home() -> Result<impl View> {
    Ok(view! {
        search()
    })
}

#[component]
async fn search(cx: &Cx) -> Result<impl View> {
    let query = signal(cx, String::new);

    Ok(view! {
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
    let api: &NotionApi = app_context(cx);
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
