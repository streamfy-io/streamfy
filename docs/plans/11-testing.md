# Task 11: Chaos Testing & CI Hardening

## Status: Not Started
## Priority: Medium | Effort: Medium

## Problem

Testing is three-tiered (unit, integration, bats CLI tests) but has gaps:
- No coverage metrics tracked in CI.
- No property-based or fuzz testing (critical for a protocol/storage system).
- No chaos/fault-injection testing (network partitions, disk failures, process crashes).
- Benchmarks exist (`streamfy-benchmark`) but don't run in CI — no regression detection.
- ~50 TODO/FIXME items in the codebase indicate untested edge cases.

## Goal

A CI pipeline that catches correctness regressions, performance regressions, and validates behavior under failure conditions.

## Scope

### Phase 1: Coverage & Property Testing

- Add `cargo-llvm-cov` to CI. Target: >70% line coverage on core crates (`streamfy-storage`, `streamfy-protocol`, `streamfy-spu-schema`).
- Add `proptest` property tests for:
  - Protocol encode/decode roundtrip (any valid message encodes then decodes to the same value).
  - Storage segment write/read roundtrip.
  - Batch compression/decompression roundtrip.
- Add `cargo-fuzz` targets for protocol deserialization (security-critical path).

### Phase 2: Chaos Testing

Use `toxiproxy` or a custom Rust harness to simulate:
- **Network partition** between SC and SPU → SPU should continue serving reads, buffer writes.
- **SPU crash** during produce → leader failover, no data loss for committed records.
- **Slow disk** (throttled I/O) → graceful degradation, not hangs.
- **SC crash and recovery** → SPU reconnects, metadata is consistent.

Test harness:
```
tests/chaos/
  network_partition.rs
  spu_crash_recovery.rs
  slow_disk.rs
  sc_failover.rs
```

### Phase 3: Benchmark CI

- Run `streamfy-benchmark` nightly on a consistent instance type.
- Track: produce throughput (MB/s), consume throughput (MB/s), p99 latency.
- Alert if any metric regresses >10% from the rolling baseline.
- Store results in a simple JSON file committed to a `benchmarks` branch.

## Key Files to Modify

- `.github/workflows/ci.yml` — add coverage step, property test step
- `crates/streamfy-protocol/` — add `proptest` dev-dependency and roundtrip tests
- `crates/streamfy-storage/` — add property tests for segment operations
- New: `tests/chaos/` directory with fault-injection test binaries
- `.github/workflows/benchmarks.yml` — nightly benchmark run

## Dependencies

- `proptest = "1.6"`, `cargo-llvm-cov`, `cargo-fuzz`
- `toxiproxy` (or custom TCP proxy for fault injection)

## Success Criteria

- CI reports coverage percentage on every PR. Merges require no coverage decrease.
- Property tests run on every CI build (< 30s additional time).
- Chaos tests run nightly and pass (SPU recovers from all failure scenarios).
- Performance dashboard shows throughput/latency trends over time.
