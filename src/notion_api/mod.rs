use reqwest::header;

pub mod common;
pub mod query_datasource;
pub mod search_by_title;
pub mod token;
pub mod types;

pub const BASE_URL: &str = "https://api.notion.com/v1/";

#[derive(Clone)]
pub struct NotionApi {
    client: reqwest::Client,
}

impl NotionApi {
    pub fn new(token: &str, notion_version: &str) -> NotionApi {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(("Bearer ".to_string() + token).as_str()).unwrap(),
        );
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Notion-Version",
            header::HeaderValue::from_str(notion_version).unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        NotionApi { client }
    }
}
