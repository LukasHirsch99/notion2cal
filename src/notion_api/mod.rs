use std::borrow::Borrow;

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

    async fn post<T>(&self, path: &str, json: &T) -> anyhow::Result<Response>
    where
        T: Serialize + ?Sized,
    {
        self.client
            .post(format!("{BASE_URL}/{path}"))
            .bearer_auth(self.token.expose_secret())
            .json(json)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn post_with_params<T, I, K, V>(
        &self,
        path: &str,
        json: &T,
        params: Option<I>,
    ) -> anyhow::Result<Response>
    where
        T: Serialize + ?Sized,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let base_url = format!("{BASE_URL}/{path}");
        let url = match params {
            Some(params) => reqwest::Url::parse_with_params(&base_url, params)
                .map_err(|_| anyhow::anyhow!("Could not parse url {base_url}"))?,
            None => reqwest::Url::parse(&base_url)
                .map_err(|_| anyhow::anyhow!("Could not parse url {base_url}"))?,
        };
        self.client
            .post(url)
            .bearer_auth(self.token.expose_secret())
            .json(json)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}
