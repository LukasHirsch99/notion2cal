use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PageTitle {
    pub plain_text: String,
    pub href: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    Date(NaiveDate),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateProperty {
    pub start: NotionTimestamp,
    pub end: Option<NotionTimestamp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TitleProperty {
    pub plain_text: Option<String>,
    pub href: Option<String>,
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
    #[serde(other)]
    Other,
}

impl Display for PagePropertyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
            _ => write!(f, ""),
        }
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

impl PageDesc {
    pub fn title_str(&self) -> String {
        let icon = match &self.icon {
            Some(Icon::Emoji { emoji }) => emoji.clone() + " - ",
            _ => String::new(),
        };
        icon + self
            .title
            .iter()
            .map(|t| t.plain_text.as_ref())
            .collect::<Vec<&str>>()
            .join(", ")
            .as_str()
    }
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
