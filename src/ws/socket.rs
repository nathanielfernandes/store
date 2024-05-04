use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use tokio::sync::{Mutex, RwLock};

pub type SocketId = usize;
static NEXTID: AtomicUsize = AtomicUsize::new(0);
fn next_id() -> SocketId {
    NEXTID.fetch_add(1, Ordering::Relaxed)
}

pub type Socket<T> = Arc<SocketInner<T>>;
pub struct SocketInner<T> {
    id: SocketId,
    sink: Mutex<SplitSink<WebSocket, Message>>,
    tag: RwLock<Option<Arc<T>>>,
}

impl<T: Clone> SocketInner<T> {
    pub fn new(sink: SplitSink<WebSocket, Message>) -> Socket<T> {
        Arc::new(Self {
            id: next_id(),
            sink: Mutex::new(sink),
            tag: RwLock::new(None),
        })
    }

    pub async fn set_tag(self: &Arc<Self>, tag: T) {
        *self.tag.write().await = Some(Arc::new(tag));
    }

    pub async fn tag(self: &Arc<Self>) -> Option<Arc<T>> {
        self.tag.read().await.clone()
    }

    pub fn id(self: &Arc<Self>) -> SocketId {
        self.id
    }

    pub async fn send(self: &Arc<Self>, message: Message) -> Result<(), axum::Error> {
        self.sink.lock().await.send(message).await
    }
}

impl<T> std::hash::Hash for SocketInner<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> PartialEq for SocketInner<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for SocketInner<T> {}
