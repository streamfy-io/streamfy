# Task 1: Prometheus & OpenTelemetry Observability

## Status: Not Started
## Priority: High | Effort: Medium

## Problem

Streamfy currently has only in-memory atomic counters for metrics (`SpuMetrics`, `ClientMetrics`) with no way to export them. Tracing uses `tracing` crate for log output but there is no structured trace export, no distributed tracing, and no `/metrics` endpoint anywhere. Connector monitoring is limited to a Unix socket at `/tmp/streamfy-connector.sock`.

This makes it nearly impossible to operate Streamfy in production — you can't alert on lag, throughput, error rates, or resource usage without manual log parsing.

## Goal

Every Streamfy component (SC, SPU, connectors) exposes a `/metrics` HTTP endpoint in Prometheus format. Optionally, traces are exported via OpenTelemetry to any OTLP-compatible backend.

## Scope

### Phase 1: Prometheus Metrics (SC + SPU)

- Add `metrics` + `metrics-exporter-prometheus` crates as dependencies.
- Start an HTTP server on a configurable port (default `:9090/metrics`) in both SC and SPU.
- Export existing counters:
  - **SPU**: `streamfy_spu_records_in_total`, `streamfy_spu_records_out_total`, `streamfy_spu_bytes_in_total`, `streamfy_spu_bytes_out_total`, partition lag, replication lag, active connections.
  - **SC**: topic count, partition count, SPU count, SPU online/offline, controller reconciliation latency.
- Add histograms for produce/fetch latency (p50/p99/p999).

### Phase 2: OpenTelemetry Traces

- Add `tracing-opentelemetry` + `opentelemetry-otlp` behind a feature flag `otel`.
- Propagate trace context through produce/fetch request headers (add a metadata field to the protocol).
- Export spans for key operations: produce batch, fetch stream, SmartModule chain invocation, replication sync.

### Phase 3: Connector & Client Metrics

- Replace Unix socket monitoring in connectors with a Prometheus HTTP endpoint.
- Add metrics to the Rust client SDK (producer/consumer record rates, error rates, batch sizes).

## Key Files to Modify

- `crates/streamfy-spu/src/start.rs` — start metrics HTTP server alongside SPU
- `crates/streamfy-spu/src/core/metrics.rs` — register Prometheus counters
- `crates/streamfy-sc/src/start.rs` — start metrics HTTP server alongside SC
- `crates/streamfy-connector-common/src/monitoring.rs` — replace Unix socket with HTTP
- `crates/streamfy/src/producer/mod.rs` and `consumer/mod.rs` — client-side metrics

## Dependencies

- `metrics = "0.24"`
- `metrics-exporter-prometheus = "0.16"`
- `tracing-opentelemetry = "0.28"` (feature-gated)
- `opentelemetry-otlp = "0.27"` (feature-gated)

## Success Criteria

- `curl localhost:9090/metrics` on a running SPU returns valid Prometheus text format.
- Grafana can visualize throughput, latency, and consumer lag out of the box.
- No performance regression (< 1% overhead on produce/consume throughput).
