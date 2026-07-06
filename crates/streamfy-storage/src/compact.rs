//! Key-based log compaction for sealed segments.
//!
//! Retains the latest record per non-null key (and all null-key records),
//! preserves original absolute offsets (gaps allowed), and swaps rewritten
//! segments into the replica's sealed segment list.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use tracing::{debug, info, instrument, warn};

use streamfy_future::fs::{create_dir_all, remove_dir_all};
use streamfy_protocol::record::{Batch, MemoryRecords, Offset, Record};

use crate::batch::FileBatchStream;
use crate::config::{ReplicaConfig, SharedReplicaConfig};
use crate::records::FileRecords;
use crate::segment::MutableSegment;
use crate::segment::ReadSegment;
use crate::segments::SharedSegments;
use crate::util::generate_file_name;
use crate::StorageError;

const COMPACT_TMP_DIR: &str = ".compact";

/// A retained record candidate with its absolute log offset.
#[derive(Clone)]
struct KeptRecord {
    offset: Offset,
    record: Record,
    /// Batch max timestamp (ms) used for tombstone age.
    timestamp_ms: i64,
    is_tombstone: bool,
}

/// Run compaction on sealed segments if enabled and dirty enough.
///
/// Returns `Ok(true)` when a compaction swap was performed.
#[instrument(skip(replica_config, segments))]
pub(crate) async fn maybe_compact(
    replica_config: &Arc<SharedReplicaConfig>,
    segments: &Arc<SharedSegments>,
) -> Result<bool> {
    if !replica_config.compact_enabled() {
        return Ok(false);
    }

    let old_bases = segments.base_offsets().await;
    if old_bases.is_empty() {
        debug!("no sealed segments to compact");
        return Ok(false);
    }

    let scan = scan_sealed_segments(segments).await?;
    if scan.total_records == 0 {
        return Ok(false);
    }

    let dirty_ratio_pct = scan.dirty_ratio_percent();
    let threshold = replica_config.min_cleanable_dirty_ratio.get();
    debug!(
        total = scan.total_records,
        kept_keys = scan.kept.len(),
        null_keys = scan.null_key_records.len(),
        dirty_ratio_pct,
        threshold,
        "compaction dirty ratio"
    );

    if dirty_ratio_pct < threshold {
        debug!("dirty ratio below threshold; skipping compaction");
        return Ok(false);
    }

    let delete_retention_ms =
        (replica_config.delete_retention_secs.get() as i64).saturating_mul(1000);
    let now_ms = now_epoch_ms();

    let mut kept: Vec<KeptRecord> = scan
        .kept
        .into_values()
        .filter(|k| {
            if k.is_tombstone {
                if k.timestamp_ms < 0 {
                    return true;
                }
                let age = now_ms.saturating_sub(k.timestamp_ms);
                age < delete_retention_ms
            } else {
                true
            }
        })
        .collect();
    kept.extend(scan.null_key_records);
    kept.sort_by_key(|k| k.offset);

    let kept_count = kept.len();
    if kept_count == scan.total_records {
        debug!("nothing to remove; skipping compaction rewrite");
        return Ok(false);
    }

    info!(
        sealed_segments = old_bases.len(),
        total_records = scan.total_records,
        kept = kept_count,
        dirty_ratio_pct,
        "compacting sealed segments"
    );

    // 1. Rewrite into temp directory (does not touch live segment files).
    let written_bases = rewrite_segments(replica_config, &kept).await?;

    // 2. Drop old sealed segments from the map and delete their files.
    segments.replace_segments(&old_bases, Vec::new()).await;

    // 3. Install compacted files into the replica directory and open them.
    let opened = install_compacted_segments(replica_config, &written_bases).await?;

    // 4. Publish new sealed segments.
    for segment in opened {
        segments.add_segment(segment).await;
    }

    let tmp = compact_tmp_dir(&replica_config.base_dir);
    if tmp.exists()
        && let Err(err) = remove_dir_all(&tmp).await
    {
        warn!(?err, path = %tmp.display(), "failed to remove compact temp dir");
    }

    info!(
        new_segments = written_bases.len(),
        "compaction swap complete"
    );
    Ok(true)
}

struct ScanResult {
    kept: HashMap<Vec<u8>, KeptRecord>,
    null_key_records: Vec<KeptRecord>,
    total_records: usize,
    dirty_records: usize,
}

impl ScanResult {
    fn dirty_ratio_percent(&self) -> u32 {
        if self.total_records == 0 {
            return 0;
        }
        ((self.dirty_records * 100) / self.total_records) as u32
    }
}

async fn scan_sealed_segments(segments: &Arc<SharedSegments>) -> Result<ScanResult> {
    let reader = segments.read().await;
    let mut kept: HashMap<Vec<u8>, KeptRecord> = HashMap::new();
    let mut null_key_records = Vec::new();
    let mut total_records = 0usize;

    // Oldest → newest; last write for each key wins (latest offset).
    for segment in reader.segments_in_order() {
        let path = segment.get_msg_log().get_path();
        let mut stream: FileBatchStream<MemoryRecords> = FileBatchStream::open(&path).await?;
        while let Some(file_batch) = stream.try_next().await? {
            let batch = file_batch.inner();
            let base = batch.get_base_offset();
            let ts = batch.get_header().max_time_stamp;
            for record in batch.own_records() {
                total_records += 1;
                let abs_offset = base + record.offset_delta();
                match record.key() {
                    None => {
                        null_key_records.push(KeptRecord {
                            offset: abs_offset,
                            record: record.clone(),
                            timestamp_ms: ts,
                            is_tombstone: false,
                        });
                    }
                    Some(key_data) => {
                        let key = key_data.as_ref().to_vec();
                        let is_tombstone = record.value().is_empty();
                        let candidate = KeptRecord {
                            offset: abs_offset,
                            record: record.clone(),
                            timestamp_ms: ts,
                            is_tombstone,
                        };
                        match kept.entry(key) {
                            std::collections::hash_map::Entry::Vacant(e) => {
                                e.insert(candidate);
                            }
                            std::collections::hash_map::Entry::Occupied(mut e) => {
                                if abs_offset >= e.get().offset {
                                    e.insert(candidate);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let kept_total = kept.len() + null_key_records.len();
    let dirty_records = total_records.saturating_sub(kept_total);

    Ok(ScanResult {
        kept,
        null_key_records,
        total_records,
        dirty_records,
    })
}

/// Write kept records into new segment files under the compact temp directory.
/// Returns base offsets of segments written (sorted ascending).
async fn rewrite_segments(
    replica_config: &Arc<SharedReplicaConfig>,
    kept: &[KeptRecord],
) -> Result<Vec<Offset>> {
    if kept.is_empty() {
        return Ok(Vec::new());
    }

    let tmp_dir = compact_tmp_dir(&replica_config.base_dir);
    if tmp_dir.exists() {
        remove_dir_all(&tmp_dir).await?;
    }
    create_dir_all(&tmp_dir).await?;

    let write_config = replica_config_for_dir(replica_config, tmp_dir);
    let option = write_config.shared();

    let mut written_bases = Vec::new();
    let first_offset = kept[0].offset;
    let mut active = MutableSegment::create(first_offset, option.clone()).await?;
    written_bases.push(first_offset);

    for kept_rec in kept {
        let mut batch = single_record_batch(kept_rec);
        match active.append_batch_preserving_offset(&mut batch).await? {
            true => {}
            false => {
                // Segment full — roll and retry once.
                active.close().await?;
                drop(active);

                let next_base = kept_rec.offset;
                active = MutableSegment::create(next_base, option.clone()).await?;
                written_bases.push(next_base);

                let mut retry = single_record_batch(kept_rec);
                if !active.append_batch_preserving_offset(&mut retry).await? {
                    return Err(StorageError::Other(
                        "failed to write record during compaction after segment roll".into(),
                    )
                    .into());
                }
            }
        }
    }

    active.close().await?;
    drop(active);

    Ok(written_bases)
}

/// Move compacted segment files from temp dir into replica base_dir and open them.
async fn install_compacted_segments(
    replica_config: &Arc<SharedReplicaConfig>,
    bases: &[Offset],
) -> Result<Vec<ReadSegment>> {
    let tmp = compact_tmp_dir(&replica_config.base_dir);
    let mut opened = Vec::with_capacity(bases.len());

    for &base in bases {
        for ext in ["log", "index"] {
            let src = generate_file_name(&tmp, base, ext);
            let dst = generate_file_name(&replica_config.base_dir, base, ext);
            if !src.exists() {
                return Err(StorageError::Other(format!(
                    "compacted {ext} missing for offset {base} at {}",
                    src.display()
                ))
                .into());
            }
            if dst.exists() {
                std::fs::remove_file(&dst).map_err(StorageError::Io)?;
            }
            std::fs::rename(&src, &dst).map_err(|err| {
                StorageError::Other(format!(
                    "failed to install compacted {ext} for offset {base}: {err}"
                ))
            })?;
        }

        let segment = ReadSegment::open_unknown(base, replica_config.clone()).await?;
        opened.push(segment);
    }

    Ok(opened)
}

fn compact_tmp_dir(base: &Path) -> PathBuf {
    base.join(COMPACT_TMP_DIR)
}

fn replica_config_for_dir(src: &SharedReplicaConfig, base_dir: PathBuf) -> ReplicaConfig {
    ReplicaConfig {
        base_dir,
        index_max_bytes: src.index_max_bytes.get(),
        index_max_interval_bytes: src.index_max_interval_bytes.get(),
        segment_max_bytes: src.segment_max_bytes.get(),
        flush_write_count: src.flush_write_count.get(),
        flush_idle_msec: src.flush_idle_msec.get(),
        max_batch_size: src.max_batch_size.get(),
        max_request_size: src.max_request_size.get(),
        update_hw: src.update_hw,
        retention_seconds: src.retention_seconds.get(),
        max_partition_size: src.max_partition_size.get(),
        compact_enabled: src.compact_enabled(),
        delete_enabled: src.delete_enabled(),
        min_cleanable_dirty_ratio: src.min_cleanable_dirty_ratio.get(),
        delete_retention_secs: src.delete_retention_secs.get(),
        min_compaction_lag_secs: src.min_compaction_lag_secs.get(),
    }
}

fn single_record_batch(kept: &KeptRecord) -> Batch<MemoryRecords> {
    let mut batch = Batch::default();
    let header = batch.get_mut_header();
    header.magic = 2;
    header.producer_id = -1;
    header.producer_epoch = -1;
    if kept.timestamp_ms >= 0 {
        header.first_timestamp = kept.timestamp_ms;
        header.max_time_stamp = kept.timestamp_ms;
    }

    let mut record = kept.record.clone();
    record.preamble.set_offset_delta(0);
    batch.add_record(record);
    // add_record renumbers deltas — ensure base offset is the absolute offset.
    batch.set_base_offset(kept.offset);
    batch
}

fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    use streamfy_util::fixture::ensure_new_dir;
    use streamfy_protocol::record::Record;

    use crate::config::ReplicaConfig;
    use crate::segments::{SegmentList, SharedSegments};

    async fn seal_segment(
        option: Arc<SharedReplicaConfig>,
        base: Offset,
        records: Vec<(Option<&[u8]>, &[u8])>,
        start_offset: Offset,
    ) -> ReadSegment {
        let mut seg = MutableSegment::create(base, option).await.expect("create");
        for (i, (key, value)) in records.into_iter().enumerate() {
            let abs = start_offset + i as Offset;
            let mut batch = Batch::default();
            batch.get_mut_header().magic = 2;
            let record = match key {
                Some(k) => Record::new_key_value(k.to_vec(), value.to_vec()),
                None => Record::new(value.to_vec()),
            };
            batch.add_record(record);
            batch.set_base_offset(abs);
            seg.append_batch_preserving_offset(&mut batch)
                .await
                .expect("append")
                .then_some(())
                .expect("segment had room");
        }
        seg.close().await.expect("close");
        seg.as_segment().await.expect("as_segment")
    }

    #[streamfy_future::test]
    async fn test_compact_keeps_latest_per_key() {
        let dir = temp_dir().join("compact-latest-key");
        ensure_new_dir(&dir).expect("dir");

        let cfg = ReplicaConfig {
            base_dir: dir.clone(),
            segment_max_bytes: 10_000_000,
            index_max_bytes: 10_000,
            index_max_interval_bytes: 0,
            compact_enabled: true,
            delete_enabled: false,
            min_cleanable_dirty_ratio: 0,
            flush_write_count: 1,
            ..Default::default()
        };
        let option = cfg.shared();

        let seg1 = seal_segment(
            option.clone(),
            0,
            vec![
                (Some(b"a"), b"v1"),
                (Some(b"b"), b"v1"),
                (Some(b"a"), b"v2"),
            ],
            0,
        )
        .await;

        let segments = SharedSegments::from(SegmentList::new());
        segments.add_segment(seg1).await;

        let ran = maybe_compact(&option, &segments).await.expect("compact");
        assert!(ran, "compaction should run");

        let read = segments.read().await;
        assert_eq!(read.len(), 1);

        let path = read
            .segments_in_order()
            .next()
            .unwrap()
            .get_msg_log()
            .get_path();
        let mut stream: FileBatchStream<MemoryRecords> =
            FileBatchStream::open(&path).await.expect("open");
        let mut found: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        while let Some(fb) = stream.try_next().await.expect("next") {
            let batch = fb.inner();
            for rec in batch.own_records() {
                if let Some(k) = rec.key() {
                    found.insert(k.as_ref().to_vec(), rec.value().as_ref().to_vec());
                }
            }
        }
        assert_eq!(
            found.get(b"a".as_slice()).map(|v| v.as_slice()),
            Some(b"v2".as_slice())
        );
        assert_eq!(
            found.get(b"b".as_slice()).map(|v| v.as_slice()),
            Some(b"v1".as_slice())
        );
        assert_eq!(found.len(), 2);
    }

    #[streamfy_future::test]
    async fn test_compact_preserves_offsets() {
        let dir = temp_dir().join("compact-offsets");
        ensure_new_dir(&dir).expect("dir");

        let cfg = ReplicaConfig {
            base_dir: dir.clone(),
            segment_max_bytes: 10_000_000,
            index_max_bytes: 10_000,
            index_max_interval_bytes: 0,
            compact_enabled: true,
            delete_enabled: false,
            min_cleanable_dirty_ratio: 0,
            flush_write_count: 1,
            ..Default::default()
        };
        let option = cfg.shared();

        let seg1 = seal_segment(
            option.clone(),
            0,
            vec![(Some(b"k"), b"old"), (Some(b"k"), b"new")],
            0,
        )
        .await;
        let segments = SharedSegments::from(SegmentList::new());
        segments.add_segment(seg1).await;

        maybe_compact(&option, &segments).await.expect("compact");

        let read = segments.read().await;
        let path = read
            .segments_in_order()
            .next()
            .unwrap()
            .get_msg_log()
            .get_path();
        let mut stream: FileBatchStream<MemoryRecords> =
            FileBatchStream::open(&path).await.expect("open");
        let mut offsets = Vec::new();
        while let Some(fb) = stream.try_next().await.expect("next") {
            let batch = fb.inner();
            let base = batch.get_base_offset();
            for rec in batch.own_records() {
                offsets.push(base + rec.offset_delta());
            }
        }
        // Latest for key k was at offset 1
        assert_eq!(offsets, vec![1]);
    }
}
