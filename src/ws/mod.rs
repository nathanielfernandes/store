pub mod pool;
pub mod socket;

use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct TaggedMessage<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
{
    pub id: socket::SocketId,
    pub message: T,
}
