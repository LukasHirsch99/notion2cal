use reqwest::{Response, header};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

pub mod common;
pub mod query_datasource;
pub mod search_by_title;
pub mod token;
pub mod types;

pub const BASE_URL: &str = "https://api.notion.com/v1";

#[derive(Clone)]
pub struct NotionApi {
    client: reqwest::Client,
    token: SecretString,
}

impl NotionApi {
    pub fn new(token: SecretString, notion_version: &str) -> NotionApi {
        let mut headers = header::HeaderMap::new();
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
        NotionApi { client, token }
    }

    async fn get(&self, path: &str) -> reqwest::Result<Response> {
        self.client
            .get(format!("{BASE_URL}/{path}"))
            .bearer_auth(self.token.expose_secret())
            .send()
            .await
    }

    async fn post<T>(&self, path: &str, json: &T) -> reqwest::Result<Response>
    where
        T: Serialize + ?Sized,
    {
        self.client
            .post(format!("{BASE_URL}/{path}"))
            .bearer_auth(self.token.expose_secret())
            .json(json)
            .send()
            .await
    }
}
