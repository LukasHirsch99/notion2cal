use crate::{
    AppConfig,
    middleware::{create_user, current_user},
    notion_api::{NotionApi, token::TokenRes},
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        error::{SeeOther, see_other},
        query_params, route,
    },
};

#[query_params(error = bad_request)]
pub struct Query {
    pub code: String,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[route(GET)]
async fn auth_redirect(cx: &Cx) -> Result<SeeOther> {
    match current_user(cx).await {
        Some(_) => Ok(see_other("/")),
        _ => {
            let query = query_params::<Query>(cx)?;
            let app_config: &AppConfig = app_context(cx);

            let token_res = NotionApi::get_token(
                &query.code,
                &app_config.redirect_uri,
                &app_config.oauth_client_id,
                &app_config.oauth_client_secret,
                &app_config.api_version,
            )
            .await?;

            match token_res {
                TokenRes::Success {
                    access_token,
                    refresh_token,
                    bot_id,
                    duplicated_template_id: _,
                    workspace_icon,
                    workspace_id,
                    workspace_name,
                    owner,
                } => {
                    create_user(
                        cx,
                        owner,
                        access_token,
                        refresh_token,
                        bot_id,
                        workspace_icon,
                        workspace_id,
                        workspace_name,
                    )
                    .await?;

                    Ok(see_other("/"))
                }
                TokenRes::Error {
                    error,
                    error_description,
                } => {
                    println!("Error: {}", error_description);
                    Err(topcoat::Error::msg(format!(
                        "{}: {}",
                        error, error_description
                    )))
                }
            }
        }
    }
}
