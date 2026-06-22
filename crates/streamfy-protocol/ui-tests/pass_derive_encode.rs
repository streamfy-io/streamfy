use streamfy_protocol::Encoder;

fn main() {}

#[derive(Encoder)]
struct PassTupleStruct(u16, String);

#[derive(Encoder)]
struct PassNamedStruct {
    number: u16,
    string: String,
}

#[repr(u16)]
#[derive(Encoder)]
#[streamfy(encode_discriminant)]
enum PassUnitEnum {
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Encoder)]
enum PassTupleEnum {
    #[streamfy(tag = 0)]
    First(String),
    #[streamfy(tag = 1)]
    Second(u16),
    #[streamfy(tag = 2)]
    Third(Vec<u8>),
}

#[derive(Encoder)]
enum PassNamedEnum {
    #[streamfy(tag = 0)]
    Alpha { name: String, number: i32 },
    #[streamfy(tag = 1)]
    Beta { data: Vec<u8> },
}