# Task 12: Dead Letter Queue & Error Handling

## Status: Not Started
## Priority: Low | Effort: Small

## Problem

When a SmartModule fails to process a record (returns an error), the record is currently dropped or the entire stream fetch fails. There is no mechanism to:

- Route failed records to a separate topic for investigation.
- Retry failed records with backoff.
- Track error rates per SmartModule.

Similarly, connectors have no standardized error handling — a malformed record can crash the connector.

## Goal

Add a Dead Letter Queue (DLQ) pattern: failed records are routed to a configurable error topic with metadata about the failure.

## Design

### SmartModule DLQ

When a SmartModule returns an error for a record:

1. The record is written to a DLQ topic (default: `{source_topic}.__errors`).
2. The DLQ record includes headers:
   - `streamfy-error-message`: the error string
   - `streamfy-error-smartmodule`: name of the SmartModule that failed
   - `streamfy-source-topic`: original topic
   - `streamfy-source-partition`: original partition
   - `streamfy-source-offset`: original offset
   - `streamfy-error-timestamp`: when the error occurred
3. The original record body is preserved as the DLQ record value.
4. Processing continues with the next record (skip-on-error).

### Configuration

In the SmartModule chain configuration:

```yaml
transforms:
  - uses: my-filter@0.1.0
    error_handling:
      strategy: dead_letter_queue    # or "skip" or "fail"
      dlq_topic: "my-topic.__errors"
      max_retries: 3
      retry_backoff_ms: 1000
```

### Connector DLQ

Connectors gain a similar config:

```yaml
meta:
  name: my-postgres-source
  error_handling:
    dlq_topic: "__connector_errors"
    strategy: dead_letter_queue
```

The connector framework (`streamfy-connector-common`) wraps the `Source`/`Sink` trait to catch panics and errors, routing them to the DLQ.

## Key Files to Modify

- `crates/streamfy-smartengine/src/engine/wasmtime/transforms/` — wrap transform invocations with error routing
- `crates/streamfy-smartengine/src/transformation.rs` — add `ErrorHandlingConfig`
- `crates/streamfy-connector-common/src/lib.rs` — add error handling wrapper
- `crates/streamfy-spu/src/smartengine/chain.rs` — produce to DLQ topic on error

## Success Criteria

- A SmartModule that fails on 5% of records routes those records to the DLQ topic.
- DLQ records contain enough metadata to replay or debug the failure.
- Processing is not blocked by individual record failures.
- DLQ topic can be consumed by standard Streamfy consumers for alerting/reprocessing.
