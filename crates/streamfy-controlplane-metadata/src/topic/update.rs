use streamfy_protocol::{Decoder, Encoder};

#[derive(Debug, Default, Encoder, Decoder, Clone)]
pub struct AddPartition {
    pub count: u32,
}

#[derive(Debug, Default, Encoder, Decoder, Clone)]
pub struct AddMirror {
    pub remote_cluster: String,
    // if set, this is mirror home
    pub home_to_mirror: bool,
}

#[derive(Debug, Encoder, Decoder, Clone)]
pub enum UpdateTopicAction {
    #[streamfy(tag = 0)]
    AddPartition(AddPartition),
    #[streamfy(tag = 1)]
    AddMirror(AddMirror),
}

impl Default for UpdateTopicAction {
    fn default() -> Self {
        Self::AddPartition(AddPartition::default())
    }
}
