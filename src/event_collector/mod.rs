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
use uuid::Uuid;
pub mod template;

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
    let title = template::render(&config.title_template, &page.properties, &page.url).ok()?;

    let mut event = Event::new();
    event.summary(&title);
    event.uid(&page.id);

    if !config.description_template.is_empty()
        && let Ok(description) =
            template::render(&config.description_template, &page.properties, &page.url)
        && !description.is_empty()
    {
        event.description(&description);
    }

    if !config.location_template.is_empty()
        && let Ok(location) =
            template::render(&config.location_template, &page.properties, &page.url)
        && !location.is_empty()
    {
        event.location(&location);
    }

    if !config.url_template.is_empty()
        && let Ok(url) = template::render(&config.url_template, &page.properties, &page.url)
        && !url.is_empty()
    {
        event.url(&url);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        event_collector::template::{Segment, Template},
        notion_api::types::page::{DateProperty, PageProperty, TitleProperty},
    };
    use chrono::NaiveDate;
    use std::collections::HashMap;

    fn page() -> Page {
        let mut properties = HashMap::new();
        properties.insert(
            "When".to_owned(),
            PageProperty {
                id: "date".to_owned(),
                value: PagePropertyValue::Date {
                    date: Some(DateProperty {
                        start: NotionTimestamp::Date(
                            NaiveDate::from_ymd_opt(2026, 10, 22).unwrap(),
                        ),
                        end: None,
                        time_zone: None,
                    }),
                },
            },
        );
        properties.insert(
            "Name".to_owned(),
            PageProperty {
                id: "title".to_owned(),
                value: PagePropertyValue::Title {
                    title: vec![TitleProperty {
                        plain_text: Some("Standup".to_owned()),
                        href: None,
                    }],
                },
            },
        );

        Page {
            id: "page-id".to_owned(),
            title: None,
            in_trash: false,
            url: "https://notion.so/page".to_owned(),
            icon: None,
            properties,
        }
    }

    fn template(src: &str) -> Template {
        if src.is_empty() {
            Template::new()
        } else {
            vec![Segment::Field(src.to_owned())]
        }
    }

    fn config(location_template: Template, url_template: Template) -> CalendarConfig {
        CalendarConfig {
            date_col: "When".to_owned(),
            title_template: template("Name"),
            description_template: Template::new(),
            location_template,
            url_template,
        }
    }

    #[test]
    fn adds_the_rendered_location_and_url_to_the_event() {
        let event =
            create_event_from_page(&page(), &config(template("Name"), template("Page URL")))
                .unwrap();

        assert_eq!(event.get_location(), Some("Standup"));
        assert_eq!(event.get_url(), Some("https://notion.so/page"));
    }

    #[test]
    fn omits_empty_location_and_url() {
        let event =
            create_event_from_page(&page(), &config(Template::new(), Template::new())).unwrap();

        assert_eq!(event.get_location(), None);
        assert_eq!(event.get_url(), None);
    }
}
