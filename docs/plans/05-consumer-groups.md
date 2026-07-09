# Task 5: Consumer Groups with Rebalancing

## Status: Not Started
## Priority: High | Effort: Large

## Problem

Streamfy has basic consumer offset management (per-consumer-id offset tracking with TTL, as described in `docs/rfc/offset-management.md`) but no consumer group coordination. Each consumer independently reads from specific partitions. There is no:

- Automatic partition assignment across multiple consumers in a group.
- Rebalancing when consumers join or leave.
- Cooperative or eager rebalancing protocols.
- CLI commands to inspect or manage consumer groups.
- Group-level offset tracking (offsets are per-consumer-id, not per-group).

This means users must manually assign partitions to consumers, which is impractical for horizontally-scaled applications. Consumer groups are **table-stakes** for any streaming platform — Kafka, Pulsar, and Redpanda all have them. Without them, Streamfy cannot support standard patterns like competing consumers, horizontal scaling, or fault-tolerant consumption.

## Goal

Implement consumer groups where multiple consumers with the same `group.id` share partitions of a topic, with automatic rebalancing when membership changes.

---

## Design

### Group State Machine

Each consumer group goes through a well-defined lifecycle:

```
                    ┌──────────┐
         create     │  Empty   │  no members
         group ────►│          │
                    └────┬─────┘
                         │ first member joins
                    ┌────▼─────┐
                    │Preparing │  waiting for all members
                    │Rebalance │  to join within timeout
                    └────┬─────┘
                         │ all members joined or timeout
                    ┌────▼─────┐
                    │Completing│  leader computes assignment,
                    │Rebalance │  members receive partitions
                    └────┬─────┘
                         │ all members synced
                    ┌────▼─────┐
              ┌────►│  Stable  │  normal operation
              │     │          │◄─── heartbeats keep it here
              │     └────┬─────┘
              │          │ member joins/leaves/times out
              │     ┌────▼─────┐
              │     │Preparing │  rebalance triggered
              └─────│Rebalance │
                    └──────────┘
                         │ last member leaves
                    ┌────▼─────┐
                    │  Dead    │  removed after retention
                    └──────────┘
```

States are stored in-memory on the SC with persistence to an internal topic `__consumer_groups` for crash recovery.

### Group Coordinator

The SC acts as the group coordinator. Key responsibilities:

1. **Membership management**: Track which consumers belong to which group via heartbeats.
2. **Leader election**: The first consumer to join a generation becomes the group leader.
3. **Assignment computation**: The leader proposes a partition assignment; the SC distributes it.
4. **Failure detection**: Consumers that miss heartbeats beyond `session_timeout` are evicted.
5. **Generation tracking**: Each rebalance increments a `generation_id`. Stale requests from old generations are rejected.

### Rebalance Protocols

#### Eager (Simple)

1. SC detects membership change → sets group state to `PreparingRebalance`.
2. All consumers receive `Revoke(all partitions)` → stop consuming, commit offsets.
3. All consumers send `JoinGroup` → SC collects member metadata.
4. Leader computes assignment → sends via `SyncGroup`.
5. SC distributes assignments → consumers start consuming new partitions.

**Downside**: Full stop-the-world. All partitions are unowned during rebalance.

#### Cooperative Sticky (Recommended Default)

1. SC detects membership change → computes the *difference* between old and new assignment.
2. Only consumers *losing* partitions receive `Revoke(specific partitions)`.
3. Those consumers commit offsets for revoked partitions, acknowledge.
4. SC sends `Assignment` to all consumers with their new partition set.
5. Consumers that gained partitions start consuming; others continue uninterrupted.

**Benefit**: Partitions that don't move are never interrupted. Rebalance is incremental.

### Assignment Strategies

```rust
pub enum AssignmentStrategy {
    /// Partitions divided into contiguous ranges per consumer.
    /// Consumer 0 gets [0,1,2], Consumer 1 gets [3,4,5], etc.
    Range,

    /// Partitions distributed round-robin across consumers.
    /// Consumer 0 gets [0,3,6], Consumer 1 gets [1,4,7], etc.
    RoundRobin,

    /// Like RoundRobin but minimizes partition movement during rebalance.
    /// Partitions stay with their current owner when possible.
    CooperativeSticky,
}
```

### Protocol: Wire Format

New API keys added to `streamfy-sc-schema`:

#### JoinGroupRequest / Response

```rust
#[derive(Encoder, Decoder)]
pub struct JoinGroupRequest {
    pub group_id: String,
    pub member_id: String,         // empty on first join, assigned by SC
    pub session_timeout_ms: u32,
    pub rebalance_timeout_ms: u32,
    pub protocol_type: String,     // "consumer"
    pub topics: Vec<String>,       // topics this consumer wants to subscribe to
    pub strategy: AssignmentStrategy,
}

#[derive(Encoder, Decoder)]
pub struct JoinGroupResponse {
    pub error_code: ErrorCode,
    pub generation_id: i32,
    pub leader: String,            // member_id of the group leader
    pub member_id: String,         // assigned member_id (on first join)
    pub members: Vec<GroupMember>, // only populated for the leader
}

#[derive(Encoder, Decoder)]
pub struct GroupMember {
    pub member_id: String,
    pub subscriptions: Vec<String>, // topics
    pub metadata: Vec<u8>,          // opaque member metadata
}
```

#### SyncGroupRequest / Response

```rust
#[derive(Encoder, Decoder)]
pub struct SyncGroupRequest {
    pub group_id: String,
    pub generation_id: i32,
    pub member_id: String,
    /// Only the leader sends assignments; followers send empty vec
    pub assignments: Vec<MemberAssignment>,
}

#[derive(Encoder, Decoder)]
pub struct MemberAssignment {
    pub member_id: String,
    pub partitions: Vec<TopicPartition>,
}

#[derive(Encoder, Decoder)]
pub struct TopicPartition {
    pub topic: String,
    pub partitions: Vec<i32>,
}

#[derive(Encoder, Decoder)]
pub struct SyncGroupResponse {
    pub error_code: ErrorCode,
    pub assignment: Vec<TopicPartition>, // this member's assigned partitions
}
```

#### HeartbeatRequest / Response

```rust
#[derive(Encoder, Decoder)]
pub struct HeartbeatRequest {
    pub group_id: String,
    pub generation_id: i32,
    pub member_id: String,
}

#[derive(Encoder, Decoder)]
pub struct HeartbeatResponse {
    pub error_code: ErrorCode, // REBALANCE_IN_PROGRESS triggers rejoin
}
```

#### LeaveGroupRequest

```rust
#[derive(Encoder, Decoder)]
pub struct LeaveGroupRequest {
    pub group_id: String,
    pub members: Vec<LeavingMember>,
}

#[derive(Encoder, Decoder)]
pub struct LeavingMember {
    pub member_id: String,
    pub reason: Option<String>,
}
```

#### OffsetCommit / OffsetFetch (Group-aware)

```rust
#[derive(Encoder, Decoder)]
pub struct GroupOffsetCommitRequest {
    pub group_id: String,
    pub generation_id: i32,
    pub member_id: String,
    pub offsets: Vec<TopicPartitionOffset>,
}

#[derive(Encoder, Decoder)]
pub struct TopicPartitionOffset {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub metadata: Option<String>,
}

#[derive(Encoder, Decoder)]
pub struct GroupOffsetFetchRequest {
    pub group_id: String,
    pub topics: Vec<TopicPartitions>, // empty = all topics
}
```

### Offset Storage

- Group offsets are stored on SPUs in a compacted internal topic `__consumer_offsets`.
- Key: `(group_id, topic, partition)` — serialized as a fixed-format binary key.
- Value: `(offset, metadata, commit_timestamp)`.
- The SC routes offset commits/fetches to the SPU that owns the relevant partition of `__consumer_offsets` (partitioned by hash of `group_id`).
- Leverage existing consumer offset infrastructure in `crates/streamfy-spu/src/services/public/offset_update.rs`, extending it with group semantics.

### Client SDK API

```rust
use streamfy::consumer::{ConsumerGroupConfig, GroupConsumer, AssignmentStrategy};

// Create a group consumer
let consumer: GroupConsumer = streamfy
    .group_consumer(ConsumerGroupConfig {
        group_id: "order-processors",
        topics: vec!["orders"],
        strategy: AssignmentStrategy::CooperativeSticky,
        session_timeout: Duration::from_secs(30),
        heartbeat_interval: Duration::from_secs(10),
        auto_commit: true,
        auto_commit_interval: Duration::from_secs(5),
        offset_reset: OffsetReset::Latest, // or Earliest
    })
    .await?;

// Stream records (partitions auto-assigned)
while let Some(record) = consumer.next().await {
    process(record?).await;
    // offsets auto-committed every 5s
}

// Or manual commit
let consumer = streamfy
    .group_consumer(ConsumerGroupConfig {
        auto_commit: false,
        ..config
    })
    .await?;

while let Some(record) = consumer.next().await {
    let record = record?;
    process(&record).await;
    consumer.commit(&record).await?; // sync commit
}

// Rebalance listener
consumer.on_partitions_revoked(|partitions| async {
    // flush state, commit offsets for revoked partitions
    println!("Revoked: {:?}", partitions);
});

consumer.on_partitions_assigned(|partitions| async {
    // initialize state for new partitions
    println!("Assigned: {:?}", partitions);
});
```

### CLI Commands

```bash
# List all consumer groups
streamfy consumer-group list
GROUP ID            TOPIC      MEMBERS  STATE     LAG
order-processors    orders     3        Stable    234
log-shippers        logs       2        Stable    12
analytics           events     0        Empty     —

# Describe a consumer group
streamfy consumer-group describe order-processors
Group: order-processors
State: Stable
Strategy: CooperativeSticky
Generation: 7
Members: 3

MEMBER ID          CLIENT HOST     PARTITIONS     CURRENT OFFSET  LOG END OFFSET  LAG
member-a1b2c3      10.0.1.5       [0, 1, 2]      1042            1050            8
member-d4e5f6      10.0.1.6       [3, 4, 5]      987             1050            63
member-g7h8i9      10.0.1.7       [6, 7, 8]      1047            1050            3

# Reset offsets (group must be empty/stopped)
streamfy consumer-group reset-offsets order-processors \
    --topic orders --to-earliest
streamfy consumer-group reset-offsets order-processors \
    --topic orders --to-offset 500
streamfy consumer-group reset-offsets order-processors \
    --topic orders --to-datetime "2026-06-01T00:00:00Z"
streamfy consumer-group reset-offsets order-processors \
    --topic orders --shift-by -100

# Delete a consumer group
streamfy consumer-group delete order-processors

# Delete offsets for a specific topic within a group
streamfy consumer-group delete-offsets order-processors --topic orders
```

### Failure Scenarios

| Scenario | Behavior |
|----------|----------|
| Consumer crashes (no graceful leave) | Heartbeat timeout after `session_timeout` → SC evicts member → rebalance |
| Consumer gracefully shuts down | Sends `LeaveGroup` → immediate rebalance (no timeout wait) |
| SC restarts | Group state recovered from `__consumer_offsets` topic → consumers rejoin on next heartbeat failure |
| Network partition (consumer ↔ SC) | Consumer can't heartbeat → SC evicts → consumer detects via failed heartbeat response → rejoins when network recovers |
| Slow consumer (processing > `max.poll.interval`) | Configurable: consumer self-evicts if `next()` isn't called within `max_poll_interval_ms` |
| New partitions added to topic | Detected on next metadata refresh → triggers rebalance to assign new partitions |
| Consumer subscribes to multiple topics | All subscribed topics' partitions are assigned as a unit |

---

## Key Files to Modify

### New Files

- `crates/streamfy-sc/src/controllers/consumer_groups/mod.rs` — group coordinator controller
- `crates/streamfy-sc/src/controllers/consumer_groups/coordinator.rs` — state machine, membership, assignment
- `crates/streamfy-sc/src/controllers/consumer_groups/assignment.rs` — Range, RoundRobin, CooperativeSticky strategies
- `crates/streamfy-sc/src/controllers/consumer_groups/state.rs` — group state persistence
- `crates/streamfy-sc-schema/src/consumer_group/mod.rs` — protocol types
- `crates/streamfy-sc-schema/src/consumer_group/request.rs` — JoinGroup, SyncGroup, Heartbeat, LeaveGroup
- `crates/streamfy-sc-schema/src/consumer_group/response.rs` — responses
- `crates/streamfy/src/consumer/group.rs` — `GroupConsumer` client implementation
- `crates/streamfy-cli/src/client/consumer_group/mod.rs` — CLI subcommands

### Modified Files

- `crates/streamfy-sc/src/controllers/mod.rs` — register consumer_groups controller
- `crates/streamfy-sc-schema/src/lib.rs` — export consumer_group module
- `crates/streamfy-sc/src/services/public_api/mod.rs` — route new API keys
- `crates/streamfy-sc/src/services/public_api/api_version.rs` — register new API versions
- `crates/streamfy/src/lib.rs` — expose `group_consumer()` builder
- `crates/streamfy-cli/src/client/mod.rs` — add `consumer-group` subcommand

---

## Phases

### Phase 1: Eager Rebalance (3 weeks)
- Group coordinator in SC with state machine (Empty → PreparingRebalance → Stable → Dead).
- JoinGroup / SyncGroup / Heartbeat / LeaveGroup protocol.
- Range and RoundRobin assignment strategies.
- `GroupConsumer` in client SDK with auto-commit.
- Single-topic subscription only.
- `streamfy consumer-group list` and `describe` CLI commands.

### Phase 2: Cooperative Sticky (2 weeks)
- CooperativeSticky assignment strategy.
- Incremental rebalance (only revoke moving partitions).
- `on_partitions_revoked` / `on_partitions_assigned` callbacks.
- Multi-topic subscription.

### Phase 3: Offset Management & CLI (1 week)
- Group offset storage in `__consumer_offsets` internal topic.
- `reset-offsets` CLI command (to-earliest, to-latest, to-offset, to-datetime, shift-by).
- `delete` and `delete-offsets` CLI commands.
- Manual commit mode in client SDK.

### Phase 4: Hardening (1 week)
- SC crash recovery (restore group state from `__consumer_offsets`).
- Fencing stale members via generation_id.
- `max_poll_interval_ms` enforcement.
- Integration tests: 3-consumer rebalance, crash recovery, rolling restart.

---

## Dependencies

- **Task #3 (Log Compaction)**: Required for `__consumer_offsets` topic to not grow unbounded. Without compaction, the internal topic retains every offset commit forever. Can work around this initially with TTL-based retention.
- **Task #13 (MCP Server)**: Should expose `consumer_group_list`, `consumer_group_describe`, `consumer_group_reset_offsets` tools.
- **Task #14 (Web UI)**: Should add a Consumer Groups page showing group state, members, lag.
- **Kafka Compatibility (proxy)**: Consumer groups are required for Phase 2 of the Kafka protocol proxy (JoinGroup, SyncGroup, Heartbeat, OffsetCommit, OffsetFetch).

## Success Criteria

- 3 consumers with the same `group.id` each receive ~1/3 of partitions from a 9-partition topic.
- When one consumer crashes, its partitions are reassigned to survivors within `session_timeout`.
- When a new consumer joins, partitions are rebalanced with minimal disruption (cooperative sticky).
- Consumer offsets are committed per-group and survive consumer restarts.
- `streamfy consumer-group describe` shows per-member partition assignments and lag.
- SC restart does not lose group state.
- Rebalance completes in < 5 seconds for groups with up to 50 members.
