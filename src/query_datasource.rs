use reqwest::StatusCode;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, query_params, response::IntoResponse, route},
};

use crate::notion_api::NotionApi;

#[query_params(error = bad_request)]
struct GetsQuery {
    datasource_id: String,
    title_col: String,
    date_col: String,
    desc_col: String,
}

#[route(GET)]
async fn get(cx: &Cx) -> Result<impl IntoResponse> {
    let api: &NotionApi = app_context(cx);
    let query = query_params::<GetsQuery>(cx)?;
    let r = api
        .query_datasource(
            &query.datasource_id,
            None,
            &query.date_col,
            &query.title_col,
            &query.desc_col,
        )
        .await
        .unwrap()
        .results;

    Ok((StatusCode::OK, Json(r)))
}
