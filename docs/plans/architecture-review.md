# Streamfy Architecture Review

> Date: 2026-07-05 · Scope: full monorepo (SC, SPU, storage, protocol, SmartModules, auth, CLI)

## Current Architecture (as implemented)

```
┌─────────────┐     admin/API      ┌──────────────────┐
│ streamfy CLI│───────────────────►│  SC (control)    │
│ / SDK       │                    │  topics, SPUs,   │
└──────┬──────┘                    │  partitions, SM  │
       │ produce/fetch             └────────┬─────────┘
       │                                    │ control plane
       ▼                                    ▼
┌─────────────┐  replication   ┌──────────────────────┐
│ SPU public  │◄──────────────►│ SPU followers / peers│
│  produce,   │                └──────────────────────┘
│  streamfetch│
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────────────────────┐
│ streamfy-storage: FileReplica                       │
│  active MutableSegment + sealed ReadSegments        │
│  cleanup: size + TTL only (delete whole segments)   │
└─────────────────────────────────────────────────────┘
```

**Strengths**

- Real leader/follower replication with HW/LEO
- Clean SC/SPU split and CRD-like admin model
- WASM SmartModules (filter/map/aggregate/lookback) on produce and consume
- Per-consumer-id offset storage (not groups)
- SC mTLS + RBAC for admin path

**Structural holes vs Kafka/Redpanda production bar**

| Capability | Status | Blocking for |
|------------|--------|--------------|
| Key-based log compaction | **Missing** | offset topics, KTables, SM state, CDC |
| Consumer groups / rebalance | **Missing** | horizontal consumers, Kafka proxy phase 2 |
| SPU data-plane authz | **Root only** | multi-tenant production |
| Tiered / remote storage | **Missing** | cheap long retention |
| Transactions / EOS | Field only | finance-grade exactly-once |
| Rack-aware placement | Metadata only | cross-AZ HA |
| Durable SM state | In-memory | restart-safe stream processing |

---

## Architecture change proposals (ranked)

| ID | Change | Impact | Difficulty | Why this rank |
|----|--------|--------|------------|---------------|
| **A** | [Log compaction](./15-arch-log-compaction.md) | Critical enabler | Large | Storage primitive; unlocks groups, state, CDC |
| **B** | [Consumer groups](./16-arch-consumer-groups.md) | Critical product | Large | Highest app-dev gap; needs durable offsets |
| **C** | [SPU trust boundary](./17-arch-spu-authz.md) | Critical security | Med–Large | Admin secure, data plane open today |
| **D** | Tiered storage (see `04`) | High cost/scale | Large | After storage trait cleanup |
| **E** | EOS / transactions (see `10`) | High semantics | XL | After A–C stable |

### Dependency graph

```
A Log compaction ──┬──► B Consumer groups ──► Kafka proxy (groups)
                   ├──► Stateful SmartModules
                   └──► Internal compacted topics
C SPU authz ──────────────────────────────► multi-tenant baseline
D Tiered storage (parallel after storage cleanup)
E Transactions (last)
```

### Selection for implementation

**Execute A (log compaction)** — not the easy path (metrics, HTTP admin, DLQ), and not the largest protocol surface (full consumer groups). It is the **highest-leverage hard storage architecture change**: without it, consumer-group offsets and stream-table patterns cannot be production-safe.

B remains the top *product* feature and should follow immediately once compaction lands.

---

## Guiding principles (unchanged)

1. Don't break produce/consume and replication.
2. Ship incrementally; each PR reviewable.
3. Prefer Rust-native log design over JVM copy-paste.
4. Measure (observability) in parallel — does not block A.

## Related existing plans

Legacy task breakdown remains under `01`–`14` and `kafka-compatibility.md`. Architecture proposals **15–17** supersede those for A/B/C with code-grounded designs.
