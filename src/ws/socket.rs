use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use tokio::sync::{Mutex, RwLock};

// sockets start at 1
pub type SocketId = usize;
static NEXTID: AtomicUsize = AtomicUsize::new(1);
fn next_id() -> SocketId {
    NEXTID.fetch_add(1, Ordering::Relaxed)
}

pub type Socket = Arc<SocketInner>;
pub struct SocketInner {
    pub(crate) id: SocketId,
    sink: Mutex<SplitSink<WebSocket, Message>>,
    terminated: RwLock<bool>,
}

impl SocketInner {
    pub fn new(sink: SplitSink<WebSocket, Message>) -> Socket {
        Arc::new(Self {
            id: next_id(),
            sink: Mutex::new(sink),
            terminated: RwLock::new(false),
        })
    }

    pub fn id(self: &Arc<Self>) -> SocketId {
        self.id
    }

    pub async fn send(self: &Arc<Self>, message: Message) -> Result<(), &'static str> {
        if *self.terminated.read().await {
            return Err("Socket is terminated");
        }

        self.sink
            .lock()
            .await
            .send(message)
            .await
            .map_err(|_| "Failed to send message")
    }

    pub async fn terminate(self: &Arc<Self>) {
        *self.terminated.write().await = true;
    }
}

impl std::hash::Hash for SocketInner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for SocketInner {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SocketInner {}
