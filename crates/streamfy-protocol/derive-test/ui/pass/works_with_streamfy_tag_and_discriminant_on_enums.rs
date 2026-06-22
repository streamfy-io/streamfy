use streamfy_protocol::{Decoder, Encoder};

#[derive(Clone, Default, Encoder, Decoder)]
pub enum SmartModuleInvocationWasm {
    #[default]
    #[streamfy(tag = 0)]
    Predefined = 0,
    #[streamfy(tag = 1)]
    AdHoc = 1,
}

fn main() {}
