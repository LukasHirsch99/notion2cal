use reqwest::{StatusCode, header};
use topcoat::{
    Result,
    context::Cx,
    router::{error::RouterErrorExt, query_params, response::IntoResponse, route},
};

use crate::{
    event_collector::{EventQuery, create_calendar},
    middleware::current_api,
    notion_api::NotionApi,
};

#[route(GET)]
async fn create_calendar_from_datasource(cx: &Cx) -> Result<impl IntoResponse> {
    let api: &NotionApi = current_api(cx).await.ok_or_unauthorized()?;
    let query = query_params::<EventQuery>(cx)?;

    let r = create_calendar(api, query).await.unwrap();

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/calendar")],
        r.to_string(),
    ))
}
