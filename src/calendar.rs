use std::collections::HashMap;

use crate::{
    components::{
        button::{ButtonSize, ButtonVariant, button, button_variants},
        input::input,
    },
    config::{Config, Keyring},
    db::{Repo, calendar::CalendarConfig},
    event_collector::{
        self,
        template::{self, PAGE_URL_FIELD, Template},
    },
    middleware::{current_user, require_api},
    notion_api::{NotionApi, query_datasource::DataSource, types::page::PagePropertyTpye},
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        StatusCode,
        error::RouterErrorExt,
        page, path_param,
        response::{IntoResponse, Response},
        route,
    },
    runtime::{Event, procedure, signal},
    view::{View, attributes, component, view},
};
use uuid::Uuid;

path_param!(pub datasource_id: Uuid, error = bad_request);
path_param!(pub calendar_id: Uuid, error = not_found);

fn datasource_title(datasource: &DataSource) -> String {
    datasource
        .title
        .iter()
        .fold(String::new(), |acc, part| acc + part.plain_text.as_str())
}

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

/// Parses an optional template field. A blank field means "not set".
fn optional_template(
    src: &str,
    label: &str,
    columns: &HashMap<String, ()>,
) -> Result<Template, String> {
    if src.trim().is_empty() {
        return Ok(Template::new());
    }
    template::parse(src, columns.clone()).map_err(|error| format!("{label} template: {error}."))
}

/// Configure how a Notion datasource becomes a calendar.
#[page("/calendar/{datasource_id}")]
async fn configure_calendar(cx: &Cx) -> Result<impl View> {
    let api = require_api(cx).await?;
    let datasource_id = *path_param::<DatasourceId>(cx)?;
    let datasource = api
        .retrieve_datasource(&datasource_id)
        .await
        .map_err(topcoat::Error::msg)?;

    let title = datasource_title(&datasource);
    let date_columns = date_columns(&datasource);
    let default_date_col = date_columns.first().cloned().unwrap_or_default();
    let default_title = datasource
        .properties
        .iter()
        .find(|(_, property)| property.field_type == PagePropertyTpye::Title)
        .map(|(name, _)| format!("{{{name}}}"))
        .unwrap_or_default();

    Ok(view! {
        calendar_form(
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
        datasource_title(&datasource)
    } else {
        calendar.title.clone()
    };

    Ok(view! {
        calendar_form(
            datasource_id: datasource_id.to_string(),
            calendar_id: Some(calendar_id.to_string()),
            title: title,
            date_columns: date_columns(&datasource),
            columns_hint: columns_hint(&datasource),
            date_col: config.date_col.clone(),
            title_template: template::format_source(&config.title_template),
            description_template: template::format_source(&config.description_template),
            location_template: template::format_source(&config.location_template),
            url_template: template::format_source(&config.url_template)
        )
    })
}

#[component]
async fn calendar_form(
    cx: &Cx,
    datasource_id: String,
    calendar_id: Option<String>,
    title: String,
    date_columns: Vec<String>,
    columns_hint: String,
    date_col: String,
    title_template: String,
    description_template: String,
    location_template: String,
    url_template: String,
) -> Result<impl View> {
    let date_col = signal(cx, || date_col);
    let title_template = signal(cx, || title_template);
    let description_template = signal(cx, || description_template);
    let location_template = signal(cx, || location_template);
    let url_template = signal(cx, || url_template);
    let error = signal(cx, String::new);

    let is_edit = calendar_id.is_some();
    let heading = if is_edit {
        "Edit calendar"
    } else {
        "Configure calendar"
    };
    let save_label = if is_edit {
        "Save changes"
    } else {
        "Save calendar"
    };

    Ok(view! {
        <main class="mx-auto w-full pb-16">
            <a
                href="/"
                class="text-sm text-muted-foreground transition-colors hover:text-foreground"
            >
                "← Calendars"
            </a>

            <header class="mt-6">
                <h1 class="text-2xl font-semibold tracking-tight sm:text-3xl">
                    (heading)
                </h1>
                <p class="mt-2 text-sm text-muted-foreground sm:text-base">(title)</p>
            </header>

            <div class="mt-8 rounded-xl border border-border bg-card p-6 shadow-xs">
                if date_columns.is_empty() {
                    <p class="text-sm text-muted-foreground">
                        "This datasource has no date column. Add one in Notion, then try again."
                    </p>
                } else {
                    <div class="flex flex-col gap-5">
                        <div>
                            <label for="date-column" class="text-sm font-medium">
                                "Date column"
                            </label>
                            <select
                                id="date-column"
                                class="mt-1.5 h-9 w-full rounded-lg border border-border bg-transparent px-3 text-sm transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                                :value=$(date_col.get())
                                @change=$(|e: Event| date_col.set(e.target.value))
                            >
                                for column in date_columns {
                                    <option value=(column.clone())>(column)</option>
                                }
                            </select>
                            <p class="mt-1.5 text-xs text-muted-foreground">
                                "Events are read from this column."
                            </p>
                        </div>

                        <div>
                            <label for="title-template" class="text-sm font-medium">
                                "Title template"
                            </label>
                            input(
                                attrs: attributes! {
                                    id="title-template"
                                    type="text"
                                    class="mt-1.5"
                                    :value=$(title_template.get())
                                    @input=$(|e: Event| title_template.set(e.target.value))
                                }
                            )
                            <p class="mt-1.5 text-xs text-muted-foreground">
                                "Insert a column with "
                                <code class="font-mono">"{Column Name}"</code>
                                ", or "
                                <code class="font-mono">"{Page URL}"</code>
                                " for the link to the Notion page. Available: "
                                (columns_hint)
                            </p>
                        </div>

                        <div>
                            <label
                                for="description-template"
                                class="text-sm font-medium"
                            >
                                "Description template"
                                <span class="font-normal text-muted-foreground">
                                    " (optional)"
                                </span>
                            </label>
                            <textarea
                                id="description-template"
                                rows="3"
                                class="mt-1.5 block min-h-20 w-full rounded-lg border border-border bg-transparent px-3 py-2 text-sm transition-colors outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                                :value=$(description_template.get())
                                @input=$(|e: Event| description_template.set(e.target.value))
                            ></textarea>
                            <p class="mt-1.5 text-xs text-muted-foreground">
                                "Uses the same "
                                <code class="font-mono">"{Column Name}"</code>
                                " syntax, including "
                                <code class="font-mono">"{Page URL}"</code>
                                "."
                            </p>
                        </div>

                        <div>
                            <label for="location-template" class="text-sm font-medium">
                                "Location template"
                                <span class="font-normal text-muted-foreground">
                                    " (optional)"
                                </span>
                            </label>
                            input(
                                attrs: attributes! {
                                    id="location-template"
                                    type="text"
                                    class="mt-1.5"
                                    :value=$(location_template.get())
                                    @input=$(|e: Event| location_template.set(e.target.value))
                                }
                            )
                            <p class="mt-1.5 text-xs text-muted-foreground">
                                "Sets the event location. Uses the same "
                                <code class="font-mono">"{Column Name}"</code>
                                " syntax, including "
                                <code class="font-mono">"{Page URL}"</code>
                                "."
                            </p>
                        </div>

                        <div>
                            <label for="url-template" class="text-sm font-medium">
                                "URL template"
                                <span class="font-normal text-muted-foreground">
                                    " (optional)"
                                </span>
                            </label>
                            input(
                                attrs: attributes! {
                                    id="url-template"
                                    type="text"
                                    class="mt-1.5"
                                    :value=$(url_template.get())
                                    @input=$(|e: Event| url_template.set(e.target.value))
                                }
                            )
                            <p class="mt-1.5 text-xs text-muted-foreground">
                                "Sets the event URL. Uses the same "
                                <code class="font-mono">"{Column Name}"</code>
                                " syntax, including "
                                <code class="font-mono">"{Page URL}"</code>
                                "."
                            </p>
                        </div>

                        <p
                            :hidden=$(error.get().is_empty())
                            class="rounded-lg bg-destructive/10 px-3 py-2 text-sm text-destructive"
                            role="alert"
                        >
                            $(error.get())
                        </p>

                        <div class="flex items-center justify-end gap-2">
                            <a
                                href="/"
                                class=(button_variants(
                                    ButtonVariant::Outline,
                                    ButtonSize::Md,
                                ))
                            >
                                "Cancel"
                            </a>
                            button(
                                attrs: attributes! {
                                    type="button"
                                    @click=$(async |_e: Event| {
                                        let message = save_calendar(
                                            datasource_id.to_owned(),
                                            calendar_id.clone(),
                                            date_col.get(),
                                            title_template.get(),
                                            description_template.get(),
                                            location_template.get(),
                                            url_template.get(),
                                        ).await;
                                        if message.is_empty() {
                                            raw!("location.assign('/')")
                                        } else {
                                            error.set(message)
                                        }
                                    })
                                },
                                (save_label)
                            )
                        </div>
                    </div>
                }
            </div>
        </main>
    })
}

/// Persist the calendar configuration. Returns an empty string on success, or a
/// message for the user on failure.
#[procedure]
async fn save_calendar(
    cx: &Cx,
    datasource_id: String,
    calendar_id: Option<String>,
    date_col: String,
    title_template: String,
    description_template: String,
    location_template: String,
    url_template: String,
) -> Result<String> {
    let user = current_user(cx).await?;
    let api = require_api(cx).await?;

    let Ok(datasource_id) = Uuid::parse_str(&datasource_id) else {
        return Ok("Invalid datasource.".to_owned());
    };

    let datasource = match api.retrieve_datasource(&datasource_id).await {
        Ok(datasource) => datasource,
        Err(error) => {
            tracing::error!(%error, "failed to load datasource");
            return Ok("Could not load the datasource. Try again.".to_owned());
        }
    };

    let Some(property) = datasource.properties.get(&date_col) else {
        return Ok("Choose a date column.".to_owned());
    };
    if property.field_type != PagePropertyTpye::Date {
        return Ok("The selected column is not a date column.".to_owned());
    }

    if title_template.trim().is_empty() {
        return Ok("Title template is required.".to_owned());
    }

    let mut columns: HashMap<String, ()> = datasource
        .properties
        .keys()
        .map(|name| (name.clone(), ()))
        .collect();
    columns.insert(PAGE_URL_FIELD.to_owned(), ());

    let title_template = match template::parse(&title_template, columns.clone()) {
        Ok(template) => template,
        Err(error) => return Ok(format!("Title template: {error}.")),
    };

    let description_template =
        match optional_template(&description_template, "Description", &columns) {
            Ok(template) => template,
            Err(message) => return Ok(message),
        };

    let location_template = match optional_template(&location_template, "Location", &columns) {
        Ok(template) => template,
        Err(message) => return Ok(message),
    };

    let url_template = match optional_template(&url_template, "URL", &columns) {
        Ok(template) => template,
        Err(message) => return Ok(message),
    };

    let config = CalendarConfig {
        date_col,
        title_template,
        description_template,
        location_template,
        url_template,
    };

    let title = datasource_title(&datasource);
    let title = if title.is_empty() {
        "Untitled calendar".to_owned()
    } else {
        title
    };

    let db: &Repo = app_context(cx);

    let result = match calendar_id {
        Some(calendar_id) => {
            let Ok(calendar_id) = Uuid::parse_str(&calendar_id) else {
                return Ok("Invalid calendar.".to_owned());
            };
            db.update_calendar(calendar_id, user.id, &title, &config)
                .await
        }
        None => Ok(db
            .create_calendar(user.id, datasource_id, &title, &config)
            .await
            .is_ok()),
    };

    match result {
        Ok(true) => Ok(String::new()),
        Ok(false) => Ok("Calendar not found.".to_owned()),
        Err(error) => {
            tracing::error!(%error, "failed to save calendar");
            Ok("Could not save the calendar. Try again.".to_owned())
        }
    }
}

/// Delete one of the user's calendars. Returns an empty string on success, or a
/// message for the user on failure.
#[procedure]
pub async fn delete_calendar(cx: &Cx, calendar_id: String) -> Result<String> {
    let user = current_user(cx).await?;
    let Ok(calendar_id) = Uuid::parse_str(&calendar_id) else {
        return Ok("Invalid calendar.".to_owned());
    };

    let db: &Repo = app_context(cx);
    match db.delete_for_user(calendar_id, user.id).await {
        Ok(true) => Ok(String::new()),
        Ok(false) => Ok("Calendar not found.".to_owned()),
        Err(error) => {
            tracing::error!(%error, "failed to delete calendar");
            Ok("Could not delete the calendar. Try again.".to_owned())
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_collector::template::Segment;

    #[test]
    fn parses_an_optional_template() {
        let mut columns = HashMap::new();
        columns.insert("Name".to_owned(), ());

        assert_eq!(
            optional_template("{Name}", "Location", &columns).unwrap(),
            vec![Segment::Field("Name".to_owned())]
        );
    }

    #[test]
    fn treats_a_blank_optional_template_as_empty() {
        assert!(
            optional_template("  ", "URL", &HashMap::new())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_unknown_columns_in_an_optional_template() {
        assert_eq!(
            optional_template("{Nope}", "URL", &HashMap::new()).unwrap_err(),
            "URL template: unknown column `Nope` at position 0."
        );
    }
}
