# Task 4: Tiered Storage (S3/Cloud Offloading)

## Status: Not Started
## Priority: Medium | Effort: Large

## Problem

All data lives on local disk attached to SPUs. This means:
- Storage capacity is limited by local disk size.
- Old data must be deleted (TTL/size retention) even if it has analytical value.
- SPU recovery after disk failure requires full replication resync.
- Cost: fast NVMe storage for cold data that is rarely accessed.

## Goal

Introduce a two-tier storage model:
- **Hot tier**: Local disk (current behavior) — recent segments, actively read/written.
- **Cold tier**: Object storage (S3, GCS, MinIO) — older segments, read on demand.

## Design

### Architecture

```
Producer → SPU (hot tier: local segments)
                  ↓ (background upload)
              Object Store (cold tier: S3/GCS/MinIO)
                  ↑ (on-demand fetch)
           Consumer reading old offsets
```

### Segment Lifecycle

1. Segment is created and written to on local disk (hot).
2. When a segment is sealed (rolled to a new active segment), it becomes eligible for upload.
3. Background uploader copies the sealed segment to object storage.
4. After upload confirmation + configurable `local.retention.ms`, the local copy is deleted.
5. Fetch requests for offsets in cold segments trigger a download-on-read from object storage.

### Storage Backend Trait

```rust
#[async_trait]
pub trait RemoteStorage: Send + Sync {
    async fn upload_segment(&self, topic: &str, partition: i32, segment: &SegmentMetadata, data: &[u8]) -> Result<()>;
    async fn download_segment(&self, topic: &str, partition: i32, base_offset: i64) -> Result<Vec<u8>>;
    async fn delete_segment(&self, topic: &str, partition: i32, base_offset: i64) -> Result<()>;
    async fn list_segments(&self, topic: &str, partition: i32) -> Result<Vec<SegmentMetadata>>;
}
```

Initial implementations: S3 (via `aws-sdk-s3`) and filesystem (for testing).

### Configuration

```toml
[storage.tiered]
enabled = true
remote_backend = "s3"
bucket = "streamfy-cold-storage"
region = "us-east-1"
upload_delay_ms = 60000           # wait before uploading sealed segments
local_retention_after_upload_ms = 3600000  # keep local copy for 1h after upload
```

## Key Files to Modify

- `crates/streamfy-storage/` — new `remote/` module with `RemoteStorage` trait and S3 impl
- `crates/streamfy-storage/src/cleaner.rs` — trigger upload before local cleanup
- `crates/streamfy-storage/src/replica.rs` — fetch from remote on cache miss
- `crates/streamfy-spu/src/services/public/fetch_handler.rs` — handle remote fetch transparently
- New crate: `crates/streamfy-storage-s3/` (optional, feature-gated)

## Risks

- Fetch latency for cold reads (S3 GET ~50-200ms vs local <1ms). Mitigate with local segment caching.
- Upload failures must not block SPU operation. Use async retry with dead-letter.
- Consistency: segment must be fully uploaded before local deletion.

## Success Criteria

- Sealed segments are uploaded to S3 within `upload_delay_ms`.
- Consumers can transparently read old data from cold storage.
- Local disk usage is bounded by `local_retention_after_upload_ms`.
- SPU operates normally if S3 is temporarily unavailable (degrades to local-only).
