use reqwest::header;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use topcoat::Result;

use crate::notion_api::{BASE_URL, NotionApi};

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "grant_type")]
pub enum GrantType {
    AuthorizationCode { code: String },
    RefreshToken { refresh_token: String },
}

#[derive(Serialize)]
pub struct TokenBody {
    #[serde(flatten)]
    pub grant_type: GrantType,
    pub redirect_uri: String,
}

#[derive(Deserialize, Debug)]
pub struct Person {
    pub email: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum User {
    Person {
        id: Uuid,
        name: Option<String>,
        avatar_url: Option<String>,
        person: Person,
    },
    AnonymPerson {
        id: Uuid,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Owner {
    User { user: User },
    Workspace { workspace: bool },
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum TokenRes {
    Success {
        access_token: String,
        refresh_token: String,
        bot_id: String,
        duplicated_template_id: Option<String>,
        owner: Owner,
        workspace_icon: Option<String>,
        workspace_id: String,
        workspace_name: Option<String>,
    },
    Error {
        error: String,
        error_description: String,
    },
}

impl NotionApi {
    pub async fn get_token(
        code: &String,
        redirect_uri: &String,
        client_id: &String,
        client_secret: &String,
        api_version: &String,
    ) -> Result<TokenRes> {
        Ok(reqwest::Client::new()
            .post(format!("{}oauth/token", BASE_URL))
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/json")
            .header("Notion-Version", api_version)
            .basic_auth(client_id, Some(client_secret))
            .json(&TokenBody {
                grant_type: GrantType::AuthorizationCode {
                    code: code.to_string(),
                },
                redirect_uri: redirect_uri.to_string(),
            })
            .send()
            .await?
            .json()
            .await?)
    }
}
