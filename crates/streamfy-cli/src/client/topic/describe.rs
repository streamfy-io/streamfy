//!
//! # Describe Topic CLI
//!
//! CLI to describe Topics and their corresponding Partitions with operational status:
//! leader, LEO (last offset), last produced time, and active consumers.
//!

use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use clap::Parser;
use futures_util::StreamExt;
use tracing::debug;

use streamfy::consumer::{ConsumerConfigExt, ConsumerOffset};
use streamfy::metadata::objects::Metadata;
use streamfy::metadata::partition::PartitionSpec;
use streamfy::metadata::topic::TopicSpec;
use streamfy::Offset;
use streamfy::Streamfy;
use streamfy_protocol::record::ReplicaKey;
use streamfy_sc_schema::objects::ListRequest;
use streamfy_types::PartitionId;

use crate::common::output::Terminal;
use crate::common::OutputFormat;

// -----------------------------------
// CLI Options
// -----------------------------------

#[derive(Debug, Parser)]
pub struct DescribeTopicsOpt {
    /// The name of the Topic to describe
    #[arg(value_name = "name")]
    topic: String,

    #[clap(flatten)]
    output: OutputFormat,
}

impl DescribeTopicsOpt {
    pub async fn process<O: Terminal>(self, out: Arc<O>, streamfy: &Streamfy) -> Result<()> {
        let topic = self.topic;
        let output_type = self.output.format;
        debug!("describe topic: {}, {:?}", topic, output_type);

        let admin = streamfy.admin().await;
        let topics = admin.list::<TopicSpec, _>(vec![topic.clone()]).await?;

        let partitions = admin
            .list_with_config::<PartitionSpec, String>(ListRequest::default())
            .await?;
        let topic_partitions = filter_partitions_by_topic(&topic, &partitions);

        let consumers = streamfy.consumer_offsets().await.unwrap_or_default();
        let consumers_by_partition = group_consumers_by_partition(&topic, &consumers);

        let mut partition_rows = Vec::with_capacity(topic_partitions.len());
        for partition_meta in &topic_partitions {
            let partition_id = partition_id_from_name(&partition_meta.name);
            let leader = partition_meta.spec.leader;
            // LAST-OFFSET is LEO (Last End Offset) per operational guidance.
            let leo = partition_meta.status.leader.leo;

            let last_produced =
                fetch_last_produced(streamfy, &topic, partition_id, leo).await;

            let active_consumers = consumers_by_partition
                .get(&partition_id)
                .cloned()
                .unwrap_or_default();

            partition_rows.push(PartitionDescribeRow {
                partition: partition_id,
                leader,
                last_offset: leo,
                last_produced,
                consumers: active_consumers,
            });
        }

        partition_rows.sort_by_key(|row| row.partition);

        display::describe_topics(topics, partition_rows, output_type, out).await?;
        Ok(())
    }
}

/// Filter partition metadata belonging to `topic`.
/// Partition names are typically `{topic}-{partition_id}`.
pub(crate) fn filter_partitions_by_topic(
    topic: &str,
    partitions: &[Metadata<PartitionSpec>],
) -> Vec<Metadata<PartitionSpec>> {
    partitions
        .iter()
        .filter(|partition| partition_belongs_to_topic(topic, &partition.name))
        .cloned()
        .collect()
}

fn partition_belongs_to_topic(topic: &str, partition_name: &str) -> bool {
    if let Ok(key) = TryInto::<ReplicaKey>::try_into(partition_name.to_string()) {
        return key.topic == topic;
    }
    // Fallback for names that do not parse as ReplicaKey
    if let Some(index) = partition_name.rfind('-') {
        &partition_name[..index] == topic
    } else {
        partition_name == topic
    }
}

fn partition_id_from_name(name: &str) -> PartitionId {
    if let Ok(key) = TryInto::<ReplicaKey>::try_into(name.to_string()) {
        return key.partition;
    }
    name.rsplit_once('-')
        .and_then(|(_, id)| id.parse().ok())
        .unwrap_or(0)
}

/// Group consumer IDs by partition for the given topic (same source as `consumer list`).
pub(crate) fn group_consumers_by_partition(
    topic: &str,
    consumers: &[ConsumerOffset],
) -> HashMap<PartitionId, Vec<String>> {
    let mut map: HashMap<PartitionId, Vec<String>> = HashMap::new();
    for consumer in consumers {
        if consumer.topic == topic {
            let entry = map.entry(consumer.partition).or_default();
            if !entry.contains(&consumer.consumer_id) {
                entry.push(consumer.consumer_id.clone());
            }
        }
    }
    for ids in map.values_mut() {
        ids.sort();
    }
    map
}

/// Fetch timestamp of the most recent event in the partition (client-side LEO approach).
/// Returns `None` when the partition has no records or the fetch fails.
async fn fetch_last_produced(
    streamfy: &Streamfy,
    topic: &str,
    partition: PartitionId,
    leo: i64,
) -> Option<i64> {
    // Empty partition: LEO is 0 (next write offset); nothing produced yet.
    if leo <= 0 {
        return None;
    }

    let config = ConsumerConfigExt::builder()
        .topic(topic.to_string())
        .partition(partition)
        .offset_start(Offset::from_end(1))
        .disable_continuous(true)
        .build()
        .ok()?;

    let mut stream = streamfy.consumer_with_config(config).await.ok()?;
    // Only need the single latest record (non-continuous fetch returns then ends).
    let record = stream.next().await?.ok()?;

    let ts = record.timestamp();
    if ts <= 0 {
        None
    } else {
        Some(ts)
    }
}

/// One row in the partition status table for topic describe.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct PartitionDescribeRow {
    pub partition: PartitionId,
    pub leader: i32,
    /// LEO (Last End Offset) from partition leader status — shown as LAST-OFFSET.
    pub last_offset: i64,
    /// Milliseconds since Unix epoch of the most recent event, if any.
    pub last_produced: Option<i64>,
    /// Active consumer IDs on this partition.
    pub consumers: Vec<String>,
}

/// Format last-produced timestamp for human-readable output (relative age).
pub(crate) fn format_last_produced(last_produced_ms: Option<i64>, now: SystemTime) -> String {
    let Some(ts_ms) = last_produced_ms else {
        return "-".to_string();
    };
    if ts_ms <= 0 {
        return "-".to_string();
    }
    let produced = UNIX_EPOCH + Duration::from_millis(ts_ms as u64);
    match now.duration_since(produced) {
        Ok(age) => format!("{} ago", humantime::format_duration(truncate_duration(age))),
        Err(_) => "just now".to_string(),
    }
}

/// Truncate duration to seconds for concise display (e.g. "1s" not "1s 12ms").
fn truncate_duration(d: Duration) -> Duration {
    Duration::from_secs(d.as_secs())
}

/// Format consumer list for a partition row.
pub(crate) fn format_consumers(consumers: &[String]) -> String {
    if consumers.is_empty() {
        "-".to_string()
    } else {
        consumers.join(", ")
    }
}

mod display {

    use comfy_table::{Cell, Row};
    use humantime::format_duration;
    use serde::Serialize;
    use std::time::SystemTime;

    use streamfy::metadata::objects::Metadata;
    use streamfy::metadata::topic::ReplicaSpec;
    use streamfy::metadata::topic::TopicSpec;

    use super::{format_consumers, format_last_produced, PartitionDescribeRow};
    use crate::common::output::{
        DescribeObjectHandler, KeyValOutputHandler, OutputError, OutputType, TableOutputHandler,
        Terminal,
    };

    #[allow(clippy::redundant_closure)]
    pub async fn describe_topics<O>(
        topics: Vec<Metadata<TopicSpec>>,
        partition_rows: Vec<PartitionDescribeRow>,
        output_type: OutputType,
        out: std::sync::Arc<O>,
    ) -> Result<(), OutputError>
    where
        O: Terminal,
    {
        let topic_list: Vec<TopicMetadata> = topics
            .into_iter()
            .map(|m| TopicMetadata {
                meta: m,
                partitions: partition_rows.clone(),
            })
            .collect();
        out.describe_objects(&topic_list, output_type)
    }

    #[derive(Serialize, Clone)]
    struct TopicMetadata {
        #[serde(flatten)]
        meta: Metadata<TopicSpec>,
        partitions: Vec<PartitionDescribeRow>,
    }

    impl DescribeObjectHandler for TopicMetadata {
        fn label() -> &'static str {
            "topic"
        }

        fn label_plural() -> &'static str {
            "topics"
        }

        fn is_ok(&self) -> bool {
            true
        }

        fn is_error(&self) -> bool {
            false
        }

        fn validate(&self) -> Result<(), OutputError> {
            Ok(())
        }
    }

    impl TableOutputHandler for TopicMetadata {
        fn header(&self) -> Row {
            Row::from([
                "PARTITION",
                "LEADER",
                "LAST-OFFSET",
                "LAST-PRODUCED",
                "CONSUMERS",
            ])
        }

        fn errors(&self) -> Vec<String> {
            vec![]
        }

        fn content(&self) -> Vec<Row> {
            let now = SystemTime::now();
            self.partitions
                .iter()
                .map(|row| {
                    Row::from([
                        Cell::new(row.partition.to_string()),
                        Cell::new(row.leader.to_string()),
                        Cell::new(row.last_offset.to_string()),
                        Cell::new(format_last_produced(row.last_produced, now)),
                        Cell::new(format_consumers(&row.consumers)),
                    ])
                })
                .collect()
        }
    }

    impl KeyValOutputHandler for TopicMetadata {
        /// key value hash map implementation
        fn key_values(&self) -> Vec<(String, Option<String>)> {
            let mut key_values = Vec::new();
            let spec = &self.meta.spec;
            let status = &self.meta.status;

            key_values.push(("Name".to_owned(), Some(self.meta.name.clone())));
            key_values.push(("Type".to_owned(), Some(spec.type_label().to_string())));
            match spec.replicas() {
                ReplicaSpec::Computed(param) => {
                    key_values.push((
                        "Partition Count".to_owned(),
                        Some(param.partitions.to_string()),
                    ));
                    key_values.push((
                        "Replication Factor".to_owned(),
                        Some(param.replication_factor.to_string()),
                    ));
                    key_values.push((
                        "Ignore Rack Assignment".to_owned(),
                        Some(param.ignore_rack_assignment.to_string()),
                    ));
                }
                ReplicaSpec::Assigned(_partitions) => {}
                ReplicaSpec::Mirror(_config) => {}
            }

            if let Some(dedup) = spec.get_deduplication() {
                key_values.push((
                    "Deduplication Filter".to_owned(),
                    Some(dedup.filter.transform.uses.clone()),
                ));
                key_values.push((
                    "Deduplication Count Bound".to_owned(),
                    Some(dedup.bounds.count)
                        .filter(|c| *c != 0)
                        .as_ref()
                        .map(ToString::to_string),
                ));
                key_values.push((
                    "Deduplication Age Bound".to_owned(),
                    dedup.bounds.age.map(|a| format_duration(a).to_string()),
                ));
            };

            key_values.push((
                "Status".to_owned(),
                Some(status.resolution.resolution_label().to_string()),
            ));
            key_values.push(("Reason".to_owned(), Some(status.reason.clone())));

            key_values.push(("-----------------".to_owned(), None));

            key_values
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use streamfy::metadata::partition::PartitionStatus;
    use streamfy_controlplane_metadata::partition::PartitionSpec as CpPartitionSpec;

    fn partition_meta(name: &str, leader: i32) -> Metadata<PartitionSpec> {
        Metadata {
            name: name.to_string(),
            spec: CpPartitionSpec::new(leader, vec![leader]),
            status: PartitionStatus::default(),
        }
    }

    #[test]
    fn filter_multi_partition_topic() {
        let partitions = vec![
            partition_meta("orders-0", 5001),
            partition_meta("orders-1", 5002),
            partition_meta("orders-2", 5001),
            partition_meta("other-0", 5001),
            partition_meta("orders-extra-0", 5003),
        ];
        let filtered = filter_partitions_by_topic("orders", &partitions);
        assert_eq!(filtered.len(), 3);
        let names: Vec<_> = filtered.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["orders-0", "orders-1", "orders-2"]);
    }

    #[test]
    fn filter_does_not_match_prefix_only() {
        let partitions = vec![
            partition_meta("my-topic-0", 5001),
            partition_meta("my-topic-extra-0", 5001),
        ];
        let filtered = filter_partitions_by_topic("my-topic", &partitions);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "my-topic-0");
    }

    #[test]
    fn filter_empty_partitions() {
        let partitions: Vec<Metadata<PartitionSpec>> = vec![];
        let filtered = filter_partitions_by_topic("any", &partitions);
        assert!(filtered.is_empty());
    }

    #[test]
    fn group_consumers_multiple_and_none() {
        let consumers = vec![
            ConsumerOffset {
                consumer_id: "c1".into(),
                topic: "orders".into(),
                partition: 0,
                offset: 10,
                modified_time: 100,
            },
            ConsumerOffset {
                consumer_id: "c2".into(),
                topic: "orders".into(),
                partition: 0,
                offset: 5,
                modified_time: 101,
            },
            ConsumerOffset {
                consumer_id: "c3".into(),
                topic: "orders".into(),
                partition: 1,
                offset: 0,
                modified_time: 102,
            },
            ConsumerOffset {
                consumer_id: "other".into(),
                topic: "other".into(),
                partition: 0,
                offset: 1,
                modified_time: 103,
            },
        ];
        let grouped = group_consumers_by_partition("orders", &consumers);
        assert_eq!(grouped.get(&0).unwrap(), &vec!["c1".to_string(), "c2".to_string()]);
        assert_eq!(grouped.get(&1).unwrap(), &vec!["c3".to_string()]);
        assert!(grouped.get(&2).is_none());

        let empty = group_consumers_by_partition("missing", &consumers);
        assert!(empty.is_empty());
    }

    #[test]
    fn format_consumers_empty_and_assigned() {
        assert_eq!(format_consumers(&[]), "-");
        assert_eq!(
            format_consumers(&["c1".into(), "c2".into()]),
            "c1, c2"
        );
    }

    #[test]
    fn format_last_produced_none_and_recent() {
        let now = UNIX_EPOCH + Duration::from_secs(1_000);
        assert_eq!(format_last_produced(None, now), "-");
        assert_eq!(format_last_produced(Some(-1), now), "-");

        // 5 seconds before `now`
        let produced_ms = (1_000 - 5) * 1000;
        let formatted = format_last_produced(Some(produced_ms), now);
        assert!(
            formatted.contains("5s") || formatted.contains("5 s"),
            "unexpected format: {formatted}"
        );
        assert!(formatted.ends_with(" ago"));
    }

    #[test]
    fn last_offset_is_leo() {
        // LAST-OFFSET displays LEO directly (including 0 for empty partitions).
        assert_eq!(3543_i64, 3543);
        let row_empty = PartitionDescribeRow {
            partition: 0,
            leader: 5001,
            last_offset: 0,
            last_produced: None,
            consumers: vec![],
        };
        assert_eq!(row_empty.last_offset, 0);
    }

    #[test]
    fn partition_row_no_records_no_consumers() {
        let row = PartitionDescribeRow {
            partition: 0,
            leader: 5001,
            last_offset: 0,
            last_produced: None,
            consumers: vec![],
        };
        assert_eq!(format_consumers(&row.consumers), "-");
        assert_eq!(
            format_last_produced(row.last_produced, SystemTime::now()),
            "-"
        );
        assert_eq!(row.last_offset, 0);
    }

    #[test]
    fn partition_row_with_consumers_and_offset() {
        let row = PartitionDescribeRow {
            partition: 1,
            leader: 5002,
            last_offset: 1240,
            last_produced: Some(1_700_000_000_000),
            consumers: vec!["c3".into()],
        };
        assert_eq!(format_consumers(&row.consumers), "c3");
        assert_eq!(row.last_offset, 1240);
        assert!(row.last_produced.is_some());
    }
}
