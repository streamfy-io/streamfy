use streamfy_protocol::{Decoder, Encoder};

#[derive(Clone, Default, Encoder, Decoder)]
#[streamfy(encode_discriminant)]
pub enum SmartModuleInvocationWasm {
    #[default]
    Predefined = 0,
    AdHoc = 1,
}

fn main() {}
