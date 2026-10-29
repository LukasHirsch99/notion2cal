use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

use crate::{
    config::Keyring, db::Repo, event_collector::template::Template,
    middleware::encryption::decrypt_token,
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

impl Repo {
    /// Loads the public feed data for a calendar, authenticating to Notion with the
    /// owning user's token.
    pub async fn find_feed(
        &self,
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
        .fetch_optional(&self.db)
        .await?
        .ok_or(anyhow::anyhow!("Could not find calendar {calendar_id}"))?;

        match result.access_token {
            Some(access_token) => {
                let r = decrypt_token(keyring, &access_token, &result.id.to_string())?;
                Ok(CalendarFeed {
                    datasource_id: result.datasource_id,
                    config: result.config,
                    access_token: r,
                })
            }
            None => Err(anyhow::anyhow!("No token for user")),
        }
    }

    /// Lists the calendars owned by a user, newest first.
    pub async fn list_for_user(&self, user_id: Uuid) -> sqlx::Result<Vec<Calendar>> {
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
        .fetch_all(&self.db)
        .await
    }

    /// Loads one of a user's calendars. Calendars owned by anyone else are treated as missing.
    pub async fn find_for_user(
        &self,
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
        .fetch_optional(&self.db)
        .await
    }

    /// Creates a new calendar for a user. Returns `false` when no calendar was created.
    pub async fn create_calendar(
        &self,
        user_id: Uuid,
        datasource_id: Uuid,
        title: &str,
        config: &CalendarConfig,
    ) -> sqlx::Result<Uuid> {
        sqlx::query_scalar!(
            r#"
            insert into calendars (user_id, datasource_id, title, config)
            values ($1, $2, $3, $4)
            returning id
            "#,
            user_id,
            datasource_id,
            title,
            Json(config) as _
        )
        .fetch_one(&self.db)
        .await
    }

    /// Replaces a user's calendar configuration. Returns `false` when no calendar
    /// with that id belongs to the user.
    pub async fn update_calendar(
        &self,
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
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Deletes a user's calendar. Returns `false` when no calendar with that id
    /// belongs to the user.
    pub async fn delete_for_user(&self, calendar_id: Uuid, user_id: Uuid) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            "delete from calendars where id = $1 and user_id = $2",
            calendar_id,
            user_id
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

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

    async fn insert_calendar(repo: &Repo, user_id: Uuid, title: &str) -> Uuid {
        repo.create_calendar(user_id, Uuid::new_v4(), title, &config("Start"))
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
        let repo = Repo::new(pool);
        let user_id = Uuid::new_v4();
        let mine = insert_calendar(&repo, user_id, "Mine").await;
        insert_calendar(&repo, Uuid::new_v4(), "Someone else's").await;

        let calendars = repo.list_for_user(user_id).await.unwrap();

        assert_eq!(calendars.len(), 1);
        assert_eq!(calendars[0].id, mine);
        assert_eq!(calendars[0].title, "Mine");
    }

    #[sqlx::test]
    async fn finds_only_a_users_own_calendar(pool: PgPool) {
        let repo = Repo::new(pool);
        let user_id = Uuid::new_v4();
        let id = insert_calendar(&repo, user_id, "Mine").await;

        assert!(repo.find_for_user(id, user_id).await.unwrap().is_some());
        assert!(
            repo.find_for_user(id, Uuid::new_v4())
                .await
                .unwrap()
                .is_none()
        );
    }

    #[sqlx::test]
    async fn updates_only_a_users_own_calendar(pool: PgPool) {
        let repo = Repo::new(pool);
        let user_id = Uuid::new_v4();
        let id = insert_calendar(&repo, user_id, "Mine").await;
        let updated = config("End");

        assert!(
            !repo
                .update_calendar(id, Uuid::new_v4(), "Stolen", &updated)
                .await
                .unwrap()
        );
        assert_eq!(
            repo.find_for_user(id, user_id)
                .await
                .unwrap()
                .unwrap()
                .title,
            "Mine"
        );

        assert!(
            repo.update_calendar(id, user_id, "Renamed", &updated)
                .await
                .unwrap()
        );
        let calendar = repo.find_for_user(id, user_id).await.unwrap().unwrap();
        assert_eq!(calendar.title, "Renamed");
        assert_eq!(calendar.config.0.date_col, "End");
    }

    #[sqlx::test]
    async fn deletes_only_a_users_own_calendar(pool: PgPool) {
        let repo = Repo::new(pool);
        let user_id = Uuid::new_v4();
        let id = insert_calendar(&repo, user_id, "Mine").await;

        assert!(!repo.delete_for_user(id, Uuid::new_v4()).await.unwrap());
        assert!(repo.find_for_user(id, user_id).await.unwrap().is_some());

        assert!(repo.delete_for_user(id, user_id).await.unwrap());
        assert!(repo.find_for_user(id, user_id).await.unwrap().is_none());
    }
}
