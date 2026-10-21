use reqwest::StatusCode;
use topcoat::{
    Result,
    context::Cx,
    router::{content::Json, error::RouterErrorExt, query_params, response::IntoResponse, route},
};

use crate::{dto::DatasourceDto, middleware::current_api, notion_api::NotionApi};

#[query_params(error = bad_request)]
struct GetsQuery {
    query: Option<String>,
}

#[route(GET)]
async fn get(cx: &Cx) -> Result<impl IntoResponse> {
    let api: &NotionApi = current_api(cx).await.ok_or_unauthorized()?;

    let query = query_params::<GetsQuery>(cx)?;

    let r = api
        .search_datasources(&query.query.clone().unwrap_or(String::new()))
        .await
        .unwrap()
        .iter()
        .map(DatasourceDto::from_page)
        .collect::<Vec<DatasourceDto>>();

    Ok((StatusCode::OK, Json(r)))
}
