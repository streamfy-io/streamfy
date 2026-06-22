use streamfy_protocol::{Encoder, Decoder};

#[repr(u16)]
#[derive(Eq, PartialEq, Debug, Encoder, Decoder, Clone, Copy, Default)]
#[streamfy(encode_discriminant)]
pub enum MirrorRemoteApiEnum {
    #[default]
    SyncRecords = 0,
    UpdateEdgeOffset = 1,
}
