use std::io::Error as IoError;
use std::convert::TryInto;

use tracing::trace;

use streamfy_protocol::bytes::Buf;
use streamfy_protocol::{Encoder, Decoder};
use streamfy_protocol::api::{RequestMessage, ApiMessage, RequestHeader};

use super::fetch_consumer_offset_request::FetchConsumerOffsetRequest;
use super::update_consumer_offset_request::UpdateConsumerOffsetRequest;
use super::fetch_stream_request::FetchStreamRequest;

#[repr(u16)]
#[derive(Eq, PartialEq, Debug, Encoder, Decoder, Clone, Copy)]
#[streamfy(encode_discriminant)]
#[derive(Default)]
pub enum SPUPeerApiEnum {
    #[default]
    FetchStream = 0,
    FetchConsumerOffset = 1,
    UpdateConsumerOffset = 2,
}

#[derive(Debug, Encoder)]
pub enum SpuPeerRequest {
    #[streamfy(tag = 0)]
    FetchStream(RequestMessage<FetchStreamRequest>),
    #[streamfy(tag = 1)]
    FetchConsumerOffset(RequestMessage<FetchConsumerOffsetRequest>),
    #[streamfy(tag = 2)]
    UpdateConsumerOffset(RequestMessage<UpdateConsumerOffsetRequest>),
}

impl Default for SpuPeerRequest {
    fn default() -> SpuPeerRequest {
        SpuPeerRequest::FetchStream(RequestMessage::<FetchStreamRequest>::default())
    }
}

impl ApiMessage for SpuPeerRequest {
    type ApiKey = SPUPeerApiEnum;

    fn decode_with_header<T>(src: &mut T, header: RequestHeader) -> Result<Self, IoError>
    where
        Self: Default + Sized,
        Self::ApiKey: Sized,
        T: Buf,
    {
        trace!("decoding with header: {:#?}", header);
        let version = header.api_version();
        match header.api_key().try_into()? {
            SPUPeerApiEnum::FetchStream => Ok(SpuPeerRequest::FetchStream(RequestMessage::new(
                header,
                FetchStreamRequest::decode_from(src, version)?,
            ))),
            SPUPeerApiEnum::FetchConsumerOffset => {
                Ok(SpuPeerRequest::FetchConsumerOffset(RequestMessage::new(
                    header,
                    FetchConsumerOffsetRequest::decode_from(src, version)?,
                )))
            }
            SPUPeerApiEnum::UpdateConsumerOffset => {
                Ok(SpuPeerRequest::UpdateConsumerOffset(RequestMessage::new(
                    header,
                    UpdateConsumerOffsetRequest::decode_from(src, version)?,
                )))
            }
        }
    }
}
