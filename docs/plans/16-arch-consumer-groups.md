# Architecture Change B: Consumer Groups (Coordinator)

## Status: Not Started (blocked on durable offsets; compaction preferred first)
## Priority: Critical product | Effort: Large

## Problem

Offsets exist per `consumer_id` on SPU (`kv/consumer.rs`), but there is **no group membership, generation fencing, or partition assignment**. Apps must pin partitions manually. RFC `offset-management.md` explicitly deferred groups.

## Goal

Kafka-style consumer groups with SC coordinator: Join/Sync/Heartbeat/Leave, rebalance, group offsets.

## Key architecture choices

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Coordinator location | **SC** | SC already owns membership of cluster objects; avoids SPU split-brain for small/medium clusters |
| First rebalance | **Eager** | Simpler correctness; co-op sticky in phase 2 |
| Offset storage | Internal topic `__consumer_offsets` on SPU | Survives SC restart; needs **compaction (A)** for bounded growth (TTL ok short-term) |
| Protocol | New SC API keys (not admin Create/List) | Interactive session protocol ≠ CRD CRUD |
| Assignment v1 | Range + RoundRobin | Sticky later |

## State machine

`Empty → PreparingRebalance → CompletingRebalance → Stable → (rebalance) → Dead`

Generation id fences stale members. Session timeout evicts.

## Depends on

- **A Log compaction** for production-safe offset topic (or temporary TTL)
- Existing SPU offset APIs as low-level storage can be reused then retired for group path

## Success criteria

- N members of same `group.id` share partitions
- Crash → reassign within `session_timeout`
- Group offsets survive restart
- CLI `consumer-group list|describe|reset-offsets`

## Detailed protocol

See also legacy `plans/05-consumer-groups.md` (protocol structs remain valid).
