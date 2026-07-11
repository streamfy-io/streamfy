use std::default::Default;
use std::path::PathBuf;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64};

use derive_builder::Builder;
use streamfy_controlplane::replica::Replica;
use serde::Deserialize;

use streamfy_controlplane_metadata::topic::{CleanupPolicy, CompactionPolicy};
use streamfy_types::defaults::{
    SPU_LOG_INDEX_MAX_BYTES, SPU_LOG_BASE_DIR, STORAGE_FLUSH_WRITE_COUNT, STORAGE_FLUSH_IDLE_MSEC,
    STORAGE_MAX_BATCH_SIZE, STORAGE_MAX_REQUEST_SIZE, STORAGE_RETENTION_SECONDS,
    SPU_PARTITION_MAX_BYTES,
};
use streamfy_types::defaults::SPU_LOG_INDEX_MAX_INTERVAL_BYTES;
use streamfy_types::defaults::SPU_LOG_SEGMENT_MAX_BYTES;
use streamfy_protocol::record::{Size, Size64};

use crate::ReplicaStorageConfig;

// Replica specific config
#[derive(Builder, Clone, Debug, Eq, PartialEq, Deserialize)]
#[builder(build_fn(private, name = "build_impl"))]
pub struct ReplicaConfig {
    #[builder(default = "default_base_dir()")]
    #[serde(default = "default_base_dir")]
    pub base_dir: PathBuf,
    #[builder(default = "default_index_max_bytes()")]
    #[serde(default = "default_index_max_bytes")]
    pub index_max_bytes: Size,
    #[builder(default = "default_index_max_interval_bytes()")]
    #[serde(default = "default_index_max_interval_bytes")]
    pub index_max_interval_bytes: Size,
    #[builder(default = "default_segment_max_bytes()")]
    #[serde(default = "default_segment_max_bytes")]
    pub segment_max_bytes: Size,
    #[builder(default = "default_flush_write_count()")]
    #[serde(default = "default_flush_write_count")]
    pub flush_write_count: Size,
    #[builder(default = "default_flush_idle_msec()")]
    #[serde(default = "default_flush_idle_msec")]
    pub flush_idle_msec: Size,
    #[builder(default = "default_max_batch_size()")]
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: Size,
    #[builder(default = "default_max_request_size()")]
    #[serde(default = "default_max_request_size")]
    pub max_request_size: Size,
    #[builder(default = "default_update_hw()")]
    #[serde(default = "default_update_hw")]
    pub update_hw: bool, // if true, enable hw update
    #[builder(default = "default_retention_seconds()")]
    #[serde(default = "default_retention_seconds")]
    pub retention_seconds: Size,
    #[builder(default = "default_max_partition_size()")]
    #[serde(default = "default_max_partition_size")]
    pub max_partition_size: Size64,
    /// Whether log compaction is enabled for this replica.
    #[builder(default = "false")]
    #[serde(default)]
    pub compaction_enabled: bool,
    /// Whether TTL/size deletion is also enabled (for CompactAndDelete).
    #[builder(default = "false")]
    #[serde(default)]
    pub compaction_delete_enabled: bool,
    /// Minimum ratio of dirty (uncompacted) bytes to total bytes (0–100).
    #[builder(default = "default_min_cleanable_dirty_ratio()")]
    #[serde(default = "default_min_cleanable_dirty_ratio")]
    pub min_cleanable_dirty_ratio: u8,
    /// How long to retain tombstones before dropping them (seconds).
    #[builder(default = "default_delete_retention_secs()")]
    #[serde(default = "default_delete_retention_secs")]
    pub delete_retention_secs: u32,
    /// Minimum age before a record is eligible for compaction (seconds).
    #[builder(default = "default_min_compaction_lag_secs()")]
    #[serde(default = "default_min_compaction_lag_secs")]
    pub min_compaction_lag_secs: u32,
}

impl fmt::Display for ReplicaConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "storage config at: {:#?}", self.base_dir)
    }
}

impl ReplicaStorageConfig for ReplicaConfig {
    fn update_from_replica(&mut self, replica: &Replica) {
        if let Some(policy) = &replica.cleanup_policy {
            match policy {
                CleanupPolicy::Segment(segment) => {
                    self.retention_seconds = segment.retention_secs();
                    self.compaction_enabled = false;
                    self.compaction_delete_enabled = false;
                }
                CleanupPolicy::Compact(compact) => {
                    self.compaction_enabled = true;
                    self.compaction_delete_enabled = false;
                    self.apply_compaction_config(compact);
                }
                CleanupPolicy::CompactAndDelete(compact, segment) => {
                    self.retention_seconds = segment.retention_secs();
                    self.compaction_enabled = true;
                    self.compaction_delete_enabled = true;
                    self.apply_compaction_config(compact);
                }
            }
        }

        if let Some(segment_size) = replica
            .storage
            .as_ref()
            .and_then(|storage| storage.segment_size)
        {
            self.segment_max_bytes = segment_size;
        }
        if let Some(max_partition_size) = replica
            .storage
            .as_ref()
            .and_then(|storage| storage.max_partition_size)
        {
            self.max_partition_size = max_partition_size;
        }
    }
}

impl ReplicaConfig {
    fn apply_compaction_config(&mut self, compact: &CompactionPolicy) {
        self.min_cleanable_dirty_ratio = compact.min_cleanable_dirty_ratio;
        self.delete_retention_secs = compact.delete_retention_secs;
        self.min_compaction_lag_secs = compact.min_compaction_lag_secs;
    }
}

fn default_base_dir() -> PathBuf {
    PathBuf::from(SPU_LOG_BASE_DIR)
}

const fn default_update_hw() -> bool {
    true
}

const fn default_index_max_bytes() -> Size {
    SPU_LOG_INDEX_MAX_BYTES
}

const fn default_index_max_interval_bytes() -> Size {
    SPU_LOG_INDEX_MAX_INTERVAL_BYTES
}

const fn default_segment_max_bytes() -> Size {
    SPU_LOG_SEGMENT_MAX_BYTES
}

const fn default_flush_write_count() -> Size {
    STORAGE_FLUSH_WRITE_COUNT
}

const fn default_flush_idle_msec() -> Size {
    STORAGE_FLUSH_IDLE_MSEC
}

const fn default_max_batch_size() -> Size {
    STORAGE_MAX_BATCH_SIZE
}

const fn default_max_request_size() -> Size {
    STORAGE_MAX_REQUEST_SIZE
}

const fn default_retention_seconds() -> Size {
    STORAGE_RETENTION_SECONDS
}

const fn default_max_partition_size() -> Size64 {
    SPU_PARTITION_MAX_BYTES
}

const fn default_min_cleanable_dirty_ratio() -> u8 {
    50
}

const fn default_delete_retention_secs() -> u32 {
    86400
}

const fn default_min_compaction_lag_secs() -> u32 {
    0
}

impl ReplicaConfig {
    // Used to get a [`ConfigOptionBuilder`].
    pub fn builder() -> ReplicaConfigBuilder {
        ReplicaConfigBuilder::default()
    }

    pub fn shared(self) -> Arc<SharedReplicaConfig> {
        Arc::new(self.into())
    }
}

impl Default for ReplicaConfig {
    fn default() -> Self {
        Self {
            base_dir: default_base_dir(),
            index_max_bytes: default_index_max_bytes(),
            index_max_interval_bytes: default_index_max_interval_bytes(),
            segment_max_bytes: default_segment_max_bytes(),
            flush_write_count: default_flush_write_count(),
            flush_idle_msec: default_flush_idle_msec(),
            max_batch_size: default_max_batch_size(),
            max_request_size: default_max_request_size(),
            retention_seconds: default_retention_seconds(),
            max_partition_size: default_max_partition_size(),
            update_hw: true,
            compaction_enabled: false,
            compaction_delete_enabled: false,
            min_cleanable_dirty_ratio: default_min_cleanable_dirty_ratio(),
            delete_retention_secs: default_delete_retention_secs(),
            min_compaction_lag_secs: default_min_compaction_lag_secs(),
        }
    }
}

impl ReplicaConfigBuilder {
    /// Build a [`ConfigOption`] with the current
    /// values in [`ConfigOptionBuilder`]
    pub fn build(&self) -> ReplicaConfig {
        // This will not fail as long as all fields
        // of ConfigOption have a #[builder(default = ..) attribute.
        self.build_impl()
            .expect("Error builing ConfigOption struct")
    }
}

#[derive(Debug)]
pub struct SharedConfigValue<V: Sync + Send>(V);

impl SharedConfigValue<AtomicU32> {
    pub fn new(value: u32) -> Self {
        SharedConfigValue(AtomicU32::new(value))
    }

    #[inline(always)]
    pub fn get(&self) -> u32 {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn get_consistent(&self) -> u32 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[inline(always)]
    pub fn set(&self, value: u32) {
        self.0.store(value, std::sync::atomic::Ordering::Relaxed)
    }
}

impl SharedConfigValue<AtomicU64> {
    pub fn new(value: u64) -> Self {
        SharedConfigValue(AtomicU64::new(value))
    }

    #[inline(always)]
    pub fn get(&self) -> u64 {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn get_consistent(&self) -> u64 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[inline(always)]
    pub fn set(&self, value: u64) {
        self.0.store(value, std::sync::atomic::Ordering::Relaxed)
    }
}

pub type SharedConfigU32Value = SharedConfigValue<AtomicU32>;
pub type SharedConfigU64Value = SharedConfigValue<AtomicU64>;

/// Config that can be shared updated
#[derive(Debug)]
pub struct SharedReplicaConfig {
    pub base_dir: PathBuf,
    pub index_max_bytes: SharedConfigU32Value,
    pub index_max_interval_bytes: SharedConfigU32Value,
    pub segment_max_bytes: SharedConfigU32Value,
    pub flush_write_count: SharedConfigU32Value,
    pub flush_idle_msec: SharedConfigU32Value,
    pub max_batch_size: SharedConfigU32Value,
    pub max_request_size: SharedConfigU32Value,
    pub update_hw: bool, // if true, enable hw update
    pub retention_seconds: SharedConfigU32Value,
    pub max_partition_size: SharedConfigU64Value,
    pub compaction_enabled: bool,
    pub compaction_delete_enabled: bool,
    pub min_cleanable_dirty_ratio: u8,
    pub delete_retention_secs: u32,
    pub min_compaction_lag_secs: u32,
}

impl From<ReplicaConfig> for SharedReplicaConfig {
    fn from(config: ReplicaConfig) -> Self {
        SharedReplicaConfig {
            base_dir: config.base_dir,
            index_max_bytes: SharedConfigU32Value::new(config.index_max_bytes),
            index_max_interval_bytes: SharedConfigU32Value::new(config.index_max_interval_bytes),
            segment_max_bytes: SharedConfigU32Value::new(config.segment_max_bytes),
            flush_write_count: SharedConfigU32Value::new(config.flush_write_count),
            flush_idle_msec: SharedConfigU32Value::new(config.flush_idle_msec),
            max_batch_size: SharedConfigU32Value::new(config.max_batch_size),
            max_request_size: SharedConfigU32Value::new(config.max_request_size),
            update_hw: config.update_hw,
            retention_seconds: SharedConfigU32Value::new(config.retention_seconds),
            max_partition_size: SharedConfigU64Value::new(config.max_partition_size),
            compaction_enabled: config.compaction_enabled,
            compaction_delete_enabled: config.compaction_delete_enabled,
            min_cleanable_dirty_ratio: config.min_cleanable_dirty_ratio,
            delete_retention_secs: config.delete_retention_secs,
            min_compaction_lag_secs: config.min_compaction_lag_secs,
        }
    }
}

/// Storage wide configuration independent of replica
#[derive(Builder, Debug, Clone)]
pub struct StorageConfig {
    #[builder(default = "10000")] // 10 seconds
    pub cleaning_interval_ms: u16,
}

impl StorageConfig {
    pub fn builder() -> StorageConfigBuilder {
        StorageConfigBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_builder() {
        let config: ReplicaConfig = ReplicaConfig::builder().build();

        assert_eq!(ReplicaConfig::default(), config);
    }
}
