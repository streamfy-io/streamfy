use std::fmt::Debug;

use anyhow::Result;

use streamfy_controlplane_metadata::mirroring::{
    MirroringRemoteClusterRequest, MirroringRemoteClusterSpec, MirroringStatusResponse,
};
use streamfy_protocol::{Decoder, Encoder, Version};
use streamfy_protocol::api::Request;

use crate::{AdminPublicApiKey, TryEncodableFrom};
use crate::objects::{COMMON_VERSION, TypeBuffer};

#[derive(Encoder, Decoder, Default, Debug)]
pub struct ObjectMirroringRequest(TypeBuffer);

impl Request for ObjectMirroringRequest {
    const API_KEY: u16 = AdminPublicApiKey::Mirroring as u16;
    const MIN_API_VERSION: i16 = 14;
    const DEFAULT_API_VERSION: i16 = COMMON_VERSION;
    type Response = MirroringStatusResponse;
}

impl<S> TryEncodableFrom<MirroringRemoteClusterRequest<S>> for ObjectMirroringRequest
where
    MirroringRemoteClusterRequest<S>: Encoder + Decoder + Debug,
    S: MirroringRemoteClusterSpec + streamfy_controlplane_metadata::core::Spec,
{
    fn try_encode_from(input: MirroringRemoteClusterRequest<S>, version: Version) -> Result<Self> {
        Ok(Self(TypeBuffer::encode::<S, _>(input, version)?))
    }

    fn downcast(&self) -> Result<Option<MirroringRemoteClusterRequest<S>>> {
        self.0.downcast::<S, _>()
    }
}
