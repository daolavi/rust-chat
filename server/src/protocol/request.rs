use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct RequestMessage {
    pub client_id: Uuid,
    pub request_data: RequestData,
}

impl RequestMessage {
    pub fn new(client_id: Uuid, request_data: RequestData) -> Self {
        RequestMessage { client_id, request_data }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum RequestData {
    Join(JoinRequestData),
    PostMessage(PostMessageRequestData),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JoinRequestData {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostMessageRequestData {
    pub text: String,
}