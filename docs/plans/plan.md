# Streamfy Improvement Plan

> Streamfy is a fork of [Fluvio](https://github.com/infinyon/fluvio) — a distributed, composable data streaming platform written in Rust. This document outlines the roadmap to evolve Streamfy into a production-grade streaming system that stands on its own.

## Overview

The tasks below are ordered by impact and urgency. Each has a dedicated file in this directory with implementation details.

| # | Task | Priority | Effort | File |
|---|------|----------|--------|------|
| 1 | [Prometheus & OpenTelemetry Observability](./01-observability.md) | High | Medium | `01-observability.md` |
| 2 | [Wire SPU Authentication & Add Encryption at Rest](./02-security.md) | High | Medium | `02-security.md` |
| 3 | [Log Compaction](./03-log-compaction.md) → **[A](./15-arch-log-compaction.md)** | High | Large | `03` / `15` (in progress) |
| 4 | [Tiered Storage (S3/Cloud Offloading)](./04-tiered-storage.md) | Medium | Large | `04-tiered-storage.md` |
| 5 | [Consumer Groups with Rebalancing](./05-consumer-groups.md) | High | Large | `05-consumer-groups.md` |
| 6 | [Production Connectors (Postgres, S3, HTTP, MQTT)](./06-production-connectors.md) | Medium | Medium | `06-production-connectors.md` |
| 7 | [Windowed Processing & Stateful SmartModules](./07-stateful-smartmodules.md) | Medium | Large | `07-stateful-smartmodules.md` |
| 8 | [HTTP Admin API & Health Checks](./08-http-admin-api.md) | Medium | Small | `08-http-admin-api.md` |
| 9 | [Multi-Language SmartModule SDK (Python, Go, JS)](./09-multi-lang-smartmodules.md) | Low | Large | `09-multi-lang-smartmodules.md` |
| 10 | [Exactly-Once Semantics & Transactions](./10-exactly-once.md) | Medium | XL | `10-exactly-once.md` |
| 11 | [Chaos Testing & CI Hardening](./11-testing.md) | Medium | Medium | `11-testing.md` |
| 12 | [Dead Letter Queue & Error Handling](./12-dead-letter-queue.md) | Low | Small | `12-dead-letter-queue.md` |
| 13 | [MCP Server (Model Context Protocol)](./13-mcp-server.md) | High | Medium | `13-mcp-server.md` |
| 14 | [Web UI Dashboard](./14-web-ui.md) | Medium | Large | `14-web-ui.md` |

## Architecture proposals (2026 review)

| ID | Change | Priority | File |
|----|--------|----------|------|
| — | [Architecture Review & Ranking](./architecture-review.md) | — | `architecture-review.md` |
| A | [Log Compaction (storage primitive)](./15-arch-log-compaction.md) | Critical | `15-arch-log-compaction.md` |
| B | [Consumer Groups (coordinator)](./16-arch-consumer-groups.md) | Critical | `16-arch-consumer-groups.md` |
| C | [SPU Trust Boundary (data-plane authz)](./17-arch-spu-authz.md) | Critical | `17-arch-spu-authz.md` |

**Selected for implementation:** **A — Log Compaction** (foundational hard change; not the easy path).

## Separate Analysis

| Topic | File |
|-------|------|
| [Kafka Protocol Compatibility — Should We?](./kafka-compatibility.md) | `kafka-compatibility.md` |

## Guiding Principles

1. **Don't break what works.** Streamfy already compiles, passes clippy, and has a working produce/consume flow. Every change must preserve that.
2. **Incremental delivery.** Each task should ship independently. No multi-month branches.
3. **Rust-native first.** Leverage Rust's strengths (zero-copy, async, WASM) instead of reimplementing patterns from JVM-based systems.
4. **Measure before optimizing.** Observability (task #1) should land first so everything after has data to back it up.
