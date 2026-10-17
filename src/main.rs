use std::env;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use reqwest::header;
use serde::Deserialize;

use crate::{
    dto::DatasourceDto,
    event_collector::{EventQuery, create_calendar},
    notion_api::NotionApi,
};

mod dto;
mod event_collector;
mod notion_api;

#[derive(Debug, Deserialize)]
struct SearchParams {
    query: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    datasource_id: String,
    title_id: String,
    date_id: String,
    description_id: String,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();

    let api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY needs to be set");
    let api_version = env::var("NOTION_API_VERSION").expect("NOTION_API_VERSION needs to be set");

    let api = NotionApi::new(&api_key, &api_version);

    // build our application with a route
    let app = Router::new()
        .route("/datasources", get(get_data_sources))
        .route("/query", get(query_data_source))
        .route("/calendar", get(create_calendar_from_datasource))
        .with_state(api);

    // run our app with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn get_data_sources(
    State(api): State<NotionApi>,
    query: Query<SearchParams>,
) -> impl IntoResponse {
    let r = api
        .search_datasources(query.query.clone().unwrap_or("".to_string()).as_str())
        .await
        .unwrap()
        .iter()
        .map(|page| DatasourceDto::from_page(page))
        .collect::<Vec<DatasourceDto>>();

    (StatusCode::OK, Json(r))
}

async fn query_data_source(
    State(api): State<NotionApi>,
    query: Query<QueryParams>,
) -> impl IntoResponse {
    let r = api
        .query_datasource(
            query.datasource_id.as_str(),
            None,
            query.date_id.as_str(),
            query.title_id.as_str(),
            query.description_id.as_str(),
        )
        .await
        .unwrap()
        .results;

    (StatusCode::OK, Json(r))
}

async fn create_calendar_from_datasource(
    State(api): State<NotionApi>,
    query: Query<EventQuery>,
) -> impl IntoResponse {
    let r = create_calendar(&api, query.0).await.unwrap();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/calendar")],
        r.to_string(),
    )
}
