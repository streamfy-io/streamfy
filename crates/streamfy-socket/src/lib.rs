mod error;
mod multiplexing;
mod sink;
mod socket;
mod stream;
mod versioned;
mod stream_socket;

#[cfg(test)]
pub mod test_request;

pub use streamfy_future::net::{BoxConnection, Connection};
pub use self::error::SocketError;
pub use self::socket::StreamfySocket;
pub use multiplexing::*;
pub use sink::*;

pub use stream::*;
pub use stream_socket::*;
pub use versioned::*;

use streamfy_protocol::api::Request;
use streamfy_protocol::api::RequestMessage;
use streamfy_protocol::api::ResponseMessage;

pub(crate) mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
/// send request and return response from calling server at socket addr
pub async fn send_and_receive<R>(
    addr: &str,
    request: &RequestMessage<R>,
) -> Result<ResponseMessage<R::Response>, SocketError>
where
    R: Request,
{
    let mut client = StreamfySocket::connect(addr).await?;
    let msgs: ResponseMessage<R::Response> = client.send(request).await?;
    Ok(msgs)
}
