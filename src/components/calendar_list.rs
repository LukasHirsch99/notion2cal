use crate::{
    calendar::delete_calendar,
    components::{
        button::{ButtonSize, ButtonVariant, button, button_variants},
        input::input,
    },
    db::calendar::list_for_user,
    middleware::current_user,
};
use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context, base_url},
    icon::{IconData, icon},
    runtime::{Event, signal},
    view::{View, attributes, component, svg::ViewBox, view},
};

const CALENDAR: IconData = IconData::unescaped_unchecked(
    ViewBox::new(0.0, 0.0, 24.0, 24.0),
    r#"<path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M8 2v4M16 2v4M3 10h18M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z"/>"#,
);

/// Lists the signed-in user's saved calendars with their public feed URLs.
#[component]
pub async fn calendars(cx: &Cx) -> Result<impl View> {
    let user = current_user(cx).await?;
    let db: &PgPool = app_context(cx);
    let calendars = list_for_user(db, user.id)
        .await
        .map_err(topcoat::Error::msg)?;

    let copied = signal(cx, String::new);
    let deleting = signal(cx, String::new);
    let error = signal(cx, String::new);
    let base = base_url(cx);

    struct Row {
        id: String,
        title: String,
        url: String,
        edit_url: String,
    }

    let rows: Vec<Row> = calendars
        .iter()
        .map(|calendar| {
            let title = if calendar.title.is_empty() {
                "Untitled calendar".to_owned()
            } else {
                calendar.title.clone()
            };
            Row {
                id: calendar.id.to_string(),
                title,
                url: base.join(&format!("/calendar/{}/calendar.ics", calendar.id)),
                edit_url: format!("/calendars/{}/edit", calendar.id),
            }
        })
        .collect();

    let count = rows.len();
    let count_label = if count == 1 {
        "1 calendar".to_owned()
    } else {
        format!("{count} calendars")
    };

    Ok(view! {
        if count > 0 {
            <section class="mt-8">
                <div class="mb-2 flex items-baseline justify-between">
                    <h2 class="text-xs font-medium text-muted-foreground">
                        "Your calendars"
                    </h2>
                    <span class="text-xs text-muted-foreground tabular-nums">
                        (count_label)
                    </span>
                </div>

                <ul
                    class="divide-y divide-border overflow-hidden rounded-xl border border-border bg-card shadow-xs"
                >
                    for row in rows {
                        let row_id = row.id.clone();
                        let row_url = row.url.clone();
                        <li class="px-4 py-4">
                            <div class="flex items-center justify-between gap-3">
                                <div class="flex min-w-0 items-center gap-3">
                                    <span
                                        class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-foreground/5"
                                        aria-hidden="true"
                                    >
                                        icon(data: CALENDAR, size: 16)
                                    </span>
                                    <p class="truncate text-sm font-medium">(row.title)</p>
                                </div>

                                <div
                                    class="flex shrink-0 items-center gap-2"
                                    :hidden=$(deleting.get() == row_id.clone())
                                >
                                    <a
                                        href=(row.edit_url.clone())
                                        class=(button_variants(
                                            ButtonVariant::Outline,
                                            ButtonSize::Sm,
                                        ))
                                    >
                                        "Edit"
                                    </a>
                                    button(
                                        variant: ButtonVariant::Destructive,
                                        size: ButtonSize::Sm,
                                        attrs: attributes! {
                                            type="button"
                                            @click=$(|_e: Event| deleting.set(row_id.clone()))
                                        },
                                        "Delete"
                                    )
                                </div>

                                <div
                                    class="flex shrink-0 items-center gap-2"
                                    :hidden=$(deleting.get() != row_id.clone())
                                >
                                    <span class="text-xs text-muted-foreground">
                                        "Delete this calendar?"
                                    </span>
                                    button(
                                        variant: ButtonVariant::Destructive,
                                        size: ButtonSize::Sm,
                                        attrs: attributes! {
                                            type="button"
                                            @click=$(async |_e: Event| {
                                                let message = delete_calendar(row_id.clone()).await;
                                                if message.is_empty() {
                                                    raw!("location.reload()", ())
                                                } else {
                                                    error.set(message)
                                                }
                                            })
                                        },
                                        "Delete"
                                    )
                                    button(
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Sm,
                                        attrs: attributes! {
                                            type="button"
                                            @click=$(|_e: Event| deleting.set("".to_owned()))
                                        },
                                        "Cancel"
                                    )
                                </div>
                            </div>

                            <div class="mt-3 flex items-center gap-2 sm:pl-12">
                                input(
                                    attrs: attributes! {
                                        readonly="true"
                                        value=(row_url.clone())
                                        aria-label="Calendar URL"
                                        class="font-mono text-xs text-muted-foreground"
                                    }
                                )
                                button(
                                    variant: ButtonVariant::Outline,
                                    size: ButtonSize::Sm,
                                    attrs: attributes! {
                                        type="button"
                                        @click=$(|_e: Event| {
                                            copied.set(row_id.clone());
                                            raw!("navigator.clipboard.writeText(${row_url})", ())
                                        })
                                    },
                                    "Copy"
                                )
                                <span
                                    :hidden=$(copied.get() != row_id.clone())
                                    class="shrink-0 text-xs text-muted-foreground"
                                >
                                    "Copied"
                                </span>
                            </div>
                        </li>
                    }
                </ul>

                <p
                    :hidden=$(error.get().is_empty())
                    class="mt-2 rounded-lg bg-destructive/10 px-3 py-2 text-sm text-destructive"
                    role="alert"
                >
                    $(error.get())
                </p>
            </section>
        }
    })
}
