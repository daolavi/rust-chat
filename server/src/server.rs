use std::{sync::Arc, time::Duration};

use futures::{StreamExt, TryStreamExt};
use log::{error, info};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::{UnboundedReceiverStream};
use warp::{ws::WebSocket, Filter};

use crate::{client::Client, protocol::request::RequestMessage, worker::Worker};

const MAX_FRAME_SIZE: usize = 1 << 16;

pub struct Server {
    port: u16,
    worker: Arc<Worker>,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Server {
            port,
            worker: Arc::new(Worker::new(Some(Duration::from_secs(5)))),
        }
    }

    pub async fn run(&self) {
        println!("{:?}", MAX_FRAME_SIZE);
        let (sender, receiver) = mpsc::unbounded_channel::<RequestMessage>();
        let worker = self.worker.clone();

        let feed = warp::path("feed")
            .and(warp::ws())
            .and(warp::any().map(move || sender.clone()))
            .and(warp::any().map(move || worker.clone()))
            .map(
                |ws: warp::ws::Ws, sender: UnboundedSender<RequestMessage>, worker: Arc<Worker>| {
                    ws.max_frame_size(MAX_FRAME_SIZE)
                        .on_upgrade(move |web_socket| async move {
                            tokio::spawn(Self::process_client(worker, web_socket, sender));
                        })
                },
            );

        let shutdown = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
        };
        let (_, serving) =
            warp::serve(feed).bind_with_graceful_shutdown(([127, 0, 0, 1], self.port), shutdown);

        let running_hub = self.worker.run(receiver);

        tokio::select! {
            _ = serving => {},
            _ = running_hub => {},
        }
    }

    async fn process_client(
        hub: Arc<Worker>,
        web_socket: WebSocket,
        input_sender: UnboundedSender<RequestMessage>,
    ) {
        let output_receiver = hub.subscribe();
        let (ws_sink, ws_stream) = web_socket.split();
        let client = Client::new();

        info!("Client {} connected", client.id);

        let reading = client.read(ws_stream).try_for_each(|input_parcel| async {
            input_sender.send(input_parcel).unwrap();
            Ok(())
        });

        let (tx, rx) = mpsc::unbounded_channel();
        let stream = UnboundedReceiverStream::new(rx);
        let receiver_stream = tokio_stream::wrappers::BroadcastStream::new(output_receiver);
        tokio::spawn(stream.forward(ws_sink));
        let writing = client
            .write(receiver_stream.into_stream())
            .try_for_each(|message| async {
                tx.send(Ok(message)).unwrap();
                Ok(())
            });

        if let Err(err) = tokio::select! {
            result = reading => result,
            result = writing => result,
        } {
            error!("Client connection error: {}", err);
        }

        hub.on_disconnect(client.id).await;
        info!("Client {} disconnected", client.id);
    }
}
