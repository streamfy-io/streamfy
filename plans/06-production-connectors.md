# Task 6: Production Connectors (Postgres, S3, HTTP, MQTT)

## Status: Not Started
## Priority: Medium | Effort: Medium

## Problem

The repo only contains test connectors (`json-test-connector`, `sink-test-connector`) and a `cargo_template` for building new ones. There are no production-ready, built-in connectors. The connector framework (`streamfy-connector-common`) is solid — it has `Source` and `Sink` traits, config parsing, monitoring, and a CDK — but the ecosystem is empty.

Without connectors, users must write custom integration code to get data in/out of Streamfy.

## Goal

Ship 4 production-quality connectors that cover the most common integration patterns.

## Connectors

### 1. PostgreSQL CDC Source

- Use logical replication (pgoutput plugin) to stream WAL changes into Streamfy topics.
- Output format: JSON with `{ "op": "INSERT|UPDATE|DELETE", "table": "...", "before": {...}, "after": {...} }`.
- Track LSN offsets for exactly-once delivery.
- Config: connection string, publication name, slot name, tables filter.

### 2. S3 Sink

- Batch records from a topic and write to S3 as Parquet, JSON, or CSV files.
- Partitioning: by time (hourly/daily) and/or record key.
- Rotation: by size (e.g., 256MB) or time (e.g., every hour).
- Config: bucket, prefix, format, rotation policy, AWS credentials.

### 3. HTTP Source/Sink

- **Source**: Poll an HTTP endpoint at intervals, or receive webhooks via an embedded HTTP server.
- **Sink**: POST records to an HTTP endpoint with configurable batching, retries, and auth (Bearer, Basic, API key).
- Support for JSON body transformation via SmartModules.

### 4. MQTT Source

- Subscribe to MQTT topics (v3.1.1 and v5) and stream messages into Streamfy topics.
- Map MQTT topic hierarchy to Streamfy topic/key (e.g., `sensors/+/temperature` → topic `sensors`, key = device_id).
- QoS 1 (at-least-once) with offset tracking.
- Config: broker URL, client ID, subscriptions, TLS.

## Structure

Each connector lives in its own crate under `connector/`:

```
connector/
  postgres-source/
  s3-sink/
  http-connector/     # both source and sink
  mqtt-source/
```

Each connector ships as a standalone binary (via CDK) and as a Docker image.

## Success Criteria

- Each connector can be deployed via `cdk deploy` with a YAML config.
- Postgres CDC captures INSERT/UPDATE/DELETE within seconds.
- S3 sink produces valid Parquet files queryable by Athena/DuckDB.
- HTTP sink delivers with retries and backoff on 5xx errors.
- MQTT source handles reconnection and QoS 1 acknowledgment.
