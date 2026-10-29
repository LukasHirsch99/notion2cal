use crate::{
    components::{
        avatar::{AvatarSize, avatar, avatar_fallback, avatar_image},
        button::{ButtonSize, ButtonVariant, button, button_variants},
        calendar_list::calendars,
        floating::FloatingAlign,
        input::input,
        popover::{popover, popover_content, popover_trigger},
    },
    config::Config,
    db::Repo,
    dto::DatasourceDto,
    middleware::{current_user, require_api},
    notion_api::types::page::{PageDesc, PagePropertyTpye},
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    icon::{icon, iconify},
    runtime::{Event, procedure, shard, signal},
    session,
    view::{View, attributes, component, element_id, view},
};

iconify::include!("feather");

/// The logged-in home page: search Notion datasources and list them.
#[component]
pub async fn search(cx: &Cx) -> Result<impl View> {
    let query = signal(cx, String::new);
    let user = current_user(cx).await?;
    let username = user.username.clone().unwrap_or_else(|| "there".to_owned());
    let avatar_fb = username.chars().next().unwrap();
    let avatar_url = user.avatar_url.clone().unwrap_or_else(|| "".to_owned());
    let share = element_id(cx);
    let config: &Config = app_context(cx);

    Ok(view! {
        <main class="mx-auto w-full pb-16">
            <header class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-semibold tracking-tight sm:text-3xl">
                        "Hello, "
                        (username.clone())
                        "."
                    </h1>
                    <p class="mt-2 text-sm text-muted-foreground sm:text-base">
                        "Manage your calendars, or add a new one from a Notion datasource."
                    </p>
                </div>
                popover(
                    popover_trigger(
                        target: &share,
                        avatar(
                            size: AvatarSize::Lg,
                            avatar_image(attrs: attributes! { src=(avatar_url) })
                            avatar_fallback(<span class="font-bold">(avatar_fb)</span>)
                        )
                    )
                    popover_content(
                        id: &share,
                        align: FloatingAlign::End,
                        <div class="flex flex-col gap-2">
                            <a
                                href=(config.notion_auth_url.clone())
                                class=(button_variants(
                                    ButtonVariant::Outline,
                                    ButtonSize::Sm,
                                ))
                            >
                                "Manage connection"
                                icon(data: feather::SETTINGS)
                            </a>
                            button(
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                attrs: attributes! {
                                    @click=$(async |_e: Event| {
                                        logout().await;
                                        raw!("location.assign('/')")
                                    })
                                },
                                "Logout"
                                icon(data: feather::LOG_OUT)
                            )
                            button(
                                variant: ButtonVariant::Destructive,
                                size: ButtonSize::Sm,
                                attrs: attributes! {
                                    @click=$(async |_e: Event| {
                                        delete_account().await;
                                        raw!("location.assign('/')")
                                    })
                                },
                                "Delete Account"
                                icon(data: feather::DELETE)
                            )
                        </div>
                    )
                )
            </header>

            calendars()

            <div class="mt-8">
                <p class="mb-2 text-xs font-medium text-muted-foreground">
                    "Add a calendar"
                </p>
                <div class="relative">
                    icon(
                        data: feather::SEARCH,
                        size: 16,
                        attrs: attributes! {
                            class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-muted-foreground"
                        }
                    )
                    input(
                        attrs: attributes! {
                            type="search"
                            @input=$(|e: Event| query.set(e.target.value))
                            placeholder="Search datasources…"
                            aria-label="Search datasources"
                            class="block min-h-11 pl-10 shadow-xs"
                        }
                    )
                </div>
            </div>

            datasource_results(query: $(query.get()))
        </main>
    })
}

fn source_eligible(source: &PageDesc) -> bool {
    source
        .properties
        .values()
        .any(|prop| prop.field_type == PagePropertyTpye::Date)
}

#[shard]
async fn datasource_results(cx: &Cx, query: String) -> Result<impl View> {
    let api = require_api(cx).await?;
    let sources = api
        .search_datasources(&query)
        .await
        .map_err(topcoat::Error::msg)?;

    struct SourceRow {
        id: String,
        emoji: Option<String>,
        title: String,
        eligible: bool,
        row_class: &'static str,
        tile_class: &'static str,
    }

    let rows: Vec<SourceRow> = sources
        .iter()
        .map(|source| {
            let dto = DatasourceDto::from_page(source);
            let eligible = source_eligible(source);
            SourceRow {
                id: dto.id,
                emoji: dto.icon,
                title: dto.title,
                eligible,
                row_class: if eligible {
                    "flex items-center gap-3 px-4 py-3 transition-colors hover:bg-foreground/5"
                } else {
                    "flex items-center gap-3 px-4 py-3 text-muted-foreground"
                },
                tile_class: if eligible {
                    "flex size-9 shrink-0 items-center justify-center rounded-lg bg-foreground/5 text-base leading-none"
                } else {
                    "flex size-9 shrink-0 items-center justify-center rounded-lg bg-foreground/5 text-base leading-none opacity-60"
                },
            }
        })
        .collect();

    let count = rows.len();
    let count_label = if count == 1 {
        "1 datasource".to_owned()
    } else {
        format!("{count} datasources")
    };

    let (empty_title, empty_hint) = if query.trim().is_empty() {
        (
            "No datasources available.".to_owned(),
            "Share a datasource with your Notion integration to see it here.".to_owned(),
        )
    } else {
        (
            format!("No datasources match “{}”.", query.trim()),
            "Try a different search, or make sure the datasource is shared with your Notion integration."
                .to_owned(),
        )
    };

    Ok(view! {
        <div class="mt-8">
            if count == 0 {
                <div
                    class="rounded-xl border border-border bg-card px-6 py-12 text-center shadow-xs"
                >
                    <div
                        class="mx-auto flex size-10 items-center justify-center rounded-lg bg-foreground/5 text-muted-foreground"
                        aria-hidden="true"
                    >
                        icon(data: feather::DATABASE, size: 18)
                    </div>
                    <p class="mt-4 text-sm font-medium">(empty_title)</p>
                    <p
                        class="mx-auto mt-1 max-w-xs text-xs leading-relaxed text-muted-foreground"
                    >
                        (empty_hint)
                    </p>
                </div>
            } else {
                <p class="mb-2 text-xs font-medium text-muted-foreground tabular-nums">
                    (count_label)
                </p>
                <ul
                    class="divide-y divide-border overflow-hidden rounded-xl border border-border bg-card shadow-xs"
                >
                    for row in rows {
                        <li>
                            if row.eligible {
                                <a
                                    href=(format!("/calendar/{}", row.id))
                                    class=(row.row_class)
                                >
                                    <span class=(row.tile_class) aria-hidden="true">
                                        match row.emoji {
                                            Some(emoji) => (emoji),
                                            None => icon(data: feather::DATABASE, size: 16),
                                        }
                                    </span>
                                    <span class="min-w-0 flex-1 truncate text-sm font-medium">
                                        (row.title)
                                    </span>
                                </a>
                            } else {
                                <div class=(row.row_class)>
                                    <span class=(row.tile_class) aria-hidden="true">
                                        match row.emoji {
                                            Some(emoji) => (emoji),
                                            None => icon(data: feather::DATABASE, size: 16),
                                        }
                                    </span>
                                    <span class="min-w-0 flex-1 truncate text-sm font-medium">
                                        (row.title)
                                    </span>
                                    <span class="shrink-0 text-xs">
                                        "Needs a date property"
                                    </span>
                                </div>
                            }
                        </li>
                    }
                </ul>
            }
        </div>
    })
}

#[procedure]
async fn logout(cx: &Cx) -> Result<()> {
    let repo: &Repo = app_context(cx);
    let session_hash = session::stop(cx).await?;
    if let Some(session_hash) = session_hash {
        repo.delete_session(&session_hash.to_vec())
            .await
            .inspect_err(|e| println!("{e}"))?;
    }
    Ok(())
}

#[procedure]
async fn delete_account(cx: &Cx) -> Result<()> {
    let repo: &Repo = app_context(cx);
    let user = current_user(cx).await?;
    let api = require_api(cx).await?;
    api.revoke_token().await.map_err(topcoat::Error::msg)?;
    repo.delete_user_by_id(user.id).await?;
    session::stop(cx).await?;
    Ok(())
}
