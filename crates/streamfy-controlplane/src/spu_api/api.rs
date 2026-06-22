use std::io::Error as IoError;
use std::convert::TryInto;

use streamfy_protocol::api::api_decode;
use streamfy_protocol::api::ApiMessage;
use streamfy_protocol::api::RequestHeader;
use streamfy_protocol::api::RequestMessage;
use streamfy_protocol::bytes::Buf;
use streamfy_protocol::Encoder;
use streamfy_protocol::Decoder;

use super::update_mirror::UpdateMirrorRequest;
use super::update_spu::UpdateSpuRequest;
use super::update_replica::UpdateReplicaRequest;
use super::update_smartmodule::UpdateSmartModuleRequest;

#[repr(u16)]
#[derive(Eq, PartialEq, Debug, Encoder, Decoder, Clone, Copy)]
#[streamfy(encode_discriminant)]
#[derive(Default)]
pub enum InternalSpuApi {
    #[default]
    UpdateSpu = 1001,
    UpdateReplica = 1002,
    UpdateSmartModule = 1003,
    // UpdateDerivedStream = 1004,
    UpdateMirror = 1004,
}

#[derive(Debug, Encoder)]
pub enum InternalSpuRequest {
    #[streamfy(tag = 0)]
    UpdateSpuRequest(RequestMessage<UpdateSpuRequest>),
    #[streamfy(tag = 1)]
    UpdateReplicaRequest(RequestMessage<UpdateReplicaRequest>),
    #[streamfy(tag = 2)]
    UpdateSmartModuleRequest(RequestMessage<UpdateSmartModuleRequest>),
    #[streamfy(tag = 3)]
    UpdateMirrorRequest(RequestMessage<UpdateMirrorRequest>),
}

// Added to satisfy Encoder/Decoder traits
impl Default for InternalSpuRequest {
    fn default() -> Self {
        Self::UpdateSpuRequest(RequestMessage::default())
    }
}

impl InternalSpuRequest {
    pub fn new_update_spu_req(msg: UpdateSpuRequest) -> Self {
        Self::UpdateSpuRequest(RequestMessage::new_request(msg))
    }
}

impl ApiMessage for InternalSpuRequest {
    type ApiKey = InternalSpuApi;

    fn decode_with_header<T>(src: &mut T, header: RequestHeader) -> Result<Self, IoError>
    where
        Self: Default + Sized,
        Self::ApiKey: Sized,
        T: Buf,
    {
        match header.api_key().try_into()? {
            InternalSpuApi::UpdateSpu => api_decode!(Self, UpdateSpuRequest, src, header),
            InternalSpuApi::UpdateReplica => api_decode!(Self, UpdateReplicaRequest, src, header),
            InternalSpuApi::UpdateSmartModule => {
                api_decode!(Self, UpdateSmartModuleRequest, src, header)
            }
            InternalSpuApi::UpdateMirror => {
                api_decode!(Self, UpdateMirrorRequest, src, header)
            }
        }
    }
}
