use crate::{
    components::input::input,
    dto::DatasourceDto,
    middleware::{current_api, require_auth},
};
use topcoat::{
    Result,
    context::Cx,
    icon::{IconData, icon},
    router::error::RouterErrorExt,
    runtime::{Event, shard, signal},
    view::{View, attributes, component, svg::ViewBox, view},
};

const SEARCH: IconData = IconData::unescaped_unchecked(
    ViewBox::new(0.0, 0.0, 24.0, 24.0),
    r#"<path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" d="M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14Zm9 16-4.35-4.35"/>"#,
);

const DATASOURCE: IconData = IconData::unescaped_unchecked(
    ViewBox::new(0.0, 0.0, 24.0, 24.0),
    r#"<path fill="currentColor" d="M12,3C7.58,3 4,4.34 4,6V18C4,19.66 7.58,21 12,21C16.42,21 20,19.66 20,18V6C20,4.34 16.42,3 12,3M12,5C15.87,5 18,5.9 18,6C18,6.1 15.87,7 12,7C8.13,7 6,6.1 6,6C6,5.9 8.13,5 12,5M18,10.5C17.1,11.1 14.9,11.5 12,11.5C9.1,11.5 6.9,11.1 6,10.5V8.9C7.5,9.5 9.7,9.8 12,9.8C14.3,9.8 16.5,9.5 18,8.9V10.5M18,15C17.1,15.6 14.9,16 12,16C9.1,16 6.9,15.6 6,15V13.5C7.5,14.1 9.7,14.4 12,14.4C14.3,14.4 16.5,14.1 18,13.5V15M18,18.1C17.1,18.7 14.9,19.1 12,19.1C9.1,19.1 6.9,18.7 6,18.1V17C7.5,17.6 9.7,17.9 12,17.9C14.3,17.9 16.5,17.6 18,17V18.1Z"/>"#,
);

/// The logged-in home page: search Notion datasources and list them.
#[component]
pub async fn search(cx: &Cx) -> Result<impl View> {
    let query = signal(cx, String::new);
    let user = require_auth(cx).await?;
    let username = user.username.clone().unwrap_or_else(|| "there".to_owned());

    Ok(view! {
        <main class="mx-auto w-full max-w-2xl pt-10 pb-16 sm:pt-20">
            <header>
                <h1 class="text-2xl font-semibold tracking-tight sm:text-3xl">
                    "Hello, "
                    (username)
                    "."
                </h1>
                <p class="mt-2 text-sm text-muted-foreground sm:text-base">
                    "Choose a Notion datasource to turn into a calendar."
                </p>
            </header>

            <div class="relative mt-8">
                icon(
                    data: SEARCH,
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

            datasource_results(query: $(query.get()))
        </main>
    })
}

#[shard]
async fn datasource_results(cx: &Cx, query: String) -> Result<impl View> {
    let api = current_api(cx).await.ok_or_unauthorized()?;
    let sources = api
        .search_datasources(&query)
        .await
        .map_err(topcoat::Error::msg)?;

    let rows: Vec<(Option<String>, String)> = sources
        .iter()
        .map(|source| {
            let dto = DatasourceDto::from_page(source);
            (dto.icon, dto.title)
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
                        icon(data: DATASOURCE, size: 18)
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
                    for (emoji, title) in rows {
                        <li
                            class="flex items-center gap-3 px-4 py-3 transition-colors hover:bg-foreground/5"
                        >
                            <span
                                class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-foreground/5 text-base leading-none"
                                aria-hidden="true"
                            >
                                match emoji {
                                    Some(emoji) => (emoji),
                                    None => icon(data: DATASOURCE, size: 16),
                                }
                            </span>
                            <span class="min-w-0 flex-1 truncate text-sm font-medium">
                                (title)
                            </span>
                        </li>
                    }
                </ul>
            }
        </div>
    })
}
