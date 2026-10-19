use reqwest::{StatusCode, header};
use serde::{Deserialize, Serialize};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{query_params, response::IntoResponse, route},
};

use crate::NotionConnectionSecrets;

#[query_params(error = bad_request)]
struct Query {
    code: String,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "grant_type")]
enum GrantType {
    AuthorizationCode { code: String },
    RefreshToken { refresh_token: String },
}

#[derive(Serialize)]
struct TokenBody {
    #[serde(flatten)]
    grant_type: GrantType,
    redirect_uri: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum TokenResErrorCode {
    InvalidRequest,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
    InvalidClient,
    TestEnvError,
    InternalServerError,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TokenRes {
    Success {
        access_token: String,
        refresh_token: String,
        bot_id: String,
        duplicated_template_id: Option<String>,
        // owner: Owner,
        workspace_icon: Option<String>,
        workspace_id: String,
        workspace_name: Option<String>,
    },
    Error {
        message: String,
        status: u32,
        code: TokenResErrorCode,
    },
}

#[route(GET)]
async fn auth_redirect(cx: &Cx) -> Result<impl IntoResponse> {
    let query = query_params::<Query>(cx)?;
    let secrets: &NotionConnectionSecrets = app_context(cx);

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.notion.com/v1/oauth/token")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .basic_auth(&secrets.oauth_client_id, Some(&secrets.oauth_client_secret))
        .json(&TokenBody {
            grant_type: GrantType::AuthorizationCode {
                code: query.code.clone(),
            },
            redirect_uri: secrets.redirect_uri.clone(),
        })
        .send()
        .await?
        .json::<TokenRes>()
        .await?;

    match res {
        TokenRes::Success {
            access_token,
            refresh_token,
            bot_id,
            duplicated_template_id,
            workspace_icon,
            workspace_id,
            workspace_name,
        } => {
            // save in DB
            println!(
                "access_token: {}, refresh_token: {}",
                access_token, refresh_token
            );
            Ok(StatusCode::OK)
        }
        TokenRes::Error {
            message,
            status: _,
            code: _,
        } => Err(topcoat::Error::msg(message)),
    }
}
