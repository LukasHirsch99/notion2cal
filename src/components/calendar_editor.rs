use topcoat::{
    Result,
    context::{Cx, app_context},
    runtime::{Event, procedure, signal},
    view::{View, attributes, component, view},
};
use uuid::Uuid;

use crate::{
    components::{
        button::{ButtonSize, ButtonVariant, button, button_variants},
        input::input,
    },
    db::{Repo, calendar::CalendarConfig},
    event_collector::template::{PAGE_URL_FIELD, Template},
    middleware::{current_user, require_api},
    notion_api::types::page::PagePropertyTpye,
};

#[component]
pub async fn calendar_editor(
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
        "Create calendar"
    };
    let save_label = if is_edit {
        "Save changes"
    } else {
        "Create calendar"
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

    let mut columns: Vec<String> = datasource
        .properties
        .keys()
        .map(|name| name.clone())
        .collect();
    columns.push(PAGE_URL_FIELD.to_owned());

    let title_template = match Template::parse(&title_template, &columns) {
        Ok(template) => template,
        Err(error) => return Ok(format!("Title template: {error}.")),
    };

    let description_template = match Template::parse(&description_template, &columns) {
        Ok(template) => template,
        Err(message) => return Ok(format!("Description: {message}")),
    };

    let location_template = match Template::parse(&location_template, &columns) {
        Ok(template) => template,
        Err(message) => return Ok(format!("Location: {message}")),
    };

    let url_template = match Template::parse(&url_template, &columns) {
        Ok(template) => template,
        Err(message) => return Ok(format!("URL: {message}")),
    };

    let config = CalendarConfig {
        date_col,
        title_template,
        description_template,
        location_template,
        url_template,
    };

    let title = datasource.title();
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
