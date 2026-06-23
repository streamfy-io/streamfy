# Kafka Protocol Compatibility — Should We?

## Executive Summary

**Recommendation: Yes, but as a protocol translation proxy — not by replacing Streamfy's internals.**

A Kafka-compatible API would dramatically lower the adoption barrier. However, rewriting Streamfy to use the Kafka wire protocol internally would be a multi-year effort that defeats the purpose of forking. The right approach is a lightweight **Kafka protocol proxy** that translates Kafka clients' requests into Streamfy operations.

---

## Current State

Streamfy has **zero Kafka protocol compatibility**:
- Custom binary protocol (`streamfy-protocol`) with its own Encoder/Decoder derive macros.
- Custom request/response types in `streamfy-spu-schema` and `streamfy-sc-schema`.
- No Kafka wire format parsing anywhere in the codebase.
- A misleading comment in `streamfy-sc-schema/src/versions.rs` says "SC supports Kafka as well as Streamfy specific APIs" — this is inherited from Fluvio and is **not true**.
- Some test helpers are named `TestKafkaApiEnum` — leftover naming, not actual Kafka support.

## What Kafka Compatibility Means

Kafka clients (librdkafka, Java client, Python confluent-kafka, etc.) speak the [Kafka wire protocol](https://kafka.apache.org/protocol.html):
- TCP connection with a binary framed protocol.
- ~60 API keys (Produce, Fetch, Metadata, FindCoordinator, JoinGroup, etc.).
- Multiple versions per API key (backwards compatibility).

"Kafka compatible" means: **a Kafka client can connect to Streamfy and produce/consume without code changes.**

## Options Analysis

### Option A: Kafka Protocol Proxy (Recommended)

A standalone service (`streamfy-kafka-proxy`) that:
1. Listens on port 9092 (Kafka default).
2. Parses Kafka wire protocol requests.
3. Translates them into Streamfy client SDK calls.
4. Returns Kafka wire protocol responses.

**Minimum viable API keys to support:**

| API Key | Name | Required For |
|---------|------|-------------|
| 0 | Produce | Producers |
| 1 | Fetch | Consumers |
| 2 | ListOffsets | Consumers |
| 3 | Metadata | All clients (topic discovery, broker list) |
| 8 | OffsetCommit | Consumer groups |
| 9 | OffsetFetch | Consumer groups |
| 10 | FindCoordinator | Consumer groups |
| 11 | JoinGroup | Consumer groups |
| 12 | Heartbeat | Consumer groups |
| 13 | LeaveGroup | Consumer groups |
| 14 | SyncGroup | Consumer groups |
| 18 | ApiVersions | All clients (version negotiation) |

**Pros:**
- Streamfy internals stay clean — no protocol pollution.
- Can iterate quickly — proxy is a separate crate/binary.
- Existing Kafka ecosystem tools (Kafka Connect, ksqlDB, Kafka UI) work out of the box.
- Users can migrate incrementally: Kafka clients talk to proxy, native Streamfy clients talk directly.

**Cons:**
- Extra hop: Kafka client → proxy → SPU (adds ~1ms latency).
- Must keep proxy in sync with Streamfy API changes.
- Consumer group support requires Task #5 (Consumer Groups) first.

**Effort: Large (3-4 months for a team of 2)**

### Option B: Native Kafka Protocol in SPU

Modify SPU to accept both Streamfy and Kafka wire protocols on different ports.

**Pros:**
- No extra hop — lowest latency.
- Single binary deployment.

**Cons:**
- Massive complexity in SPU — two protocol parsers, two request pipelines.
- Kafka protocol has ~60 API keys with multiple versions each — huge surface area.
- Tight coupling between Kafka protocol evolution and Streamfy releases.
- Risk of introducing bugs in the core data path.

**Effort: XL (6-12 months). Not recommended.**

### Option C: Don't Do It

Focus on Streamfy's native protocol and SDK. Provide connectors for Kafka interop (Kafka source/sink connectors).

**Pros:**
- Zero effort. Ship features instead.
- Kafka source/sink connector (Task #6) covers the interop use case.

**Cons:**
- Adoption barrier remains high. Users must rewrite all client code.
- Can't use Kafka ecosystem tools (Schema Registry, Kafka UI, etc.).
- Positioning: "yet another streaming system with its own protocol" is a hard sell.

**This is viable only if Streamfy targets a niche (embedded, edge, IoT) where Kafka compatibility doesn't matter.**

---

## Recommendation: Option A (Kafka Protocol Proxy)

### Why

1. **Adoption is the #1 problem for any new streaming system.** Kafka has 80%+ market share. If Kafka clients can connect to Streamfy, adoption friction drops to near zero.

2. **The proxy approach is proven.** Redpanda, WarpStream, and AutoMQ all started as Kafka-compatible systems. Even Pulsar added a Kafka protocol handler (`KoP`). The proxy pattern specifically is used by services like AWS MSK Serverless.

3. **It's decoupled.** The proxy is a separate crate. If it doesn't work out or becomes unmaintainable, it can be dropped without affecting core Streamfy.

4. **It unlocks the Kafka ecosystem.** Schema Registry, Kafka Connect, ksqlDB, Conduktor, Kafka UI — all work through the wire protocol. Supporting it opens all of these.

### Implementation Plan

```
Phase 1 (4 weeks): Produce + Fetch + Metadata + ApiVersions
  → Kafka producers and simple consumers work.

Phase 2 (4 weeks): Consumer Groups (requires Task #5)
  → Kafka consumer groups with rebalancing work.

Phase 3 (2 weeks): Schema Registry pass-through
  → Proxy forwards Schema Registry requests to a Confluent-compatible registry.

Phase 4 (2 weeks): Admin APIs (CreateTopics, DeleteTopics, DescribeConfigs)
  → Kafka admin tools work.
```

### Architecture

```
                    ┌─────────────────────────────┐
                    │    streamfy-kafka-proxy      │
Kafka Client ──────►  :9092                       │
                    │  Kafka Wire Protocol Parser  │
                    │         ↓                    │
                    │  Streamfy Client SDK calls   │
                    │         ↓                    │
                    └─────────┬───────────────────┘
                              │
                    ┌─────────▼───────────────────┐
                    │        Streamfy SC + SPUs     │
                    │    (native protocol, unchanged)│
                    └─────────────────────────────┘
```

### Key Dependencies

- `kafka-protocol` crate (Apache 2.0) — Rust implementation of Kafka wire protocol parsing. Saves months of work.
- Task #5 (Consumer Groups) — required for Phase 2.

### Crate Structure

```
crates/
  streamfy-kafka-proxy/
    src/
      main.rs              # binary entry point
      server.rs            # TCP listener, connection handler
      protocol/
        mod.rs             # Kafka request/response routing
        produce.rs         # Kafka Produce → Streamfy produce
        fetch.rs           # Kafka Fetch → Streamfy consume
        metadata.rs        # Kafka Metadata → Streamfy topic list
        consumer_group.rs  # Kafka group protocol → Streamfy groups
      translator.rs        # mapping layer (topic names, offsets, errors)
```

## Verdict

| Factor | Score |
|--------|-------|
| **Impact on adoption** | Very High |
| **Engineering effort** | Large but bounded (proxy approach) |
| **Risk** | Medium (proxy can be dropped if it fails) |
| **Dependency on other tasks** | Consumer Groups (#5) for full support |
| **Worth it?** | **Yes** — it's the single highest-leverage thing for adoption |

Build it as a proxy. Start with Produce + Fetch (Phase 1) to prove the concept in 4 weeks. If that works, complete the rest.
