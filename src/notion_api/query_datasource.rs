use crate::notion_api::{BASE_URL, NotionApi, common::RequestStatus, types::page::Page};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasourceQueryRes {
    #[serde(rename = "type")]
    pub response_type: String,
    pub object: String,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub results: Vec<Page>,
    pub request_status: Option<RequestStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatasourceQueryReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    start_cursor: Option<String>,
}

impl NotionApi {
    pub async fn query_datasource(
        &self,
        datasource_id: &str,
        start_cursor: Option<String>,
        date_id: &str,
        title_id: &str,
        description_id: &str,
    ) -> anyhow::Result<DatasourceQueryRes> {
        Ok(self.client
        .post(format!(
            "{}data_sources/{}/query?filter_properties[]={}&filter_properties[]={}&filter_properties[]={}",
            BASE_URL, datasource_id, date_id, title_id, description_id)
        )
        .json(&DatasourceQueryReq{ start_cursor })
        .send()
        .await?
        .error_for_status()?
        .json::<DatasourceQueryRes>()
        .await?)
    }
}
