use crate::notion_api::NotionApi;
use std::env;
use topcoat::{
    Result, asset::{AssetBundle, RouterBuilderAssetExt}, context::{Cx, app_context}, router::{Router, RouterBuilderDiscoverExt, module_router, page}, runtime::{Event, RouterBuilderRuntimeExt, shard, signal}, view::{View, component, view},
};

mod calendar;
mod datasources;
mod dto;
mod event_collector;
mod notion_api;
mod query_datasource;

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

#[page]
async fn home() -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Notion2Cal"</title>
                topcoat::dev::script()
                topcoat::runtime::script()
            </head>
            <body>
                hello(name: "World")
                search()
            </body>
        </html>
    })
}

#[component]
async fn search(cx: &Cx) -> Result<impl View> {
    let query = signal(cx, String::new);

    Ok(view! {
        <input @input=$(|e: Event| query.set(e.target.value))>
        // Updates as the user types.
        datasource_results(query: $(query.get()))
    })
}

#[shard]
async fn datasource_results(cx: &Cx, query: String) -> Result<impl View> {
    let api: &NotionApi = app_context(cx);
    let sources = api.search_datasources(&query).await;

    match sources {
        Ok(sources) => Ok(view! {
            <ul>
                for source in sources {
                    <li> (source.title_str()) </li>
                }
            </ul>
        }),
        Err(e) => {
            println!("error: {}", e);
            Err(topcoat::Error::msg(e))
        }
    }
}

#[component]
async fn hello(name: &str) -> Result<impl View> {
    Ok(view! {
        <h1>"Hello, " (name) "!"</h1>
    })
}
