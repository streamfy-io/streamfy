use streamfy_protocol::{Decoder, Encoder};

#[derive(Clone, Default, Encoder, Decoder)]
pub enum SmartModuleInvocationWasm {
    #[default]
    Predefined,
    AdHoc,
}

fn main() {}
