use std::{borrow::Borrow, fmt};

use reqwest::{Response, StatusCode, header};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

pub mod common;
pub mod query_datasource;
pub mod revoke_token;
pub mod search_by_title;
pub mod token;
pub mod types;

const BASE_URL: &str = "https://api.notion.com/v1";

#[derive(Debug)]
pub enum NotionApiError {
    Unauthorized,
}

impl fmt::Display for NotionApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotionApiError::Unauthorized => f.write_str("Notion API returned 401 Unauthorized"),
        }
    }
}

impl std::error::Error for NotionApiError {}

impl NotionApiError {
    /// True when the error, including anything wrapped by anyhow, is a Notion 401.
    pub fn is_unauthorized(error: &anyhow::Error) -> bool {
        matches!(error.downcast_ref::<Self>(), Some(Self::Unauthorized))
    }
}

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

        let mut auth =
            header::HeaderValue::from_str(&format!("Bearer {}", token.expose_secret())).unwrap();
        auth.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        NotionApi { client, token }
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> anyhow::Result<Response> {
        let response = request.send().await.map_err(|e| anyhow::anyhow!("{e}"))?;
        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(NotionApiError::Unauthorized.into());
        }
        Ok(response.error_for_status()?)
    }

    async fn get(&self, path: &str) -> anyhow::Result<Response> {
        self.send(
            self.client
                .get(format!("{BASE_URL}/{path}"))
                .bearer_auth(self.token.expose_secret()),
        )
        .await
    }

    async fn post<T>(&self, path: &str, json: &T) -> anyhow::Result<Response>
    where
        T: Serialize + ?Sized,
    {
        self.send(
            self.client
                .post(format!("{BASE_URL}/{path}"))
                .bearer_auth(self.token.expose_secret())
                .json(json),
        )
        .await
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
        self.send(
            self.client
                .post(url)
                .bearer_auth(self.token.expose_secret())
                .json(json),
        )
        .await
    }
}
