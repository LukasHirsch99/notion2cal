use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, SecondsFormat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PageTitle {
    pub plain_text: String,
    pub href: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PagePropertyTpye {
    Number,
    Date,
    Title,
    RichText,
    Formula,
    Select,
    MultiSelect,
    Status,
    Relation,
    Rollup,
    UniqueId,
    Url,
    People,
    Files,
    Email,
    PhoneNumber,
    Checkbox,
    CreatedBy,
    CreatedTime,
    LastEditedBy,
    LastEditedTime,
    Button,
    Place,
    Verification,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PagePropertyDesc {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub field_type: PagePropertyTpye,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageProperty {
    pub id: String,
    #[serde(flatten)]
    pub value: PagePropertyValue,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum NotionTimestamp {
    DateTime(DateTime<FixedOffset>),
    LocalDateTime(NaiveDateTime),
    Date(NaiveDate),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateProperty {
    pub start: NotionTimestamp,
    pub end: Option<NotionTimestamp>,
    pub time_zone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TitleProperty {
    pub plain_text: Option<String>,
    pub href: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectOption {
    pub id: Option<String>,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRef {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRef {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelationRef {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UniqueId {
    pub number: Option<u64>,
    pub prefix: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Place {
    pub lat: f64,
    pub lon: f64,
    pub name: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Verification {
    pub state: String,
    pub date: Option<DateProperty>,
    pub verified_by: Option<UserRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Empty {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FormulaValue {
    String {
        string: Option<String>,
    },
    Number {
        number: Option<f32>,
    },
    Boolean {
        boolean: Option<bool>,
    },
    Date {
        date: Option<DateProperty>,
    },
    Unsupported {
        unsupported: Empty,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RollupValue {
    Number {
        number: Option<f32>,
    },
    Date {
        date: Option<DateProperty>,
    },
    Array {
        array: Vec<PagePropertyValue>,
    },
    Unsupported {
        unsupported: Empty,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PagePropertyValue {
    Number {
        number: Option<f32>, // TODO: handle floats
    },
    Date {
        date: Option<DateProperty>,
    },
    Title {
        title: Vec<TitleProperty>,
    },
    RichText {
        rich_text: Vec<TitleProperty>,
    },
    Email {
        email: Option<String>,
    },
    Checkbox {
        checkbox: bool,
    },
    CreatedTime {
        created_time: NotionTimestamp,
    },
    LastEditedTime {
        last_edited_time: NotionTimestamp,
    },
    Formula {
        formula: FormulaValue,
    },
    Select {
        select: Option<SelectOption>,
    },
    MultiSelect {
        multi_select: Vec<SelectOption>,
    },
    Status {
        status: Option<SelectOption>,
    },
    Relation {
        relation: Vec<RelationRef>,
    },
    Rollup {
        rollup: RollupValue,
    },
    UniqueId {
        unique_id: UniqueId,
    },
    Url {
        url: Option<String>,
    },
    People {
        people: Vec<UserRef>,
    },
    Files {
        files: Vec<FileRef>,
    },
    PhoneNumber {
        phone_number: Option<String>,
    },
    CreatedBy {
        created_by: UserRef,
    },
    LastEditedBy {
        last_edited_by: UserRef,
    },
    Place {
        place: Option<Place>,
    },
    Verification {
        verification: Option<Verification>,
    },
    Button {
        button: Empty,
    },
    #[serde(other)]
    Other,
}

fn write_optional<T: Display>(
    f: &mut std::fmt::Formatter<'_>,
    value: Option<&T>,
) -> std::fmt::Result {
    match value {
        Some(value) => write!(f, "{value}"),
        None => Ok(()),
    }
}

fn join_display<T: Display>(items: &[T]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ")
}

fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

impl Display for NotionTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotionTimestamp::DateTime(date_time) => {
                write!(
                    f,
                    "{}",
                    date_time.to_rfc3339_opts(SecondsFormat::AutoSi, true)
                )
            }
            NotionTimestamp::LocalDateTime(date_time) => {
                write!(f, "{}", date_time.format("%Y-%m-%dT%H:%M:%S"))
            }
            NotionTimestamp::Date(date) => write!(f, "{date}"),
        }
    }
}

impl Display for DateProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.end {
            Some(end) => write!(f, "{} – {end}", self.start),
            None => write!(f, "{}", self.start),
        }
    }
}

impl Display for SelectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for UserRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{name}"),
            None => write!(f, "{}", self.id),
        }
    }
}

impl Display for FileRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for RelationRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Display for UniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.prefix, self.number) {
            (Some(prefix), Some(number)) => write!(f, "{prefix}-{number}"),
            (Some(prefix), None) => write!(f, "{prefix}"),
            (None, Some(number)) => write!(f, "{number}"),
            (None, None) => Ok(()),
        }
    }
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{name}")
        } else if let Some(address) = &self.address {
            write!(f, "{address}")
        } else {
            write!(f, "{}, {}", self.lat, self.lon)
        }
    }
}

impl Display for Verification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.state)
    }
}

impl Display for FormulaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormulaValue::String { string } => write_optional(f, string.as_ref()),
            FormulaValue::Number { number } => write_optional(f, number.as_ref()),
            FormulaValue::Boolean { boolean } => match boolean {
                Some(boolean) => write!(f, "{}", yes_no(*boolean)),
                None => Ok(()),
            },
            FormulaValue::Date { date } => write_optional(f, date.as_ref()),
            FormulaValue::Unsupported { .. } | FormulaValue::Other => {
                write!(f, "Type not implemented")
            }
        }
    }
}

impl Display for RollupValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RollupValue::Number { number } => write_optional(f, number.as_ref()),
            RollupValue::Date { date } => write_optional(f, date.as_ref()),
            RollupValue::Array { array } => write!(f, "{}", join_display(array)),
            RollupValue::Unsupported { .. } | RollupValue::Other => {
                write!(f, "Type not implemented")
            }
        }
    }
}

impl Display for PagePropertyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PagePropertyValue::Number { number } => write_optional(f, number.as_ref()),
            PagePropertyValue::Date { date } => write_optional(f, date.as_ref()),
            PagePropertyValue::Title { title } => write!(
                f,
                "{}",
                title
                    .iter()
                    .map(|t| t.plain_text.as_deref().unwrap_or(""))
                    .collect::<Vec<&str>>()
                    .join(", ")
            ),
            PagePropertyValue::RichText { rich_text } => write!(
                f,
                "{}",
                rich_text
                    .iter()
                    .map(|t| t.plain_text.as_deref().unwrap_or(""))
                    .collect::<Vec<&str>>()
                    .join(", ")
            ),
            PagePropertyValue::Email { email } => write_optional(f, email.as_ref()),
            PagePropertyValue::Checkbox { checkbox } => write!(f, "{}", yes_no(*checkbox)),
            PagePropertyValue::CreatedTime { created_time } => write!(f, "{created_time}"),
            PagePropertyValue::LastEditedTime { last_edited_time } => {
                write!(f, "{last_edited_time}")
            }
            PagePropertyValue::Formula { formula } => write!(f, "{formula}"),
            PagePropertyValue::Select { select } => write_optional(f, select.as_ref()),
            PagePropertyValue::MultiSelect { multi_select } => {
                write!(f, "{}", join_display(multi_select))
            }
            PagePropertyValue::Status { status } => write_optional(f, status.as_ref()),
            PagePropertyValue::Relation { relation } => write!(f, "{}", join_display(relation)),
            PagePropertyValue::Rollup { rollup } => write!(f, "{rollup}"),
            PagePropertyValue::UniqueId { unique_id } => write!(f, "{unique_id}"),
            PagePropertyValue::Url { url } => write_optional(f, url.as_ref()),
            PagePropertyValue::People { people } => write!(f, "{}", join_display(people)),
            PagePropertyValue::Files { files } => write!(f, "{}", join_display(files)),
            PagePropertyValue::PhoneNumber { phone_number } => {
                write_optional(f, phone_number.as_ref())
            }
            PagePropertyValue::CreatedBy { created_by } => write!(f, "{created_by}"),
            PagePropertyValue::LastEditedBy { last_edited_by } => write!(f, "{last_edited_by}"),
            PagePropertyValue::Place { place } => write_optional(f, place.as_ref()),
            PagePropertyValue::Verification { verification } => {
                write_optional(f, verification.as_ref())
            }
            PagePropertyValue::Button { .. } => Ok(()),
            PagePropertyValue::Other => write!(f, "Type not implemented"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn property(json: &str) -> PagePropertyValue {
        serde_json::from_str(json).unwrap()
    }

    fn date_property(start: NotionTimestamp, end: Option<NotionTimestamp>) -> PagePropertyValue {
        PagePropertyValue::Date {
            date: Some(DateProperty {
                start,
                end,
                time_zone: None,
            }),
        }
    }

    #[test]
    fn formats_a_number() {
        assert_eq!(
            PagePropertyValue::Number { number: Some(42.0) }.to_string(),
            "42"
        );
    }

    #[test]
    fn formats_an_empty_number_as_blank() {
        assert_eq!(PagePropertyValue::Number { number: None }.to_string(), "");
    }

    #[test]
    fn formats_a_date() {
        let property = date_property(
            NotionTimestamp::Date(NaiveDate::from_ymd_opt(2026, 10, 22).unwrap()),
            None,
        );

        assert_eq!(property.to_string(), "2026-10-22");
    }

    #[test]
    fn formats_a_date_time() {
        let start = DateTime::parse_from_rfc3339("2026-10-22T13:00:00+02:00").unwrap();
        let property = date_property(NotionTimestamp::DateTime(start), None);

        assert_eq!(property.to_string(), "2026-10-22T13:00:00+02:00");
    }

    #[test]
    fn formats_a_named_time_zone_date() {
        let property = property(
            r#"{"type":"date","date":{"start":"2026-09-15T09:00:00","end":null,"time_zone":"America/Los_Angeles"}}"#,
        );

        assert_eq!(property.to_string(), "2026-09-15T09:00:00");
    }

    #[test]
    fn formats_a_date_range() {
        let start = NotionTimestamp::Date(NaiveDate::from_ymd_opt(2026, 10, 22).unwrap());
        let end = NotionTimestamp::Date(NaiveDate::from_ymd_opt(2026, 10, 24).unwrap());

        assert_eq!(
            date_property(start, Some(end)).to_string(),
            "2026-10-22 – 2026-10-24"
        );
    }

    #[test]
    fn formats_an_empty_date_as_blank() {
        assert_eq!(PagePropertyValue::Date { date: None }.to_string(), "");
    }

    #[test]
    fn formats_an_email() {
        assert_eq!(
            PagePropertyValue::Email {
                email: Some("someone@example.com".to_owned())
            }
            .to_string(),
            "someone@example.com"
        );
    }

    #[test]
    fn formats_an_empty_email_as_blank() {
        assert_eq!(PagePropertyValue::Email { email: None }.to_string(), "");
    }

    #[test]
    fn formats_a_checkbox() {
        assert_eq!(
            PagePropertyValue::Checkbox { checkbox: true }.to_string(),
            "Yes"
        );
        assert_eq!(
            PagePropertyValue::Checkbox { checkbox: false }.to_string(),
            "No"
        );
    }

    #[test]
    fn formats_a_created_time() {
        let property =
            property(r#"{"type":"created_time","created_time":"2026-09-01T09:00:00.000Z"}"#);

        assert_eq!(property.to_string(), "2026-09-01T09:00:00Z");
    }

    #[test]
    fn formats_a_url() {
        let property = property(r#"{"type":"url","url":"https://example.com/projects/launch"}"#);

        assert_eq!(property.to_string(), "https://example.com/projects/launch");
    }

    #[test]
    fn formats_an_empty_url_as_blank() {
        assert_eq!(property(r#"{"type":"url","url":null}"#).to_string(), "");
    }

    #[test]
    fn formats_a_select_option() {
        let property =
            property(r#"{"type":"select","select":{"id":"opt","name":"High","color":"red"}}"#);

        assert_eq!(property.to_string(), "High");
    }

    #[test]
    fn formats_an_empty_select_as_blank() {
        assert_eq!(
            property(r#"{"type":"select","select":null}"#).to_string(),
            ""
        );
    }

    #[test]
    fn formats_multi_select_options() {
        let property = property(
            r#"{"type":"multi_select","multi_select":[{"id":"1","name":"High","color":"red"},{"id":"2","name":"Low","color":"blue"}]}"#,
        );

        assert_eq!(property.to_string(), "High, Low");
    }

    #[test]
    fn formats_a_status() {
        let property = property(
            r#"{"type":"status","status":{"id":"stat","name":"In progress","color":"blue"}}"#,
        );

        assert_eq!(property.to_string(), "In progress");
    }

    #[test]
    fn formats_an_empty_status_as_blank() {
        assert_eq!(
            property(r#"{"type":"status","status":null}"#).to_string(),
            ""
        );
    }

    #[test]
    fn formats_relation_ids() {
        let property = property(
            r#"{"type":"relation","relation":[{"id":"page-1"},{"id":"page-2"}],"has_more":false}"#,
        );

        assert_eq!(property.to_string(), "page-1, page-2");
    }

    #[test]
    fn formats_a_number_rollup() {
        let property =
            property(r#"{"type":"rollup","rollup":{"type":"number","number":8,"function":"sum"}}"#);

        assert_eq!(property.to_string(), "8");
    }

    #[test]
    fn formats_a_date_rollup() {
        let property = property(
            r#"{"type":"rollup","rollup":{"type":"date","date":{"start":"2026-09-15","end":null,"time_zone":null},"function":"earliest_date"}}"#,
        );

        assert_eq!(property.to_string(), "2026-09-15");
    }

    #[test]
    fn formats_an_array_rollup() {
        let property = property(
            r#"{"type":"rollup","rollup":{"type":"array","array":[{"type":"number","number":8},{"type":"email","email":"alex@example.com"}],"function":"show_original"}}"#,
        );

        assert_eq!(property.to_string(), "8, alex@example.com");
    }

    #[test]
    fn labels_an_unsupported_rollup() {
        let property = property(
            r#"{"type":"rollup","rollup":{"type":"unsupported","unsupported":{},"function":"sum"}}"#,
        );

        assert_eq!(property.to_string(), "Type not implemented");
    }

    #[test]
    fn formats_a_unique_id() {
        let property =
            property(r#"{"type":"unique_id","unique_id":{"number":42,"prefix":"TASK"}}"#);

        assert_eq!(property.to_string(), "TASK-42");
    }

    #[test]
    fn formats_a_unique_id_without_a_prefix() {
        let property = property(r#"{"type":"unique_id","unique_id":{"number":42,"prefix":null}}"#);

        assert_eq!(property.to_string(), "42");
    }

    #[test]
    fn formats_people_by_name_or_id() {
        let property = property(
            r#"{"type":"people","people":[{"object":"user","id":"user-1","name":"Alex"},{"object":"user","id":"user-2"}]}"#,
        );

        assert_eq!(property.to_string(), "Alex, user-2");
    }

    #[test]
    fn formats_file_names() {
        let property = property(
            r#"{"type":"files","files":[{"name":"Project brief.pdf","type":"external","external":{"url":"https://example.com/project-brief.pdf"}},{"name":"notes.txt"}]}"#,
        );

        assert_eq!(property.to_string(), "Project brief.pdf, notes.txt");
    }

    #[test]
    fn formats_a_phone_number() {
        let property = property(r#"{"type":"phone_number","phone_number":"+1 415 555 0123"}"#);

        assert_eq!(property.to_string(), "+1 415 555 0123");
    }

    #[test]
    fn formats_an_empty_phone_number_as_blank() {
        assert_eq!(
            property(r#"{"type":"phone_number","phone_number":null}"#).to_string(),
            ""
        );
    }

    #[test]
    fn formats_a_created_by_user() {
        let property =
            property(r#"{"type":"created_by","created_by":{"object":"user","id":"user-1"}}"#);

        assert_eq!(property.to_string(), "user-1");
    }

    #[test]
    fn formats_a_last_edited_by_user() {
        let property = property(
            r#"{"type":"last_edited_by","last_edited_by":{"object":"user","id":"user-1","name":"Alex"}}"#,
        );

        assert_eq!(property.to_string(), "Alex");
    }

    #[test]
    fn formats_a_last_edited_time() {
        let property = property(
            r#"{"type":"last_edited_time","last_edited_time":"2026-09-04T14:30:00.000Z"}"#,
        );

        assert_eq!(property.to_string(), "2026-09-04T14:30:00Z");
    }

    #[test]
    fn formats_a_place_by_name() {
        let property = property(
            r#"{"type":"place","place":{"lat":37.7749,"lon":-122.4194,"name":"San Francisco","address":null}}"#,
        );

        assert_eq!(property.to_string(), "San Francisco");
    }

    #[test]
    fn formats_a_place_without_a_name_by_coordinates() {
        let property = property(
            r#"{"type":"place","place":{"lat":37.7749,"lon":-122.4194,"name":null,"address":null}}"#,
        );

        assert_eq!(property.to_string(), "37.7749, -122.4194");
    }

    #[test]
    fn formats_a_verification_state() {
        let property = property(
            r#"{"type":"verification","verification":{"state":"verified","date":null,"verified_by":null}}"#,
        );

        assert_eq!(property.to_string(), "verified");
    }

    #[test]
    fn formats_a_button_as_blank() {
        assert_eq!(property(r#"{"type":"button","button":{}}"#).to_string(), "");
    }

    #[test]
    fn formats_a_formula_string() {
        let property =
            property(r#"{"type":"formula","formula":{"type":"string","string":"Ready"}}"#);

        assert_eq!(property.to_string(), "Ready");
    }

    #[test]
    fn formats_a_formula_number() {
        let property = property(r#"{"type":"formula","formula":{"type":"number","number":16}}"#);

        assert_eq!(property.to_string(), "16");
    }

    #[test]
    fn formats_a_formula_boolean() {
        assert_eq!(
            property(r#"{"type":"formula","formula":{"type":"boolean","boolean":true}}"#)
                .to_string(),
            "Yes"
        );
        assert_eq!(
            property(r#"{"type":"formula","formula":{"type":"boolean","boolean":false}}"#)
                .to_string(),
            "No"
        );
    }

    #[test]
    fn formats_a_formula_date() {
        let property = property(
            r#"{"type":"formula","formula":{"type":"date","date":{"start":"2026-09-15","end":null,"time_zone":null}}}"#,
        );

        assert_eq!(property.to_string(), "2026-09-15");
    }

    #[test]
    fn formats_an_empty_formula_as_blank() {
        let property = property(r#"{"type":"formula","formula":{"type":"number","number":null}}"#);

        assert_eq!(property.to_string(), "");
    }

    #[test]
    fn labels_an_unsupported_formula() {
        let property =
            property(r#"{"type":"formula","formula":{"type":"unsupported","unsupported":{}}}"#);

        assert_eq!(property.to_string(), "Type not implemented");
    }

    #[test]
    fn labels_an_unsupported_type() {
        assert_eq!(PagePropertyValue::Other.to_string(), "Type not implemented");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub title: Option<Vec<PageTitle>>,
    pub in_trash: bool,
    pub url: String,
    pub icon: Option<Icon>,
    pub properties: HashMap<String, PageProperty>,
    // pub object: String,
    // pub created_time: String,
    // pub last_edited_time: String,
    // pub is_archived: Option<bool>,
    // pub is_locked: Option<bool>,
    // pub public_url: Option<String>,
    // pub parent: Parent,
    // pub cover: Option<Cover>,
    // pub created_by: UserRef,
    // pub last_edited_by: UserRef,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageDesc {
    pub id: String,
    pub title: Vec<PageTitle>,
    pub in_trash: bool,
    pub url: String,
    pub icon: Option<Icon>,
    pub properties: HashMap<String, PagePropertyDesc>,
    // pub object: String,
    // pub created_time: String,
    // pub last_edited_time: String,
    // pub is_archived: Option<bool>,
    // pub is_locked: Option<bool>,
    // pub public_url: Option<String>,
    // pub parent: Parent,
    // pub cover: Option<Cover>,
    // pub created_by: UserRef,
    // pub last_edited_by: UserRef,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Icon {
    Emoji {
        emoji: String,
    },
    #[serde(other)]
    Other,
}
