# Task 10: Exactly-Once Semantics & Transactions

## Status: Not Started
## Priority: Medium | Effort: XL

## Problem

Streamfy currently provides at-least-once delivery:
- Producers retry on failure, which can cause duplicates.
- Consumers track offsets, but a crash between processing and offset commit causes reprocessing.
- There is no transactional produce (atomic writes across multiple partitions).

For use cases like financial transactions, inventory updates, and exactly-once stream processing, this is insufficient.

## Goal

Provide exactly-once semantics (EOS) through idempotent producers and transactional produce/consume.

## Design

### Phase 1: Idempotent Producer

- Assign each producer a `producer_id` (PID) and a monotonically increasing `sequence_number` per partition.
- SPU tracks the last sequence number per (PID, partition). Duplicate sequence numbers are silently dropped.
- On producer restart, request a new PID from SC, or resume with persisted PID.

Protocol changes:
- `ProduceRequest` gains `producer_id: u64` and `first_sequence: u64` fields.
- `ProduceResponse` gains `duplicate: bool` per partition.
- SPU maintains `ProducerStateManager` — a map of `(PID, Partition) → last_sequence`.

### Phase 2: Transactions

- Add `TransactionCoordinator` to SC (or as a dedicated service).
- Transaction lifecycle:
  1. `InitTransaction(transactional_id)` → allocates PID, epoch.
  2. `BeginTransaction()` → starts buffering.
  3. `Produce(...)` → records are written with transaction marker (uncommitted).
  4. `CommitTransaction()` → transaction marker written, records become visible.
  5. `AbortTransaction()` → abort marker written, records are discarded on read.
- Consumers with `isolation.level=read_committed` only see committed records.

### Phase 3: Consume-Transform-Produce (EOS)

- Combine consumer offset commit + producer writes in a single transaction.
- This enables exactly-once stream processing: read from input topic, process, write to output topic, commit input offset — all atomically.

## Key Files to Modify

- `crates/streamfy-spu-schema/src/produce/request.rs` — add PID/sequence fields
- `crates/streamfy-spu/src/services/public/produce_handler.rs` — dedup logic
- `crates/streamfy-sc/src/controllers/` — new `transactions/` controller
- `crates/streamfy/src/producer/mod.rs` — idempotent producer mode
- `crates/streamfy-storage/src/replica.rs` — transaction markers in log

## Risks

- Significant protocol complexity. Transaction state must survive SC/SPU restarts.
- Performance impact: sequence number tracking adds overhead per produce request.
- Zombie fencing: old producers with same transactional_id must be fenced via epoch.

## Success Criteria

- Producer with `enable.idempotence=true` produces zero duplicates under retry.
- Transactional produce across 3 partitions is atomic (all or nothing).
- `read_committed` consumers never see uncommitted or aborted records.
