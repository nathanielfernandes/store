use serde::Deserialize;

pub struct WebSocketPoolInner<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync, {}
