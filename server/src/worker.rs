use crate::{
    model::{feed::Feed, message::Message, user::User},
    protocol::{
        request::{JoinRequestData, PostMessageRequestData, RequestData, RequestMessage},
        response::{
            ErrorType, JoinedResponse, MessageResponse, PostedResponse, ResponseData,
            ResponseMessage, UserJoinedResponse, UserLeftResponse, UserResponse,
        },
    },
};
use chrono::Utc;
use futures::StreamExt;
use regex::Regex;
use std::{collections::HashMap, time::Duration};
use tokio::sync::{RwLock, broadcast, mpsc::UnboundedReceiver};
use tokio::time;
use tokio_stream::wrappers::UnboundedReceiverStream;
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

    pub async fn run(&self, receiver: UnboundedReceiver<RequestMessage>) {
        let ticking_alive = self.tick_alive();
        let stream = UnboundedReceiverStream::new(receiver);
        let processing = stream.for_each(|input_parcel| self.process(input_parcel));
        tokio::select! {
          _ = ticking_alive => (),
          _ = processing => ()
        };
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ResponseMessage> {
        self.response_sender.subscribe()
    }

    pub async fn on_disconnect(&self, client_id: Uuid) {
        if self.users.write().await.remove(&client_id).is_some() {
            self.send_message_to_other_clients(
                client_id,
                ResponseData::UserLeft(UserLeftResponse::new(client_id)),
            )
            .await;
        }
    }

    async fn tick_alive(&self) {
        match self.alive_interval {
            Some(interval) => loop {
                time::sleep(interval).await;
                self.send(ResponseData::Alive).await;
            },
            None => return,
        }
    }

    async fn process(&self, request_message: RequestMessage) {
        match request_message.request_data {
            RequestData::Join(request) => {
                self.process_join(request_message.client_id, request).await
            }
            RequestData::PostMessage(request) => {
                self.process_post(request_message.client_id, request).await
            }
        }
    }

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

        let user = User::new(client_id, user_name);
        self.users.write().await.insert(client_id, user);

        let user_response = UserResponse::new(client_id, user_name);
        let other_users = self
            .users
            .read()
            .await
            .values()
            .filter_map(|user| {
                if user.id != client_id {
                    Some(UserResponse::new(user.id, &user.name))
                } else {
                    None
                }
            })
            .collect();

        let messages = self
            .feed
            .read()
            .await
            .iter()
            .map(|message| {
                MessageResponse::new(
                    message.id,
                    UserResponse::new(message.user.id, &message.user.name),
                    &message.text,
                    message.created_at_utc,
                )
            })
            .collect();

        self.send_message_to_client(
            client_id,
            ResponseData::Joined(JoinedResponse::new(
                user_response.clone(),
                other_users,
                messages,
            )),
        );

        self.send_message_to_other_clients(
            client_id,
            ResponseData::UserJoined(UserJoinedResponse::new(user_response)),
        )
        .await;
    }

    async fn process_post(
        &self,
        client_id: Uuid,
        post_message_request_data: PostMessageRequestData,
    ) {
        let user = if let Some(user) = self.users.read().await.get(&client_id) {
            user.clone()
        } else {
            self.send_error(client_id, ErrorType::NotJoined);
            return;
        };

        if post_message_request_data.text.is_empty() {
            self.send_error(client_id, ErrorType::InvalidMessage);
            return;
        }

        let message = Message::new(
            Uuid::new_v4(),
            user.clone(),
            &post_message_request_data.text,
            Utc::now(),
        );
        self.feed.write().await.add_message(message.clone());

        let message_reponse = MessageResponse::new(
            message.id,
            UserResponse::new(user.id, &user.name),
            &message.text,
            message.created_at_utc,
        );

        self.send_message_to_client(
            client_id,
            ResponseData::Posted(PostedResponse::new(message_reponse.clone())),
        );

        self.send_message_to_other_clients(
            client_id,
            ResponseData::UserPosted(PostedResponse::new(message_reponse)),
        )
        .await;
    }

    async fn send(&self, response_data: ResponseData) {
        if self.response_sender.receiver_count() > 0 {
            self.users.read().await.keys().for_each(|user_id| {
                self.response_sender
                    .send(ResponseMessage::new(*user_id, response_data.clone()))
                    .unwrap();
            })
        }
    }

    fn send_message_to_client(&self, client_id: Uuid, response_data: ResponseData) {
        if self.response_sender.receiver_count() > 0 {
            self.response_sender
                .send(ResponseMessage::new(client_id, response_data))
                .unwrap();
        }
    }

    async fn send_message_to_other_clients(&self, client_id: Uuid, response_data: ResponseData) {
        if self.response_sender.receiver_count() > 0 {
            self.users
                .read()
                .await
                .values()
                .filter(|user| user.id != client_id)
                .for_each(|user| {
                    self.response_sender
                        .send(ResponseMessage::new(user.id, response_data.clone()))
                        .unwrap();
                })
        }
    }

    fn send_error(&self, client_id: Uuid, error_type: ErrorType) {
        self.send_message_to_client(client_id, ResponseData::Error(error_type))
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use tokio::{runtime::Runtime, sync::mpsc};
    use uuid::Uuid;

    use crate::protocol::{request::{JoinRequestData, PostMessageRequestData, RequestData, RequestMessage}, response::ResponseData};

    use super::Worker;

    #[test]
    fn join_and_post() {
        let worker = Worker::new(Some(Duration::from_secs(1)));
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut subscription = worker.subscribe();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let case = async {
                let client_id = Uuid::new_v4();

                // Join
                sender
                    .send(RequestMessage::new(
                        client_id,
                        RequestData::Join(JoinRequestData {
                            name: String::from("daolavi"),
                        }),
                    ))
                    .unwrap();
                let output = subscription.recv().await.unwrap().response_data;
                println!("{:?}", output);
                let user;
                if let ResponseData::Joined(joined) = output {
                    assert_eq!(joined.user.name.as_str(), "daolavi");
                    user = joined.user;
                } else {
                    panic!("Expected Output::Joined got {:?}", output);
                }

                // Post message
                sender
                    .send(RequestMessage::new(
                        client_id,
                        RequestData::PostMessage(PostMessageRequestData {
                            text: String::from("Hello"),
                        }),
                    ))
                    .unwrap();
                let output = subscription.recv().await.unwrap().response_data;
                if let ResponseData::Posted(posted) = output {
                    assert_eq!(posted.message.text, "Hello");
                    assert_eq!(posted.message.user.id, user.id);
                    assert_eq!(posted.message.user.name, user.name);
                } else {
                    panic!("Expected Output::Posted got {:?}", output);
                }

                return;
            };
            tokio::select! {
              _ = worker.run(receiver) => {},
              _ = case => {},
            }
        });
    }
}
