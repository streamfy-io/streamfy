# Architecture Change A: Log Compaction (Storage Primitive)

## Status: Implemented (core path)
## Priority: Critical | Effort: Large | Selected for implementation

## Problem

`CleanupPolicy` has a single variant `Segment` — the cleaner only deletes whole sealed segments by size (`enforce_size`) or age (`enforce_ttl`). There is no key-based retention.

That blocks:

- CDC / latest-value-per-key topics
- Bounded `__consumer_offsets` (and any changelog topic)
- Stream-table joins and durable SmartModule state topics
- Kafka-like `compact` / `compact,delete` workloads

Evidence:

- `CleanupPolicy` in `crates/streamfy-controlplane-metadata/src/topic/spec.rs` — tag 0 only
- `crates/streamfy-storage/src/cleaner.rs` — size + TTL only
- `ReplicaConfig::update_from_replica` only maps `Segment` retention seconds

## Goal

First-class **key-based log compaction** on sealed segments, coexisting with delete-based retention.

## Design decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Policy model | `Segment` \| `Compact` \| `CompactAndDelete` | Matches Kafka mental model; wire-compatible tags |
| Scope | Sealed segments only | Active segment stays append-only; no produce stalls |
| Offsets | **Preserve** original offsets (gaps OK) | Consumer offsets remain valid across cycles |
| Null keys | Never compacted | Append-only / log-like records |
| Tombstones | key set + empty value; drop after `delete_retention_secs` | Standard delete-by-key |
| Trigger | Dirty ratio ≥ `min_cleanable_dirty_ratio` | Avoid useless rewrites |
| Concurrency | Compaction under segment list write lock only at swap | Reads continue on open segments until swap |

## Algorithm

1. **Eligibility**: `compact_enabled`; at least one sealed segment; dirty ratio ≥ threshold.
2. **Scan** sealed segments oldest→newest (or newest→oldest for latest-wins map).
3. **Keep set**:
   - All null-key records
   - Latest record per non-null key (by absolute offset)
   - Drop tombstones older than `delete_retention_secs`
   - Records younger than `min_compaction_lag_secs` always eligible to stay if still latest
4. **Rewrite** into new segments under `{replica_dir}/.compact/` using preserved offsets (single-record batches).
5. **Swap**: remove old sealed segments from map + disk; move compacted files into replica dir; open as `ReadSegment`s; update replica size.
6. Active segment untouched.

## Config surface

```text
cleanup_policy = segment | compact | compact,delete
min_cleanable_dirty_ratio   default 50 (%)
delete_retention_secs      default 86400
min_compaction_lag_secs    default 0
```

Topic CLI: `--cleanup-policy compact|segment|compact,delete`

## Components

| Component | Change |
|-----------|--------|
| `streamfy-controlplane-metadata` | `CompactPolicy`, new enum variants |
| `streamfy-storage` config | map policy → compact/delete flags + knobs |
| `streamfy-storage` segment | `append_batch_preserving_offset` |
| `streamfy-storage` segments | `replace_segments` |
| `streamfy-storage` compact | new module: scan, dirty ratio, rewrite, swap |
| `streamfy-storage` cleaner | call compact path when enabled |
| `streamfy-cli` | topic create flag |

## Risks & mitigations

| Risk | Mitigation |
|------|------------|
| Produce blocked by rewrite | Never rewrite active; swap under brief lock |
| Crash mid-compact | Work in `.compact/`; only swap after full success; temp dir cleaned on next start |
| High key cardinality memory | v1 in-memory map; document limit; disk-backed map later |
| Empty segments after compact | Skip writing empty; may remove all sealed if fully superseded + tombstoned |

## Success criteria

- [x] Design + metadata policy variants
- [x] Topic with `compact` retains latest value per key after cleaner run (unit tests)
- [x] Tombstones expire after retention (delete_retention_secs filter in keep set)
- [x] Offset gaps preserved (unit test `test_compact_preserves_offsets`)
- [x] Active segment untouched (only sealed segments rewritten)
- [x] Unit tests in `streamfy-storage` (`compact` module)
- [ ] End-to-end cluster integration test (follow-up)
- [ ] min_compaction_lag_secs enforcement (config present; lag filter follow-up)

## PR Plan

1. **Metadata + config** — enum, defaults, `update_from_replica`
2. **Preserving write path + segment replace**
3. **Compactor + cleaner integration**
4. **CLI + tests**

## Non-goals (this change)

- Compacting the active segment
- Cooperative multi-partition cleaner pool
- Disk-backed offset maps
- Kafka wire protocol
