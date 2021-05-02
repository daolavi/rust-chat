use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ResponseMessage {
    pub client_id: Uuid,
    pub response_data: ResponseData,
}

impl ResponseMessage {
    pub fn new(client_id: Uuid, response_data: ResponseData) -> Self {
        ResponseMessage {
            client_id,
            response_data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ResponseData {
    Error(ErrorType),
    Alive,
    Joined(JoinedResponse),
    UserJoined(UserJoinedResponse),
    UserLeft,
    Posted(PostedResponse),
    UserPosted(PostedResponse),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostedResponse {
  pub message: MessageResponse
}

impl PostedResponse {
  pub fn new(message: MessageResponse) -> Self {
    PostedResponse {
      message
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserJoinedResponse {
    pub user: UserResponse,
}

impl UserJoinedResponse {
    pub fn new(user: UserResponse) -> Self {
        UserJoinedResponse { user }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JoinedResponse {
    pub user: UserResponse,
    pub other_users: Vec<UserResponse>,
    pub messages: Vec<MessageResponse>,
}

impl JoinedResponse {
    pub fn new(
        user: UserResponse,
        other_users: Vec<UserResponse>,
        messages: Vec<MessageResponse>,
    ) -> Self {
        JoinedResponse {
            user,
            other_users,
            messages,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
}

impl UserResponse {
    pub fn new(id: Uuid, name: &str) -> Self {
        UserResponse {
            id,
            name: String::from(name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub id: Uuid,
    pub user: UserResponse,
    pub text: String,
    pub created_at_utc: DateTime<Utc>,
}

impl MessageResponse {
    pub fn new(id: Uuid, user: UserResponse, text: &str, created_at_utc: DateTime<Utc>) -> Self {
        MessageResponse {
            id,
            user,
            text: String::from(text),
            created_at_utc,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorType {
    NameExisted,
    InvalidName,
    InvalidRequest,
    NotJoined,
    InvalidMessage,
}