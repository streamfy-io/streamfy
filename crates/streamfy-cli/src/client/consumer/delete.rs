use clap::Parser;
use anyhow::Result;

use streamfy::Streamfy;
use streamfy_types::PartitionId;

use crate::common::output::Terminal;
use crate::common::OutputFormat;

/// Option for Deleting Consumers
#[derive(Debug, Parser)]
pub struct DeleteConsumerOpt {
    #[clap(flatten)]
    output: OutputFormat,

    consumer: String,
    #[arg(short, long, required = false)]
    topic: Option<String>,
    #[arg(short, long, required = false, requires = "topic")]
    partition: Option<PartitionId>,
}

impl DeleteConsumerOpt {
    /// perform actions
    pub async fn process<O>(self, _out: std::sync::Arc<O>, streamfy: &Streamfy) -> Result<()>
    where
        O: Terminal,
    {
        if let Some((topic, partition)) = self.topic.as_ref().zip(self.partition.as_ref()) {
            delete(streamfy, self.consumer, topic.clone(), *partition).await?;
        } else {
            let consumers: Vec<_> = streamfy
                .consumer_offsets()
                .await?
                .into_iter()
                .filter(|c| c.consumer_id.eq(&self.consumer))
                .filter(|c| {
                    self.topic.is_none() || c.topic.eq(self.topic.as_deref().unwrap_or_default())
                })
                .collect();
            if consumers.is_empty() {
                println!("no consumers found");
            } else {
                for consumer in consumers {
                    delete(
                        streamfy,
                        consumer.consumer_id,
                        consumer.topic,
                        consumer.partition,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }
}

async fn delete(
    streamfy: &Streamfy,
    consumer: String,
    topic: String,
    partition: PartitionId,
) -> Result<()> {
    let message = format!(
        "consumer \"{consumer}\" on topic \"{topic}\" and partition \"{partition}\" deleted"
    );
    streamfy
        .delete_consumer_offset(consumer, (topic, partition))
        .await?;
    println!("{message}");

    Ok(())
}
