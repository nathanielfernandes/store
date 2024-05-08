pub mod messages;

use std::{sync::Arc, time::Duration};

use axum::extract::ws::WebSocket;
use futures::{channel::mpsc::UnboundedReceiver, StreamExt};

use moka::future::Cache;

use crate::{
    store::{Store, StoreInner},
    ws::{
        pool::{WebSocketPool, WebSocketPoolInner},
        socket::SocketId,
        TaggedMessage,
    },
};

use messages::{ClientMessage, ServerMessage};

pub type Namespace = Arc<NamespaceInner>;
pub struct NamespaceInner {
    write_key: String,

    pool: WebSocketPool<ClientMessage, bool>,
    stores: Cache<String, Store<SocketId>>,
}

impl NamespaceInner {
    pub async fn new(write_key: String) -> Namespace {
        let (pool, listener) = WebSocketPoolInner::new();

        let this = Arc::new(Self {
            write_key,

            pool,
            stores: Cache::builder()
                .time_to_idle(Duration::from_secs(3600 * 12))
                .build(),
        });

        this.start(listener).await;

        this
    }

    pub async fn read_store(self: &Arc<Self>, name: &String) -> Option<String> {
        let store = self.stores.get(name).await?;
        Some(store.get().await)
    }

    pub async fn write_store(
        self: &Arc<Self>,
        name: &String,
        write_key: &String,
        value: String,
    ) -> Result<(), &'static str> {
        if write_key != &self.write_key {
            return Err("Invalid write key");
        }

        let store = self.stores.get(name).await;
        if let Some(store) = store {
            store.set(value.clone()).await;

            let mut subscribers = store.subscibers().await;
            let message = ServerMessage::Update {
                store: name.clone(),
                value,
            };
            let _ = self.pool.send_to_many(&mut subscribers, message).await;

            return Ok(());
        }

        self.new_store(name.clone(), value).await;

        Ok(())
    }

    pub async fn new_store(self: &Arc<Self>, name: String, value: String) -> Store<SocketId> {
        let store = StoreInner::new(value);
        self.stores.insert(name, store.clone()).await;
        store
    }

    async fn start(
        self: &Arc<Self>,
        mut listener: UnboundedReceiver<TaggedMessage<ClientMessage, bool>>,
    ) {
        let this = self.clone();
        tokio::task::spawn(async move {
            while let Some(TaggedMessage {
                mut socket_id,
                message,
                tag: can_write,
            }) = listener.next().await
            {
                match message {
                    ClientMessage::Subscribe {
                        store: store_name,
                        initial,
                    } => {
                        let store = match this.stores.get(&store_name).await {
                            Some(store) => store,
                            None => {
                                if !can_write {
                                    continue;
                                }
                                this.new_store(store_name.clone(), initial.clone()).await
                            }
                        };

                        store.subscribe(socket_id).await;

                        let value = store.get().await;
                        let message = ServerMessage::Update {
                            store: store_name,
                            value,
                        };
                        let _ = this.pool.send_to(&mut socket_id, message).await;
                    }
                    ClientMessage::Unsubscribe { store } => {
                        if let Some(store) = this.stores.get(&store).await {
                            store.unsubscribe(&socket_id).await;
                        }
                    }
                    ClientMessage::Set {
                        store: store_name,
                        value,
                    } => {
                        if !can_write {
                            continue;
                        }

                        let store = match this.stores.get(&store_name).await {
                            Some(store) => store,
                            None => {
                                this.new_store(store_name.clone(), value.clone()).await;
                                continue;
                            }
                        };

                        store.set(value.clone()).await;

                        let mut subscribers = store.subscibers().await;
                        let message = ServerMessage::Update {
                            store: store_name,
                            value,
                        };
                        let _ = this.pool.send_to_many(&mut subscribers, message).await;

                        // if the message failed to send, remove the socket from the store
                        store.unsubscribe_many(&subscribers).await;
                    }

                    ClientMessage::Get { store: store_name } => {
                        if let Some(store) = this.stores.get(&store_name).await {
                            let value = store.get().await;

                            let message = ServerMessage::Update {
                                store: store_name,
                                value,
                            };

                            let _ = this.pool.send_to(&mut socket_id, message).await;

                            // if the message failed to send, remove the socket from the store
                            store.unsubscribe(&socket_id).await;
                        }
                    }
                }
            }
        });
    }

    pub async fn add_connection(
        self: &Arc<Self>,
        websocket: WebSocket,
        write_key: Option<&String>,
    ) {
        let can_write = write_key == Some(&self.write_key);
        self.pool.listen_to(websocket, can_write).await;
    }
}
