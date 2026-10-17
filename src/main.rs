use crate::notion_api::NotionApi;
use std::env;
use topcoat::{
    Result,
    router::{module_router, page},
    view::{View, component, view},
};

mod calendar;
mod datasources;
mod dto;
mod event_collector;
mod notion_api;
mod query_datasource;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();

    let api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY needs to be set");
    let api_version = env::var("NOTION_API_VERSION").expect("NOTION_API_VERSION needs to be set");

    let api = NotionApi::new(&api_key, &api_version);

    topcoat::start(module_router!().app_context(api).build())
        .await
        .unwrap();
}

#[page]
async fn home() -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Hello world"</title>
                topcoat::dev::script()
            </head>
            <body>
                hello(name: "World")
            </body>
        </html>
    })
}

#[component]
async fn hello(name: &str) -> Result<impl View> {
    Ok(view! {
        <h1>"Hello, " (name) "!"</h1>
    })
}
