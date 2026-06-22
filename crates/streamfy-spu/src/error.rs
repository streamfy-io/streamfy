use async_channel::SendError;
use streamfy_types::PartitionError;
use streamfy_storage::StorageError;
use streamfy_socket::SocketError;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum InternalServerError {
    #[error("Storage error")]
    Storage(#[from] StorageError),
    #[error("Partition error")]
    Partition(#[from] PartitionError),
    #[error("Socket error")]
    Socket(#[from] SocketError),
    #[error("Channel send error")]
    Send(String),
}

impl<T> From<SendError<T>> for InternalServerError {
    fn from(error: SendError<T>) -> Self {
        InternalServerError::Send(error.to_string())
    }
}
