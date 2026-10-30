use crate::notion_api::{NotionApi, common::RequestStatus, types::page::PageDesc};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchReqFilterProperty {
    Object,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchReqFilterValue {
    // Page,
    DataSource,
}

#[derive(Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum SearchReqFilter {
    Property {
        property: SearchReqFilterProperty,
        value: SearchReqFilterValue,
        in_trash: bool,
    },
    // TrashOnly {
    //     in_trash: bool,
    // },
}

#[derive(Serialize)]
pub struct SearchReq {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_cursor: Option<String>,
    page_size: u32,
    filter: SearchReqFilter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRes {
    #[serde(rename = "type")]
    pub response_type: String,
    // pub page_or_data_source: Value,
    pub object: String,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub results: Vec<PageDesc>,
    pub request_status: Option<RequestStatus>,
}

impl NotionApi {
    pub async fn search_datasources(&self, query: &str) -> anyhow::Result<Vec<PageDesc>> {
        Ok(self
            .post(
                "search",
                &SearchReq {
                    query: query.to_string(),
                    start_cursor: None,
                    page_size: 100,
                    filter: SearchReqFilter::Property {
                        property: SearchReqFilterProperty::Object,
                        value: SearchReqFilterValue::DataSource,
                        in_trash: false,
                    },
                },
            )
            .await?
            .json::<SearchRes>()
            .await?
            .results)
    }
}
