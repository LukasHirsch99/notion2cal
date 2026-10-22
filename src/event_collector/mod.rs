use crate::{
    db::calendar::CalendarConfig,
    notion_api::{
        NotionApi,
        query_datasource::QueryDataSourceRequest,
        types::page::{NotionTimestamp, Page, PagePropertyValue},
    },
};
use chrono::{Duration, Utc};
use icalendar::{Calendar, Component, Event, EventLike};
use topcoat::router::query_params;
use uuid::Uuid;
pub mod template;

#[query_params(error = bad_request)]
pub struct EventQuery {
    pub datasource_id: String,
    pub title_col: String,
    pub date_col: String,
    pub description_col: Option<String>,
}

pub async fn create_calendar(
    api: &NotionApi,
    datasource_id: Uuid,
    config: &CalendarConfig,
) -> anyhow::Result<Calendar> {
    let mut calendar = Calendar::new();
    calendar.name("Notion2Cal");

    let mut cursor: Option<String> = None;
    loop {
        let res = api
            .query_datasource(
                &datasource_id,
                &[],
                QueryDataSourceRequest {
                    page_size: Some(100),
                    start_cursor: cursor,
                    ..Default::default()
                },
            )
            .await?;

        for page in res.results {
            if let Some(event) = create_event_from_page(&page, config) {
                calendar.push(event);
            }
        }

        if !res.has_more {
            break;
        }
        cursor = res.next_cursor;
        if cursor.is_none() {
            break;
        }
    }

    Ok(calendar.done())
}

fn create_event_from_page(page: &Page, config: &CalendarConfig) -> Option<Event> {
    let title = template::render(&config.title_template, &page.properties).ok()?;

    let mut event = Event::new();
    event.summary(&title);
    event.uid(&page.id);

    if !config.description_template.is_empty()
        && let Ok(description) = template::render(&config.description_template, &page.properties)
        && !description.is_empty()
    {
        event.description(&description);
    }

    if let Some(PagePropertyValue::Date { date: Some(date) }) =
        page.properties.get(&config.date_col).map(|p| &p.value)
    {
        match (&date.start, &date.end) {
            (NotionTimestamp::DateTime(start), end) => {
                event.starts(start.with_timezone(&Utc));
                if let Some(NotionTimestamp::DateTime(end)) = end {
                    event.ends(end.with_timezone(&Utc));
                }
            }
            // Named time zones (e.g. "America/Los_Angeles") arrive without an
            // offset and are not mapped to UTC here; treat them as UTC.
            (NotionTimestamp::LocalDateTime(start), end) => {
                event.starts(start.and_utc());
                match end {
                    Some(NotionTimestamp::LocalDateTime(end)) => {
                        event.ends(end.and_utc());
                    }
                    Some(NotionTimestamp::DateTime(end)) => {
                        event.ends(end.with_timezone(&Utc));
                    }
                    _ => {}
                }
            }
            (NotionTimestamp::Date(start), Some(NotionTimestamp::Date(e))) => {
                event.starts(*start);
                event.ends(*e + Duration::days(1));
            }
            (NotionTimestamp::Date(s), _) => {
                event.starts(*s);
                event.ends(*s + Duration::days(1));
            }
        };
    } else {
        return None;
    }

    Some(event)
}
