use crate::{
    components::{button::{ButtonSize, ButtonVariant, button_variants}, theme_button::theme_button}, config::Config,
};
use chrono::{Datelike, NaiveDate};
use topcoat::{
    Result,
    context::{Cx, app_context},
    view::{View, component, view},
};

#[component]
pub async fn welcome(cx: &Cx) -> Result<impl View> {
    let app_config: &Config = app_context(cx);

    // Month preview for the calendar card.
    let today = chrono::Local::now().date_naive();
    let month_start = today.with_day(1).expect("first day of month");
    let next_month = if today.month() == 12 {
        NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
    }
    .expect("first day of next month");
    let days_in_month = (next_month - month_start).num_days() as usize;
    let leading_blanks = month_start.weekday().num_days_from_monday() as usize;
    let total_cells = (leading_blanks + days_in_month).div_ceil(7) * 7;

    // Days that carry a dated page from a Notion database.
    let events = [
        (4, "Blog draft", "Content calendar"),
        (9, "Team sync", "Ops"),
        (16, "Launch day", "Content calendar"),
        (23, "Invoice due", "Finance"),
    ];

    struct DayCell {
        label: String,
        class: &'static str,
        dot_class: &'static str,
    }

    let cells: Vec<DayCell> = (0..total_cells)
        .map(|index| {
            if index < leading_blanks || index >= leading_blanks + days_in_month {
                return DayCell {
                    label: String::new(),
                    class: "h-10",
                    dot_class: "size-1 opacity-0",
                };
            }

            let day = index - leading_blanks + 1;
            let is_today = day == today.day() as usize;
            let has_event = events.iter().any(|(event_day, ..)| *event_day == day);

            let class = if is_today {
                "flex h-10 flex-col items-center justify-center gap-0.5 rounded-lg bg-primary text-sm font-semibold text-primary-foreground"
            } else if has_event {
                "flex h-10 flex-col items-center justify-center gap-0.5 rounded-lg text-sm font-medium text-foreground"
            } else {
                "flex h-10 flex-col items-center justify-center gap-0.5 text-sm text-foreground"
            };
            let dot_class = match (has_event, is_today) {
                (true, true) => "size-1 rounded-full bg-primary-foreground",
                (true, false) => "size-1 rounded-full bg-primary",
                (false, _) => "size-1 rounded-full opacity-0",
            };

            DayCell {
                label: day.to_string(),
                class,
                dot_class,
            }
        })
        .collect();

    let (next_event_day, next_event_title, next_event_source) = *events
        .iter()
        .find(|(day, ..)| *day >= today.day() as usize)
        .unwrap_or(&events[0]);
    let next_event_date = month_start
        .with_day(next_event_day as u32)
        .expect("event day within month");
    let next_event_when = next_event_date.format("%a, %b %-d").to_string();

    Ok(view! {
        <main class="mx-auto flex min-h-[calc(100vh-2.5rem)] w-full flex-col">
            <div
                class="grid flex-1 content-center items-center gap-14 pb-20 md:grid-cols-[1.1fr_0.9fr] md:gap-10 md:pb-6"
            >
                <section class="max-w-xl">
                    <h1
                        class="text-4xl leading-[1.05] font-semibold tracking-tight sm:text-5xl lg:text-6xl"
                    >
                        "Your Notion dates, on your calendar."
                    </h1>
                    <p
                        class="mt-6 max-w-md text-base leading-relaxed text-muted-foreground"
                    >
                        "Notion2Cal turns the dated pages in your databases into a calendar feed. Add a page, move a date — your calendar follows."
                    </p>
                    <div class="mt-9 flex flex-col items-start gap-3">
                        <a
                            href=(app_config.notion_auth_url.clone())
                            class=(button_variants(
                                ButtonVariant::Primary,
                                ButtonSize::Lg,
                            ))
                        >
                            "Continue with Notion"
                        </a>
                        <p class="text-xs text-muted-foreground">
                            "Read-only access. Disconnect anytime."
                        </p>
                    </div>
                </section>

                <aside
                    class="w-full max-w-sm justify-self-center md:justify-self-end"
                    aria-label="Calendar preview"
                >
                    <div class="rounded-2xl border border-border bg-card p-5 shadow-sm">
                        <div class="mb-5 flex items-baseline justify-between">
                            <h2 class="text-sm font-semibold tracking-tight">
                                (month_start.format("%B %Y").to_string())
                            </h2>
                            <span class="text-xs text-muted-foreground">
                                "From Notion"
                            </span>
                        </div>

                        <div
                            class="mb-1 grid grid-cols-7 text-center text-xs font-medium text-muted-foreground"
                        >
                            <span>"Mo"</span>
                            <span>"Tu"</span>
                            <span>"We"</span>
                            <span>"Th"</span>
                            <span>"Fr"</span>
                            <span>"Sa"</span>
                            <span>"Su"</span>
                        </div>

                        <div
                            class="grid grid-cols-7 gap-y-1 text-center"
                            aria-hidden="true"
                        >
                            for cell in cells {
                                <div class=(cell.class)>
                                    <span>(cell.label)</span>
                                    <span class=(cell.dot_class)></span>
                                </div>
                            }
                        </div>

                        <div
                            class="mt-4 flex items-center justify-between border-t border-border pt-4"
                        >
                            <div class="flex items-center gap-2">
                                <span class="size-1.5 rounded-full bg-primary"></span>
                                <span class="text-sm font-medium">(next_event_title)</span>
                                <span class="text-xs text-muted-foreground">
                                    (next_event_source)
                                </span>
                            </div>
                            <span class="text-xs text-muted-foreground">
                                (next_event_when)
                            </span>
                        </div>
                    </div>
                </aside>
            </div>
        </main>
    })
}
