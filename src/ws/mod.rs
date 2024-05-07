pub mod pool;
pub mod socket;

use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct TaggedMessage<M, T>
where
    M: for<'a> Deserialize<'a> + Send + Sync,
{
    pub socket_id: socket::SocketId,
    pub message: M,
    pub tag: T,
}
