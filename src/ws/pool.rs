use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    SinkExt, StreamExt,
};
use moka::future::Cache;
use serde::{Deserialize, Serialize};

use super::{
    socket::{Socket, SocketId, SocketInner},
    TaggedMessage,
};

pub type WebSocketPool<M, Tag> = Arc<WebSocketPoolInner<M, Tag>>;
pub struct WebSocketPoolInner<M, Tag>
where
    M: for<'a> Deserialize<'a> + Send + Sync,
{
    sockets: Cache<SocketId, Socket>,
    subscriber: UnboundedSender<TaggedMessage<M, Tag>>,
}

impl<M, Tag> WebSocketPoolInner<M, Tag>
where
    M: for<'a> Deserialize<'a> + Send + Sync + 'static,
    Tag: Clone + Send + Sync + 'static,
{
    pub fn new() -> (
        WebSocketPool<M, Tag>,
        UnboundedReceiver<TaggedMessage<M, Tag>>,
    ) {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        (
            Arc::new(Self {
                sockets: Cache::builder().build(),
                subscriber: tx,
            }),
            rx,
        )
    }

    pub async fn add_socket(self: &Arc<Self>, socket: Socket) {
        self.sockets.insert(socket.id, socket).await;
    }

    pub async fn remove_socket(self: &Arc<Self>, id: SocketId) {
        if let Some(socket) = self.sockets.remove(&id).await {
            socket.terminate().await;
        }
    }

    pub async fn broadcast<T>(self: &Arc<Self>, message: T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let message = serde_json::to_string(&message)?;
        for (id, socket) in self.sockets.iter() {
            if let Err(_) = socket.send(Message::Text(message.clone())).await {
                self.remove_socket(*id).await;
            }
        }

        Ok(())
    }

    pub async fn send_to<T>(self: &Arc<Self>, id: &mut SocketId, message: T) -> Result<(), Error>
    where
        T: Serialize,
    {
        if let Some(socket) = self.sockets.get(&id).await {
            let message = serde_json::to_string(&message)?;
            match socket.send(Message::Text(message)).await {
                Err(_) => self.remove_socket(*id).await,
                _ => *id = 0,
            }
        }

        Ok(())
    }

    pub async fn send_to_many<T>(
        self: &Arc<Self>,
        ids: &mut [SocketId],
        message: T,
    ) -> Result<(), Error>
    where
        T: Serialize,
    {
        if ids.is_empty() {
            return Ok(());
        }

        let message = serde_json::to_string(&message)?;
        for id in ids {
            if let Some(socket) = self.sockets.get(id).await {
                match socket.send(Message::Text(message.clone())).await {
                    Err(_) => self.remove_socket(*id).await,
                    _ => *id = 0,
                }
            }
        }

        Ok(())
    }

    pub async fn listen_to(self: &Arc<Self>, websocket: WebSocket, tag: Tag) -> SocketId {
        let (sink, mut stream) = websocket.split();
        let socket = SocketInner::new(sink);
        let id = socket.id;

        self.add_socket(socket.clone()).await;

        let this = self.clone();
        tokio::task::spawn(async move {
            let mut subscriber = this.subscriber.clone();
            println!("Listening to socket {}", socket.id);
            while let Some(message) = stream.next().await {
                let result: Result<(), Error> = try {
                    match message? {
                        Message::Text(text) => {
                            let message: M = match serde_json::from_str(&text) {
                                Ok(m) => m,
                                Err(e) => {
                                    eprintln!("Error deserializing message: {}", e);
                                    continue;
                                }
                            };

                            subscriber
                                .send(TaggedMessage {
                                    tag: tag.clone(),
                                    socket_id: socket.id,
                                    message,
                                })
                                .await?;
                        }
                        Message::Close(_) => break,
                        _ => (),
                    }
                };

                if let Err(e) = result {
                    eprintln!("Error sending message: {}", e);
                    break;
                }
            }

            this.remove_socket(socket.id).await;
            println!("Socket {} terminated", socket.id);
        });

        id
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;
