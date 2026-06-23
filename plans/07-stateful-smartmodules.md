# Task 7: Windowed Processing & Stateful SmartModules

## Status: Not Started
## Priority: Medium | Effort: Large

## Problem

SmartModules currently support stateless transforms (filter, map, filter_map, array_map) and a basic `aggregate` that accumulates state across records. There is no:

- **Windowing**: tumbling, sliding, or session windows for time-based aggregations.
- **Temporal operators**: event-time processing, watermarks, late arrival handling.
- **State backend**: aggregate state is in-memory only, lost on restart.
- **Joins**: no stream-stream or stream-table joins.

The `rfc/materialize_view.md` describes an ambitious vision (columnar topics, view joins, SQL-like queries) but none of it is implemented.

## Goal

Extend SmartModules with windowed processing, persistent state, and basic join support — keeping the WASM-based architecture.

## Design

### Phase 1: Windowed Aggregations

Add window-aware aggregate functions:

```rust
#[smartmodule(aggregate)]
fn aggregate(accumulator: &[u8], current: &Record, window: &WindowContext) -> Result<Vec<u8>> {
    // window.type: Tumbling(Duration) | Sliding(size, slide) | Session(gap)
    // window.start, window.end, window.is_closing
    // ...
}
```

Window types:
- **Tumbling**: fixed-size, non-overlapping (e.g., 1-minute counts).
- **Sliding**: overlapping windows (e.g., 5-minute window sliding every 1 minute).
- **Session**: gap-based (e.g., group events with < 30s between them).

### Phase 2: Persistent State Backend

- State is currently the `accumulator: &[u8]` passed between invocations — purely in-memory.
- Add a `StateStore` trait backed by `streamfy-kv-storage` (which already exists but is underused).
- State is checkpointed to a compacted internal topic `__smartmodule_state` for recovery.
- On SmartModule restart, state is restored from the checkpoint.

### Phase 3: Stream-Table Joins

- A **table** is a compacted topic (depends on Task #3: Log Compaction).
- SmartModules can look up keys from a table topic during processing:

```rust
#[smartmodule(map)]
fn enrich(record: &Record, ctx: &SmartModuleContext) -> Result<(Option<RecordData>, RecordData)> {
    let user_id = extract_user_id(record)?;
    let user_profile = ctx.table_lookup("users", user_id)?;
    // join record with user_profile
}
```

## Key Files to Modify

- `crates/streamfy-smartengine/src/engine/wasmtime/transforms/aggregate.rs` — add window context
- `crates/streamfy-smartengine/src/engine/` — new `state.rs` module for state backend
- `crates/streamfy-smartmodule/src/lib.rs` — add `WindowContext` and `SmartModuleContext` types
- `crates/streamfy-kv-storage/` — use as state backend
- `crates/streamfy-spu/src/smartengine/` — integrate state checkpointing

## Dependencies

- Task #3 (Log Compaction) is a prerequisite for stream-table joins.

## Success Criteria

- A tumbling-window SmartModule can count events per 1-minute window and emit window results.
- State survives SPU restart via checkpoint recovery.
- Stream-table join can enrich a stream with data from a compacted topic.
