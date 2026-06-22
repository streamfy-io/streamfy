use streamfy_protocol::{Decoder, Encoder};

#[derive(Clone, Default, Encoder, Decoder)]
pub enum SmartModuleInvocationWasm {
    #[default]
    #[streamfy(tag = 0)]
    Predefined,
    #[streamfy(tag = 1)]
    AdHoc,
}

fn main() {}
