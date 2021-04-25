use std::{collections::HashMap, time::Duration};
use tokio::sync::{RwLock, broadcast, mpsc::UnboundedReceiver};

use crate::{model::{feed::Feed, user::User}, protocol::{request::{JoinRequestData, PostMessageRequestData, RequestMessage}, response::{ResponseData, ResponseMessage}}};
use uuid::Uuid;

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
        }
    }

    async fn process_post(
        &self,
        client_id: Uuid,
        post_message_request_data: PostMessageRequestData,
    ) {
    }

    fn send_error(&self) {}

    fn send_message_to_client(&self, client_id: Uuid, response_data: ResponseData) {
      if self.response_sender.receiver_count() > 0 {
        self.response_sender.send(ResponseMessage::new(client_id, response_data)).unwrap();
      }
    }
}
