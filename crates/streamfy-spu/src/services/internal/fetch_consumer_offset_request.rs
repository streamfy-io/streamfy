use std::fmt;

use streamfy_protocol::api::Request;
use streamfy_protocol::link::ErrorCode;
use streamfy_protocol::record::ReplicaKey;
use streamfy_protocol::{Encoder, Decoder};

use streamfy_spu_schema::COMMON_VERSION;
use streamfy_types::PartitionId;

use super::SPUPeerApiEnum;

#[derive(Decoder, Encoder, Default, Debug)]
pub struct FetchConsumerOffsetRequest {
    pub replica_id: ReplicaKey,
    pub consumer_id: String,
}

impl Request for FetchConsumerOffsetRequest {
    const API_KEY: u16 = SPUPeerApiEnum::FetchConsumerOffset as u16;
    const DEFAULT_API_VERSION: i16 = COMMON_VERSION;
    type Response = FetchConsumerOffsetResponse;
}

impl FetchConsumerOffsetRequest {
    #[allow(dead_code)]
    pub fn new(
        topic: impl Into<String>,
        partition: PartitionId,
        consumer_id: impl Into<String>,
    ) -> Self {
        let replica_id = ReplicaKey::new(topic, partition);
        Self {
            consumer_id: consumer_id.into(),
            replica_id,
        }
    }
}

#[derive(Encoder, Decoder, Default, Debug)]
pub struct FetchConsumerOffsetResponse {
    pub error_code: ErrorCode,
    pub consumer: Option<Consumer>,
}

#[derive(Encoder, Decoder, Default, Debug)]
pub struct Consumer {
    pub offset: i64,
}

impl FetchConsumerOffsetResponse {
    pub fn new(error_code: ErrorCode, consumer: Option<Consumer>) -> Self {
        Self {
            error_code,
            consumer,
        }
    }
}

impl Consumer {
    pub fn new(offset: i64) -> Self {
        Self { offset }
    }
}

impl fmt::Display for FetchConsumerOffsetResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "error: {:#?}, consumer: {:?}",
            self.error_code, self.consumer
        )
    }
}
