use serde::{Deserialize, Serialize};

use crate::notion_api::types::page::{Icon, PageDesc};


#[derive(Debug, Serialize, Deserialize)]
pub struct DatasourceDto {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub url: String,
}

impl DatasourceDto {
    pub fn from_page(page: &PageDesc) -> DatasourceDto {
        let icon = match &page.icon {
            Some(Icon::Emoji { emoji }) => Some(emoji.clone()),
            _ => None,
        };
        DatasourceDto {
            id: page.id.clone(),
            title: page
                .title
                .iter()
                .fold(String::new(), |acc, e| acc + e.plain_text.as_str()),
            icon: icon,
            url: page.url.clone(),
        }
    }
}
