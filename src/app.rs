use axum::extract::ws::WebSocket;
use moka::future::Cache;

use crate::namespace::{Namespace, NamespaceInner};

#[repr(transparent)]
#[derive(Clone)]
pub struct App {
    namespaces: Cache<String, Namespace>,
}

impl App {
    pub fn new() -> Self {
        Self {
            namespaces: Cache::builder().build(),
        }
    }

    pub async fn new_namespace(&self, name: String, write_key: String) -> Namespace {
        let namespace: std::sync::Arc<NamespaceInner> = NamespaceInner::new(write_key).await;
        self.namespaces.insert(name, namespace.clone()).await;
        namespace
    }

    // ideally namespaces should be created seperate from connections
    pub async fn add_connection(
        self,
        namespace: String,
        write_key: Option<String>,
        websocket: WebSocket,
    ) {
        let Some(ns) = self.namespaces.get(&namespace).await else {
            println!("Namespace not found: {}", namespace);
            return;
        };
        ns.add_connection(websocket, write_key.as_ref()).await;
    }

    pub async fn read_store(self, namespace: &String, store: &String) -> Option<String> {
        let Some(ns) = self.namespaces.get(namespace).await else {
            return None;
        };
        ns.read_store(store).await
    }

    pub async fn write_store(
        self,
        namespace: &String,
        write_key: &String,
        store: &String,
        value: String,
    ) -> Result<(), &'static str> {
        let Some(ns) = self.namespaces.get(namespace).await else {
            return Err("Namespace not found");
        };
        ns.write_store(store, write_key, value).await
    }
}
