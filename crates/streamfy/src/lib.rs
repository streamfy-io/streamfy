#![cfg_attr(
    feature = "nightly",
    doc = include_str!("../../../DEVELOPER.md")
)]
#![doc = include_str!("../README.md")]

mod admin;
mod error;
mod streamfy;
mod offset;
mod producer;
mod sync;

pub mod config;
pub mod consumer;
pub mod metrics;
pub mod spu;

pub use error::StreamfyError;
pub use config::{StreamfyClusterConfig, StreamfyConfig};
pub use producer::{
    ProducerCallback, SharedProducerCallback, ProduceCompletionBatchEvent,
    TopicProducerConfigBuilder, TopicProducerConfig, TopicProducer, TopicProducerPool, RecordKey,
    ProduceOutput, FutureRecordMetadata, RecordMetadata, DeliverySemantic, RetryPolicy,
    RetryStrategy, Partitioner, PartitionerConfig, ProducerError,
};
#[cfg(feature = "smartengine")]
pub use producer::{SmartModuleChainBuilder, SmartModuleConfig, SmartModuleInitialData};

pub use streamfy_spu_schema::Isolation;

pub use consumer::{
    PartitionConsumer, ConsumerConfig, MultiplePartitionConsumer, PartitionSelectionStrategy,
    SmartModuleInvocation, SmartModuleInvocationWasm, SmartModuleKind, SmartModuleContextData,
    SmartModuleExtraParams,
};
pub use offset::Offset;

pub use crate::admin::StreamfyAdmin;
pub use crate::streamfy::Streamfy;

pub use streamfy_compression::Compression;

pub use streamfy_types::PartitionId;
use tracing::instrument;

/// The minimum VERSION of the Streamfy Platform that this client is compatible with.
const MINIMUM_PLATFORM_VERSION: &str = "0.9.0";
pub(crate) const VERSION: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/VERSION"));

/// Creates a producer that sends records to the named topic
///
/// This is a shortcut function that uses the current profile
/// settings. If you need to specify any custom configurations,
/// try directly creating a [`Streamfy`] client object instead.
///
/// # Example: Simple records
///
/// Streamfy can send "simple" records that contain arbitrary
/// binary data. An easy way to demonstrate this is by sending
/// a string:
///
/// ```no_run
/// # use streamfy::{StreamfyError, RecordKey};
/// # async fn do_produce() -> anyhow::Result<()> {
/// let producer = streamfy::producer("my-topic").await?;
/// producer.send(RecordKey::NULL, "Hello, world!").await?;
/// # Ok(())
/// # }
/// ```
///
/// # Example: Key-value records
///
/// Streamfy also supports "key-value" records, where the key and
/// the value given may each be any binary data. Typically, the
/// key represents some unique property of the value, such as
/// an email address, username, or request ID.
///
/// ```no_run
/// # use streamfy::StreamfyError;
/// # async fn do_produce() -> anyhow::Result<()> {
/// let producer = streamfy::producer("my-topic").await?;
/// let key = "streamfy";
/// let value = r#"
/// {"project":"streamfy","about":"Data streaming in Rust!"}
/// "#;
/// producer.send(key, value).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Example: Flushing
///
/// Streamfy batches records by default, so it's important to flush the producer before terminating.
///
/// ```no_run
///     # use streamfy::StreamfyError;
///     # use streamfy_protocol::record::RecordKey;
///     # async fn produce_records() -> anyhow::Result<()> {
///     let producer = streamfy::producer("echo").await?;
///     for i in 0..10u8 {
///         producer.send(RecordKey::NULL, format!("Hello, Streamfy {}!", i)).await?;
///     }
///     producer.flush().await?;
///     # Ok(())
///     # }
/// ```
///
///
/// [`Streamfy`]: ./struct.Streamfy.html
#[instrument(skip(topic))]
pub async fn producer(
    topic: impl Into<String>,
) -> anyhow::Result<TopicProducer<spu::SpuSocketPool>> {
    let streamfy = Streamfy::connect().await?;
    let producer = streamfy.topic_producer(topic).await?;
    Ok(producer)
}

/// Creates a consumer that receives events from the given topic and partition
///
/// This is a shortcut function that uses the current profile
/// settings. If you need to specify any custom configurations,
/// try directly creating a [`Streamfy`] client object instead.
///
/// # Example
///
/// ```no_run
/// # use streamfy::{ConsumerConfig, StreamfyError, Offset};
/// # mod futures {
/// #     pub use futures_util::stream::StreamExt;
/// # }
/// #  async fn example() -> anyhow::Result<()> {
/// use futures::StreamExt;
/// let consumer = streamfy::consumer("my-topic", 0).await?;
/// let mut stream = consumer.stream(Offset::beginning()).await?;
/// while let Some(Ok(record)) = stream.next().await {
///     let key_str = record.get_key().map(|key| key.as_utf8_lossy_string());
///     let value_str = record.get_value().as_utf8_lossy_string();
///     println!("Got record: key={:?}, value={}", key_str, value_str);
/// }
/// # Ok(())
/// # }
/// ```
///
/// [`Streamfy`]: ./struct.Streamfy.html
#[deprecated(
    since = "0.21.8",
    note = "use `Streamfy::consumer_with_config()` instead"
)]
#[instrument(skip(topic, partition))]
#[allow(deprecated)]
pub async fn consumer(
    topic: impl Into<String>,
    partition: PartitionId,
) -> anyhow::Result<PartitionConsumer> {
    let streamfy = Streamfy::connect().await?;
    let consumer = streamfy.partition_consumer(topic, partition).await?;
    Ok(consumer)
}

/// re-export metadata from sc-api
pub mod metadata {

    pub use streamfy_sc_schema::AdminSpec;

    pub mod topic {
        pub use streamfy_sc_schema::topic::*;
    }

    pub mod smartmodule {
        pub use streamfy_sc_schema::smartmodule::*;
    }

    pub mod customspu {
        pub use streamfy_sc_schema::customspu::*;
    }

    pub mod spu {
        pub use streamfy_sc_schema::spu::*;
    }

    pub mod spg {
        pub use streamfy_sc_schema::spg::*;
    }

    pub mod partition {
        pub use streamfy_sc_schema::partition::*;
    }

    pub mod objects {
        pub use streamfy_sc_schema::objects::*;
    }

    pub mod tableformat {
        pub use streamfy_sc_schema::tableformat::*;
    }

    pub mod core {
        pub use streamfy_sc_schema::core::*;
    }

    pub mod store {
        pub use streamfy_sc_schema::store::*;
    }
}

pub mod dataplane {
    pub use streamfy_protocol::*;
}
