use reqwest::StatusCode;
use topcoat::{
    Result,
    context::Cx,
    router::{content::Json, query_params, response::IntoResponse, route},
};
use uuid::Uuid;

use crate::{
    event_collector::EventQuery, middleware::require_api,
    notion_api::query_datasource::QueryDataSourceRequest,
};

#[route(GET)]
async fn get(cx: &Cx) -> Result<impl IntoResponse> {
    let api = require_api(cx).await?;
    let query = query_params::<EventQuery>(cx)?;
    let datasource_id = Uuid::parse_str(&query.datasource_id)?;
    let r = api
        .query_datasource(
            &datasource_id,
            &[],
            QueryDataSourceRequest {
                page_size: Some(100),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .results;

    Ok((StatusCode::OK, Json(r)))
}
