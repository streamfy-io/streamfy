//! API call from Spu to SC

use std::io::Error as IoError;
use std::convert::TryInto;

use streamfy_protocol::api::api_decode;
use streamfy_protocol::api::ApiMessage;
use streamfy_protocol::api::RequestHeader;
use streamfy_protocol::api::RequestMessage;
use streamfy_protocol::bytes::Buf;
use streamfy_protocol::derive::Encoder;
use streamfy_protocol::derive::Decoder;

use crate::sc_api::update_mirror::UpdateMirrorStatRequest;
use crate::sc_api::update_partition::UpdatePartitionStatRequest;

use super::register_spu::RegisterSpuRequest;
use super::update_lrs::UpdateLrsRequest;
use super::remove::ReplicaRemovedRequest;

#[repr(u16)]
#[derive(Eq, PartialEq, Debug, Encoder, Decoder, Clone, Copy)]
#[streamfy(encode_discriminant)]
#[derive(Default)]
pub enum InternalScKey {
    #[default]
    RegisterSpu = 2000,
    UpdateLrs = 2001,
    ReplicaRemoved = 2002,
    UpdateMirror = 2003,
    UpdatePartition = 2004,
}

/// Request made to Spu from Sc
#[derive(Debug, Encoder)]
pub enum InternalScRequest {
    #[streamfy(tag = 0)]
    RegisterSpuRequest(RequestMessage<RegisterSpuRequest>),
    #[streamfy(tag = 1)]
    UpdateLrsRequest(RequestMessage<UpdateLrsRequest>),
    #[streamfy(tag = 2)]
    ReplicaRemovedRequest(RequestMessage<ReplicaRemovedRequest>),
    #[streamfy(tag = 3)]
    UpdateMirrorStatRequest(RequestMessage<UpdateMirrorStatRequest>),
    #[streamfy(tag = 4)]
    UpdatePartitionStatRequest(RequestMessage<UpdatePartitionStatRequest>),
}

impl Default for InternalScRequest {
    fn default() -> InternalScRequest {
        InternalScRequest::RegisterSpuRequest(RequestMessage::default())
    }
}

impl ApiMessage for InternalScRequest {
    type ApiKey = InternalScKey;

    fn decode_with_header<T>(src: &mut T, header: RequestHeader) -> Result<Self, IoError>
    where
        Self: Default + Sized,
        Self::ApiKey: Sized,
        T: Buf,
    {
        match header.api_key().try_into()? {
            InternalScKey::RegisterSpu => {
                api_decode!(InternalScRequest, RegisterSpuRequest, src, header)
            }
            InternalScKey::UpdateLrs => {
                api_decode!(InternalScRequest, UpdateLrsRequest, src, header)
            }
            InternalScKey::ReplicaRemoved => {
                api_decode!(InternalScRequest, ReplicaRemovedRequest, src, header)
            }
            InternalScKey::UpdateMirror => {
                api_decode!(InternalScRequest, UpdateMirrorStatRequest, src, header)
            }
            InternalScKey::UpdatePartition => {
                api_decode!(InternalScRequest, UpdatePartitionStatRequest, src, header)
            }
        }
    }
}
