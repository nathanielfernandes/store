mod unique;

use std::sync::Arc;
use tokio::sync::RwLock;
use unique::Unique;

pub type Store<S> = Arc<StoreInner<S>>;
pub struct StoreInner<S>
where
    S: std::hash::Hash + Eq + Clone,
{
    data: RwLock<String>,
    subscribers: RwLock<Unique<S>>,
}

impl<S> StoreInner<S>
where
    S: std::hash::Hash + Eq + Clone,
{
    pub fn new(inital: String) -> Store<S> {
        Arc::new(Self {
            data: RwLock::new(inital),
            subscribers: RwLock::new(Unique::new()),
        })
    }

    pub async fn get(&self) -> String {
        self.data.read().await.clone()
    }

    pub async fn set(&self, value: String) {
        *self.data.write().await = value;
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

    pub async fn subscibers(&self) -> Vec<S> {
        self.subscribers.read().await.get_all()
    }
}
