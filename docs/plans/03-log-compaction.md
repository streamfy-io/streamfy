# Task 3: Log Compaction

## Status: Not Started
## Priority: High | Effort: Large

## Problem

Streamfy only supports `CleanupPolicy::Segment` — TTL-based and size-based segment removal. There is no key-based log compaction. This means:

- Use cases like CDC (Change Data Capture), configuration state, and materialized views cannot keep the latest value per key while discarding old versions.
- Topics grow unbounded or lose history entirely when segments are cleaned.
- Kafka has `cleanup.policy=compact` which is table-stakes for many production workloads.

The `CleanupPolicy` enum in `crates/streamfy-storage/src/config.rs` has a single `Segment` variant — there is no `Compact` variant.

## Goal

Add a `Compact` cleanup policy that retains only the latest record for each key within a topic partition, removing superseded records during background compaction.

## Design

### Data Model

- Records already have a key field (`RecordKey` in `streamfy-protocol`). Compaction uses this key.
- Records with `null` key are never compacted (append-only behavior preserved).
- Tombstones: a record with a non-null key and null/empty value marks the key for deletion. After a configurable `delete.retention.ms`, the tombstone itself is removed.

### Compaction Process

1. **Offset map building**: Scan the log from newest to oldest, building a `HashMap<Key, Offset>` of the latest offset for each key.
2. **Rewrite**: Create new segments containing only the records whose offsets appear in the map. Preserve original offsets (compacted log has offset gaps).
3. **Swap**: Atomically replace old segments with compacted segments.
4. **Dirty ratio trigger**: Only compact when the ratio of dirty (potentially superseded) records exceeds `min.cleanable.dirty.ratio` (default 0.5).

### Configuration

```toml
[topic.config]
cleanup_policy = "compact"           # or "delete" (current segment-based) or "compact,delete"
min_cleanable_dirty_ratio = 0.5
delete_retention_ms = 86400000       # 24h before tombstones are removed
min_compaction_lag_ms = 0            # minimum age before a record can be compacted
```

## Key Files to Modify

- `crates/streamfy-storage/src/config.rs` — add `Compact` and `CompactDelete` variants to `CleanupPolicy`
- `crates/streamfy-storage/src/cleaner.rs` — add compaction logic alongside existing `enforce_ttl`/`enforce_size`
- `crates/streamfy-storage/src/replica.rs` — expose compaction trigger
- `crates/streamfy-controlplane-metadata/src/topic/spec.rs` — add topic-level compaction config
- `crates/streamfy-protocol/src/record/batch.rs` — ensure key extraction is efficient

## Risks

- Compaction under write load requires careful locking to avoid blocking producers.
- Offset gaps after compaction must not break consumer offset tracking.
- Memory usage for the offset map can be large for high-cardinality key spaces — may need a disk-backed approach for very large partitions.

## Success Criteria

- A topic with `cleanup.policy=compact` retains only the latest value per key.
- Tombstone records are removed after `delete.retention.ms`.
- Compaction does not block produce/consume operations.
- Consumer offsets remain valid across compaction cycles.
