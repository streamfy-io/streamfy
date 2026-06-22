use streamfy_protocol::{Decoder, Encoder};

#[derive(Clone, Default, Encoder, Decoder)]
pub enum SmartModuleInvocationWasm {
    #[default]
    #[streamfy(min_version = 1)]
    Predefined,
    #[streamfy(min_version = 2)]
    AdHoc,
}

fn main() {}
