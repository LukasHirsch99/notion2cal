use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestStatus {
    #[serde(rename = "type")]
    pub status_type: String,
    pub incomplete_reason: Option<String>,
}
