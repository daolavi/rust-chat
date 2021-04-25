use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct RequestParcel {
    pub client_id: Uuid,
    pub request: Request,
}

impl RequestParcel {
    pub fn new(client_id: Uuid, request: Request) -> Self {
        RequestParcel { client_id, request }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    Join(JoinRequest),
    PostMessage(PostMessageRequest),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JoinRequest {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostMessageRequest {
    pub text: String,
}
