use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::event_collector::template::Template;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConfig {
    pub date_col: String,
    pub title_template: Template,
    pub description_template: Template,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Calendar {
    pub id: Uuid,
    pub user_id: Uuid,
    pub datasource_id: Uuid,
    pub config: Json<CalendarConfig>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CalendarFeed {
    pub datasource_id: Uuid,
    pub config: Json<CalendarConfig>,
    pub access_token: String,
}

/// Loads the public feed data for a calendar, authenticating to Notion with the
/// owning user's token.
pub async fn find_feed(db: &PgPool, calendar_id: Uuid) -> sqlx::Result<Option<CalendarFeed>> {
    sqlx::query_as(
        r#"
        select c.datasource_id, c.config, u.access_token
        from calendars c
        join users u on u.id = c.user_id
        where c.id = $1
        "#,
    )
    .bind(calendar_id)
    .fetch_optional(db)
    .await
}
