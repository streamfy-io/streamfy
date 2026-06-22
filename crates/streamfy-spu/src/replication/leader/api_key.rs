use streamfy_protocol::{Encoder, Decoder};

#[repr(u16)]
#[derive(Eq, PartialEq, Debug, Encoder, Decoder, Clone, Copy)]
#[streamfy(encode_discriminant)]
#[derive(Default)]
pub enum LeaderPeerApiEnum {
    #[default]
    UpdateOffsets = 0,
}
