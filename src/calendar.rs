use reqwest::{StatusCode, header};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{query_params, response::IntoResponse, route},
};

use crate::{
    event_collector::{EventQuery, create_calendar},
    notion_api::NotionApi,
};

#[route(GET)]
async fn create_calendar_from_datasource(cx: &Cx) -> Result<impl IntoResponse> {
    let api: &NotionApi = app_context(cx);
    let query = query_params::<EventQuery>(cx)?;

    let r = create_calendar(api, query).await.unwrap();

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/calendar")],
        r.to_string(),
    ))
}
