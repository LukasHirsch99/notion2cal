use reqwest::StatusCode;
use topcoat::{
    Result,
    context::Cx,
    router::{content::Json, error::RouterErrorExt, query_params, response::IntoResponse, route},
};

use crate::{event_collector::EventQuery, middleware::current_api};

#[route(GET)]
async fn get(cx: &Cx) -> Result<impl IntoResponse> {
    let api = current_api(cx).await.ok_or_unauthorized()?;
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
