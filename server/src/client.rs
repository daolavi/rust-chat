use std::{error, result};

use futures::{Stream, StreamExt, TryStream, TryStreamExt, future, stream::SplitStream};
use uuid::Uuid;
use warp::ws::WebSocket;

use crate::{error::{Error, Result}, protocol::{request::RequestMessage, response::ResponseMessage}};

pub struct Client {
    pub id: Uuid,
}

impl Client {
    pub fn new() -> Self {
        Client { id: Uuid::new_v4() }
    }

    pub fn read(
        &self,
        stream: SplitStream<WebSocket>,
    ) -> impl Stream<Item = Result<RequestMessage>> {
        let client_id = self.id;
        stream
            // Take only text messages
            .take_while(|message| {
                future::ready(match message {
                    Ok(message) => message.is_text(),
                    Err(_) => false,
                })
            })
            // Deserialize JSON messages into proto::Input
            .map(move |message| match message {
                Err(err) => Err(Error::System(err.to_string())),
                Ok(message) => {
                    let input = serde_json::from_str(message.to_str().unwrap())?;
                    Ok(RequestMessage::new(client_id, input))
                }
            })
    }

    pub fn write<S, E>(&self, stream: S) -> impl Stream<Item = Result<warp::ws::Message>>
    where
        S: TryStream<Ok = ResponseMessage, Error = E> + Stream<Item = result::Result<ResponseMessage, E>>,
        E: error::Error,
    {
      let client_id = self.id;
      stream
          // Skip irrelevant parcels
          .try_filter(move |output_parcel| future::ready(output_parcel.client_id == client_id))
          // Serialize to JSON
          .map_ok(|output_parcel| {
              let data = serde_json::to_string(&output_parcel.response_data).unwrap();
              warp::ws::Message::text(data)
          })
          .map_err(|err| Error::System(err.to_string()))
    }
}
