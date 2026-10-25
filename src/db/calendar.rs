use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::{
    config::Keyring, event_collector::template::Template, middleware::encryption::decrypt_token,
};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CalendarConfig {
    pub date_col: String,
    pub title_template: Template,
    pub description_template: Template,
    #[serde(default)]
    pub location_template: Template,
    #[serde(default)]
    pub url_template: Template,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Calendar {
    pub id: Uuid,
    pub datasource_id: Uuid,
    pub title: String,
    pub config: Json<CalendarConfig>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CalendarFeed {
    pub datasource_id: Uuid,
    pub config: Json<CalendarConfig>,
    pub access_token: SecretString,
}

/// Loads the public feed data for a calendar, authenticating to Notion with the
/// owning user's token.
pub async fn find_feed(
    db: &PgPool,
    keyring: &Keyring,
    calendar_id: Uuid,
) -> anyhow::Result<CalendarFeed> {
    let result = sqlx::query!(
        r#"
        select c.datasource_id, c.config as "config: Json<CalendarConfig>", u.access_token, u.id
        from calendars c
        join users u on u.id = c.user_id
        where c.id = $1
        "#,
        calendar_id
    )
    .fetch_optional(db)
    .await?
    .ok_or(anyhow::anyhow!("Could not find calendar {calendar_id}"))?;

    let r = decrypt_token(keyring, &result.access_token, &result.id.to_string())?;
    Ok(CalendarFeed {
        datasource_id: result.datasource_id,
        config: result.config,
        access_token: r,
    })
}

/// Lists the calendars owned by a user, newest first.
pub async fn list_for_user(db: &PgPool, user_id: Uuid) -> sqlx::Result<Vec<Calendar>> {
    sqlx::query_as!(
        Calendar,
        r#"
        select id, datasource_id, title, config as "config: Json<CalendarConfig>"
        from calendars
        where user_id = $1
        order by created_at desc
        "#,
        user_id
    )
    .fetch_all(db)
    .await
}

/// Loads one of a user's calendars. Calendars owned by anyone else are treated as missing.
pub async fn find_for_user(
    db: &PgPool,
    calendar_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<Calendar>> {
    sqlx::query_as!(
        Calendar,
        r#"
        select id, datasource_id, title, config as "config: Json<CalendarConfig>"
        from calendars
        where id = $1 and user_id = $2
        "#,
        calendar_id,
        user_id
    )
    .fetch_optional(db)
    .await
}

/// Creates a new calendar for a user. Returns `false` when no calendar was created.
pub async fn create_calendar(
    db: &PgPool,
    user_id: Uuid,
    datasource_id: Uuid,
    title: &str,
    config: &CalendarConfig,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        r#"
        insert into calendars (user_id, datasource_id, title, config)
        values ($1, $2, $3, $4)
        "#,
        user_id,
        datasource_id,
        title,
        Json(config) as _
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Replaces a user's calendar configuration. Returns `false` when no calendar
/// with that id belongs to the user.
pub async fn update_calendar(
    db: &PgPool,
    calendar_id: Uuid,
    user_id: Uuid,
    title: &str,
    config: &CalendarConfig,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        r#"
        update calendars
        set title = $3, config = $4
        where id = $1 and user_id = $2
        "#,
        calendar_id,
        user_id,
        title,
        Json(config) as _
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Deletes a user's calendar. Returns `false` when no calendar with that id
/// belongs to the user.
pub async fn delete_for_user(db: &PgPool, calendar_id: Uuid, user_id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "delete from calendars where id = $1 and user_id = $2",
        calendar_id,
        user_id
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_collector::template::Segment;

    fn config(date_col: &str) -> CalendarConfig {
        CalendarConfig {
            date_col: date_col.to_owned(),
            title_template: vec![Segment::Text("Event".to_owned())],
            description_template: Vec::new(),
            location_template: Vec::new(),
            url_template: Vec::new(),
        }
    }

    async fn insert_calendar(pool: &PgPool, user_id: Uuid, title: &str) -> Uuid {
        sqlx::query_scalar(
            r#"
            insert into calendars (user_id, datasource_id, title, config)
            values ($1, $2, $3, $4)
            returning id
            "#,
        )
        .bind(user_id)
        .bind(Uuid::new_v4())
        .bind(title)
        .bind(Json(config("Start")))
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[test]
    fn loads_calendars_saved_before_location_and_url_templates() {
        let config: CalendarConfig = serde_json::from_str(
            r#"{"date_col":"Start","title_template":[{"Text":"Event"}],"description_template":[]}"#,
        )
        .unwrap();

        assert!(config.location_template.is_empty());
        assert!(config.url_template.is_empty());
    }

    #[sqlx::test]
    async fn lists_only_the_users_calendars(pool: PgPool) {
        let user_id = Uuid::new_v4();
        let mine = insert_calendar(&pool, user_id, "Mine").await;
        insert_calendar(&pool, Uuid::new_v4(), "Someone else's").await;

        let calendars = list_for_user(&pool, user_id).await.unwrap();

        assert_eq!(calendars.len(), 1);
        assert_eq!(calendars[0].id, mine);
        assert_eq!(calendars[0].title, "Mine");
    }

    #[sqlx::test]
    async fn finds_only_a_users_own_calendar(pool: PgPool) {
        let user_id = Uuid::new_v4();
        let id = insert_calendar(&pool, user_id, "Mine").await;

        assert!(find_for_user(&pool, id, user_id).await.unwrap().is_some());
        assert!(
            find_for_user(&pool, id, Uuid::new_v4())
                .await
                .unwrap()
                .is_none()
        );
    }

    #[sqlx::test]
    async fn updates_only_a_users_own_calendar(pool: PgPool) {
        let user_id = Uuid::new_v4();
        let id = insert_calendar(&pool, user_id, "Mine").await;
        let updated = config("End");

        assert!(
            !update_calendar(&pool, id, Uuid::new_v4(), "Stolen", &updated)
                .await
                .unwrap()
        );
        assert_eq!(
            find_for_user(&pool, id, user_id)
                .await
                .unwrap()
                .unwrap()
                .title,
            "Mine"
        );

        assert!(
            update_calendar(&pool, id, user_id, "Renamed", &updated)
                .await
                .unwrap()
        );
        let calendar = find_for_user(&pool, id, user_id).await.unwrap().unwrap();
        assert_eq!(calendar.title, "Renamed");
        assert_eq!(calendar.config.0.date_col, "End");
    }

    #[sqlx::test]
    async fn deletes_only_a_users_own_calendar(pool: PgPool) {
        let user_id = Uuid::new_v4();
        let id = insert_calendar(&pool, user_id, "Mine").await;

        assert!(!delete_for_user(&pool, id, Uuid::new_v4()).await.unwrap());
        assert!(find_for_user(&pool, id, user_id).await.unwrap().is_some());

        assert!(delete_for_user(&pool, id, user_id).await.unwrap());
        assert!(find_for_user(&pool, id, user_id).await.unwrap().is_none());
    }
}
