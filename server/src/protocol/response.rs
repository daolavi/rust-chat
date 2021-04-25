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
      response_data
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ResponseData {
    Error(ErrorType),
    Alive,
    Joined,
    UserJoined,
    UserLeft,
    Posted,
    UserPosted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorType {
  NameExisted,
  InvalidName,
  InvalidRequest
}