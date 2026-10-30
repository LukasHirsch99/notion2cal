use crate::{
    components::calendar_editor::calendar_editor,
    config::{Config, Keyring},
    db::Repo,
    event_collector::{
        self,
        template::{self},
    },
    middleware::{current_user, require_api},
    notion_api::{
        NotionApi, NotionApiError, query_datasource::DataSource, types::page::PagePropertyTpye,
    },
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        StatusCode,
        error::{RouterErrorExt, see_other},
        page, path_param,
        response::{IntoResponse, Response},
        route,
    },
    session,
    view::{View, view},
};
use uuid::Uuid;

path_param!(pub datasource_id: Uuid, error = bad_request);
path_param!(pub calendar_id: Uuid, error = not_found);

fn date_columns(datasource: &DataSource) -> Vec<String> {
    let mut columns: Vec<String> = datasource
        .properties
        .iter()
        .filter(|(_, property)| property.field_type == PagePropertyTpye::Date)
        .map(|(name, _)| name.clone())
        .collect();
    columns.sort();
    columns
}

fn columns_hint(datasource: &DataSource) -> String {
    let mut columns: Vec<String> = datasource.properties.keys().cloned().collect();
    columns.sort();
    columns.join(", ")
}

/// Configure how a Notion datasource becomes a calendar.
#[page("/calendar/{datasource_id}")]
async fn configure_calendar(cx: &Cx) -> Result<impl View> {
    let api = require_api(cx).await?;
    let datasource_id = *path_param::<DatasourceId>(cx)?;
    let datasource = match api.retrieve_datasource(&datasource_id).await {
        Ok(source) => source,
        Err(err) => {
            return if NotionApiError::is_unauthorized(&err) {
                session::stop(cx).await?;
                Err(see_other("/").into())
            } else {
                Err(topcoat::Error::msg(err))
            };
        }
    };

    let title = datasource.title();
    let date_columns = date_columns(&datasource);
    let default_date_col = date_columns.first().cloned().unwrap_or_default();
    let default_title = datasource
        .properties
        .iter()
        .find(|(_, property)| property.field_type == PagePropertyTpye::Title)
        .map(|(name, _)| format!("{{{name}}}"))
        .unwrap_or_default();

    Ok(view! {
        calendar_editor(
            datasource_id: datasource_id.to_string(),
            calendar_id: None,
            title: title,
            date_columns: date_columns,
            columns_hint: columns_hint(&datasource),
            date_col: default_date_col,
            title_template: default_title,
            description_template: String::new(),
            location_template: String::new(),
            url_template: String::new()
        )
    })
}

/// Edit a saved calendar's configuration.
#[page("/calendars/{calendar_id}/edit")]
async fn edit_calendar(cx: &Cx) -> Result<impl View> {
    let user = current_user(cx).await?;
    let calendar_id = *path_param::<CalendarId>(cx)?;

    let db: &Repo = app_context(cx);
    let calendar = db
        .find_for_user(calendar_id, user.id)
        .await?
        .ok_or_not_found()?;

    let datasource_id = calendar.datasource_id;
    let api = require_api(cx).await?;
    let datasource = api
        .retrieve_datasource(&datasource_id)
        .await
        .map_err(topcoat::Error::msg)?;

    let config = calendar.config.0;
    let title = if calendar.title.is_empty() {
        datasource.title()
    } else {
        calendar.title.clone()
    };

    Ok(view! {
        calendar_editor(
            datasource_id: datasource_id.to_string(),
            calendar_id: Some(calendar_id.to_string()),
            title: title,
            date_columns: date_columns(&datasource),
            columns_hint: columns_hint(&datasource),
            date_col: config.date_col.clone(),
            title_template: config.title_template.to_string(),
            description_template: config.description_template.to_string(),
            location_template: config.location_template.to_string(),
            url_template: config.url_template.to_string()
        )
    })
}

/// Public iCal feed for a saved calendar.
///
/// Calendar clients such as Google Calendar fetch this URL without a session,
/// so the calendar's UUID acts as the access token.
#[route(GET "/calendar/{calendar_id}/calendar.ics")]
async fn calendar_feed(cx: &Cx) -> Result<Response> {
    let calendar_id = *path_param::<CalendarId>(cx)?;

    let db: &Repo = app_context(cx);
    let keyring: &Keyring = app_context(cx);
    let feed = db.find_feed(keyring, calendar_id).await.ok_or_not_found()?;

    let config: &Config = app_context(cx);
    let api = NotionApi::new(feed.access_token, &config.api_version);

    let ics = match event_collector::create_calendar(&api, feed.datasource_id, &feed.config).await {
        Ok(calendar) => calendar.to_string(),
        Err(error) => {
            tracing::error!(%error, %calendar_id, "failed to build calendar feed");
            return (StatusCode::BAD_GATEWAY, "Could not build calendar").into_response(cx);
        }
    };

    (
        StatusCode::OK,
        [("Content-Type", "text/calendar; charset=utf-8")],
        ics,
    )
        .into_response(cx)
}
