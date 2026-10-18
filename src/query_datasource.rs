use reqwest::StatusCode;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, query_params, response::IntoResponse, route},
};

use crate::{event_collector::EventQuery, notion_api::NotionApi};

#[route(GET)]
async fn get(cx: &Cx) -> Result<impl IntoResponse> {
    let api: &NotionApi = app_context(cx);
    let query = query_params::<EventQuery>(cx)?;
    let r = api
        .query_datasource(
            &query.datasource_id,
            None,
            &query.date_col,
            &query.title_col,
            query.description_col.as_ref().unwrap(),
        )
        .await
        .unwrap()
        .results;

    Ok((StatusCode::OK, Json(r)))
}
