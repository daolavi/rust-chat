use std::{collections::HashMap, time::Duration};
use regex::Regex;
use tokio::sync::{broadcast, mpsc::UnboundedReceiver, RwLock};

use crate::{
    model::{feed::Feed, user::User},
    protocol::{
        request::{JoinRequestData, PostMessageRequestData, RequestMessage},
        response::{ErrorType, ResponseData, ResponseMessage},
    },
};
use uuid::Uuid;

lazy_static! {
  static ref USER_NAME_REGEX: Regex = Regex::new("[A-Za-z\\s]{4,24}").unwrap();
}

pub struct Worker {
    pub alive_interval: Option<Duration>,
    pub response_sender: broadcast::Sender<ResponseMessage>,
    pub users: RwLock<HashMap<Uuid, User>>,
    pub feed: RwLock<Feed>,
}

impl Worker {
    pub fn new(duration: Option<Duration>) -> Self {
        let (sender, _) = broadcast::channel(16);
        Worker {
            alive_interval: duration,
            response_sender: sender,
            users: Default::default(),
            feed: Default::default(),
        }
    }

    pub fn run(&self, receiver: UnboundedReceiver<RequestMessage>) {}

    async fn process_join(&self, client_id: Uuid, join_request_data: JoinRequestData) {
        let user_name = join_request_data.name.trim();
        if self
            .users
            .read()
            .await
            .values()
            .any(|user| user.name == user_name)
        {
          self.send_error(client_id, ErrorType::NameExisted);
          return;
        }

        if !USER_NAME_REGEX.is_match(user_name) {
          self.send_error(client_id, ErrorType::InvalidName);
          return;
        }
    }

    async fn process_post(
        &self,
        client_id: Uuid,
        post_message_request_data: PostMessageRequestData,
    ) {
    }

    fn send_error(&self, client_id: Uuid, error_type: ErrorType) {
        self.send_message_to_client(client_id, ResponseData::Error(error_type))
    }

    fn send_message_to_client(&self, client_id: Uuid, response_data: ResponseData) {
        if self.response_sender.receiver_count() > 0 {
            self.response_sender
                .send(ResponseMessage::new(client_id, response_data))
                .unwrap();
        }
    }
}
