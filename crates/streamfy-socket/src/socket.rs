use std::fmt;

use tracing::{debug, instrument};

use streamfy_protocol::api::Request;
use streamfy_protocol::api::RequestMessage;
use streamfy_protocol::api::ResponseMessage;

use streamfy_future::net::{
    BoxReadConnection, BoxWriteConnection, ConnectionFd, DefaultDomainConnector, TcpDomainConnector,
};

use super::SocketError;
use crate::StreamfySink;
use crate::StreamfyStream;

/// Socket abstract that can send and receive streamfy objects
pub struct StreamfySocket {
    sink: StreamfySink,
    stream: StreamfyStream,
    stale: bool,
}

impl fmt::Debug for StreamfySocket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Socket({})", self.id())
    }
}

impl StreamfySocket {
    pub fn new(sink: StreamfySink, stream: StreamfyStream) -> Self {
        Self {
            sink,
            stream,
            stale: false,
        }
    }

    pub fn split(self) -> (StreamfySink, StreamfyStream) {
        (self.sink, self.stream)
    }

    /// mark as stale
    pub fn set_stale(&mut self) {
        self.stale = true;
    }

    pub fn is_stale(&self) -> bool {
        self.stale
    }

    pub fn get_mut_sink(&mut self) -> &mut StreamfySink {
        &mut self.sink
    }

    pub fn get_mut_stream(&mut self) -> &mut StreamfyStream {
        &mut self.stream
    }

    pub fn id(&self) -> ConnectionFd {
        self.sink.id()
    }

    /// as client, send request and wait for reply from server
    pub async fn send<R>(
        &mut self,
        req_msg: &RequestMessage<R>,
    ) -> Result<ResponseMessage<R::Response>, SocketError>
    where
        R: Request,
    {
        self.sink.send_request(req_msg).await?;
        self.stream.next_response(req_msg).await
    }
}

impl StreamfySocket {
    #[allow(clippy::clone_on_copy)]
    pub fn from_stream(
        write: BoxWriteConnection,
        read: BoxReadConnection,
        fd: ConnectionFd,
    ) -> Self {
        Self::new(
            StreamfySink::new(write, fd.clone()),
            StreamfyStream::new(fd, read),
        )
    }

    /// connect to target address with connector
    #[instrument(skip(connector))]
    pub async fn connect_with_connector(
        addr: &str,
        connector: &dyn TcpDomainConnector,
    ) -> Result<Self, SocketError> {
        debug!("connecting to addr at: {}", addr);

        let (write, read, fd) = connector.connect(addr).await.map_err(|e| {
            let emsg = e.to_string();
            SocketError::Io {
                source: e,
                msg: format!("{emsg}, can't connect to {addr}"),
            }
        })?;

        Ok(Self::from_stream(write, read, fd))
    }
}

impl From<(StreamfySink, StreamfyStream)> for StreamfySocket {
    fn from(pair: (StreamfySink, StreamfyStream)) -> Self {
        let (sink, stream) = pair;
        Self::new(sink, stream)
    }
}

impl StreamfySocket {
    pub async fn connect(addr: &str) -> Result<Self, SocketError> {
        let connector = DefaultDomainConnector::new();
        Self::connect_with_connector(addr, &connector).await
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(unix, windows))] {
        use streamfy_future::net::{
            AsConnectionFd, TcpStream,
        };
        impl From<TcpStream> for StreamfySocket {
            fn from(tcp_stream: TcpStream) -> Self {
                let fd = tcp_stream.as_connection_fd();
                Self::from_stream(Box::new(tcp_stream.clone()),Box::new(tcp_stream), fd)
            }
        }
    }
}
