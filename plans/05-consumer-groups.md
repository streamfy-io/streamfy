# Task 5: Consumer Groups with Rebalancing

## Status: Not Started
## Priority: High | Effort: Large

## Problem

Streamfy has basic consumer offset management (per-consumer-id offset tracking with TTL, as described in `rfc/offset-management.md`) but no consumer group coordination. Each consumer independently reads from specific partitions. There is no:

- Automatic partition assignment across multiple consumers in a group.
- Rebalancing when consumers join or leave.
- Cooperative or eager rebalancing protocols.

This means users must manually assign partitions to consumers, which is impractical for horizontally-scaled applications.

## Goal

Implement consumer groups where multiple consumers with the same `group.id` share partitions of a topic, with automatic rebalancing when membership changes.

## Design

### Group Coordinator

The SC acts as the group coordinator (similar to Kafka's `__consumer_offsets` topic approach, but simpler):

1. **Group membership**: Consumers send heartbeats to the SC with their `group_id`. The SC maintains a member list per group.
2. **Partition assignment**: When membership changes, the coordinator computes a new assignment using a configurable strategy (range, round-robin, sticky).
3. **Rebalance protocol**:
   - Consumer joins → SC triggers rebalance.
   - SC sends `Revoke` to consumers losing partitions.
   - Consumers commit offsets for revoked partitions, then send `Ack`.
   - SC sends new `Assignment` to all consumers.
   - Cooperative: only revoke partitions that are moving, not all.

### API Changes

```rust
// New consumer builder option
let consumer = streamfy.consumer_with_group(ConsumerGroupConfig {
    group_id: "my-group".to_string(),
    topic: "events".to_string(),
    strategy: AssignmentStrategy::CooperativeSticky,
    session_timeout: Duration::from_secs(30),
    heartbeat_interval: Duration::from_secs(10),
}).await?;
```

### Offset Storage

- Leverage existing consumer offset management in SPU (`consumer_offset.rs`).
- Offsets are keyed by `(group_id, topic, partition)`.
- Auto-commit with configurable interval, or manual commit.

### SC Schema Changes

New request/response types in `streamfy-sc-schema`:
- `JoinGroupRequest` / `JoinGroupResponse`
- `HeartbeatRequest` / `HeartbeatResponse`
- `LeaveGroupRequest`
- `SyncGroupRequest` / `SyncGroupResponse` (carries partition assignment)

## Key Files to Modify

- `crates/streamfy-sc/src/controllers/` — new `consumer_groups/` controller module
- `crates/streamfy-sc-schema/src/` — new `consumer_group/` schema module
- `crates/streamfy/src/consumer/` — add `GroupConsumer` alongside existing `PartitionConsumer`
- `crates/streamfy-cli/src/client/consumer/` — `streamfy consumer-group list/describe/delete` commands

## Success Criteria

- 3 consumers with the same `group.id` each receive ~1/3 of partitions from a 9-partition topic.
- When one consumer crashes, its partitions are reassigned to survivors within `session_timeout`.
- When a new consumer joins, partitions are rebalanced with minimal disruption (cooperative sticky).
- Consumer offsets are committed per-group and survive consumer restarts.
