mod unique;

use std::{error::Error, sync::Arc};
use tokio::sync::RwLock;
use unique::Unique;

pub trait Subscriber {
    async fn notify(&self, message: String) -> Result<(), Box<dyn Error>>;
}

pub type Store<S> = Arc<StoreInner<S>>;
pub struct StoreInner<S>
where
    S: Subscriber + std::hash::Hash + Eq + Clone,
{
    data: RwLock<String>,
    subscribers: RwLock<Unique<S>>,
}

impl<S> StoreInner<S>
where
    S: Subscriber + std::hash::Hash + Eq + Clone,
{
    pub fn new(inital: String) -> Store<S> {
        Arc::new(Self {
            data: RwLock::new(inital),
            subscribers: RwLock::new(Unique::new()),
        })
    }

    pub async fn subscribe(&self, s: S) {
        self.subscribers.write().await.insert(s);
    }

    pub async fn unsubscribe(&self, s: &S) {
        self.subscribers.write().await.remove(s);
    }

    pub async fn unsubscribe_many(&self, ids: &[S]) {
        if ids.is_empty() {
            return;
        }

        let mut subs = self.subscribers.write().await;
        for s in ids {
            subs.remove(s);
        }
    }

    pub async fn broadcast(&self) {
        let data = self.data.read().await.clone();
        let mut to_remove = Vec::new();
        {
            for s in self.subscribers.read().await.iter() {
                if let Err(e) = s.notify(data.clone()).await {
                    eprintln!("Error notifying subscriber: {}", e);
                    to_remove.push(s.clone());
                }
            }
        }

        self.unsubscribe_many(&to_remove).await;
    }
}
