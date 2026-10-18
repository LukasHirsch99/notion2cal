use chrono::{Duration, Utc};
use icalendar::{Calendar, Component, Event, EventLike};
use topcoat::router::query_params;

use crate::notion_api::{
    NotionApi,
    types::page::{NotionTimestamp, Page, PagePropertyValue},
};

#[query_params(error = bad_request)]
pub struct EventQuery {
    pub datasource_id: String,
    pub title_col: String,
    pub date_col: String,
    pub description_col: Option<String>,
}

pub async fn create_calendar(api: &NotionApi, query: &EventQuery) -> anyhow::Result<Calendar> {
    let mut res = api
        .query_datasource(
            &query.datasource_id,
            None,
            &query.date_col,
            &query.title_col,
            query.description_col.as_ref().unwrap(),
        )
        .await?;

    let mut calendar = Calendar::new();

    while res.has_more {
        for page in res.results {
            if let Some(event) = create_event_from_page(&page, query) {
                calendar.push(event);
            }
        }
        res = api
            .query_datasource(
                &query.datasource_id,
                res.next_cursor,
                &query.date_col,
                &query.title_col,
                query.description_col.as_ref().unwrap(),
            )
            .await?;
    }
    Ok(calendar.done())
}

fn create_event_from_page(page: &Page, query: &EventQuery) -> Option<Event> {
    let title = page.properties.get(&query.title_col)?.value.to_string();

    let mut event = Event::new();
    event.summary(&title);
    event.uid(&page.id);

    if let Some(desc_id) = &query.description_col
        && let Some(desc) = page.properties.get(desc_id)
    {
        event.description(&desc.value.to_string());
    }

    if let Some(PagePropertyValue::Date { date: Some(date) }) =
        page.properties.get(&query.date_col).map(|p| &p.value)
    {
        match (&date.start, &date.end) {
            (NotionTimestamp::DateTime(start), end) => {
                event.starts(start.with_timezone(&Utc));
                if let Some(NotionTimestamp::DateTime(end)) = end {
                    event.ends(end.with_timezone(&Utc));
                }
            }
            (NotionTimestamp::Date(start), Some(NotionTimestamp::Date(e))) => {
                event.starts(*start);
                event.ends(*e + Duration::days(1));
            }
            (NotionTimestamp::Date(s), _) => {
                event.all_day(*s);
            }
        };
    } else {
        return None;
    }

    Some(event)
}
