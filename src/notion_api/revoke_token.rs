use secrecy::ExposeSecret;
use serde::Serialize;

use crate::notion_api::NotionApi;

#[derive(Serialize)]
struct RevokeBody {
    token: String,
}

impl NotionApi {
    pub async fn revoke_token(&self) -> anyhow::Result<()> {
        self.post(
            "oauth/revoke",
            &RevokeBody {
                token: self.token.expose_secret().to_owned(),
            },
        )
        .await?;
        Ok(())
    }
}
