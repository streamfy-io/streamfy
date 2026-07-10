# Schema Registry with Produce-Time Validation

## Status: Not Started
## Priority: High | Effort: Medium–Large
## Estimated agent time: 45–90 minutes for Phase 1 MVP

> Not covered in `docs/plans/` (observability, security, compaction, consumer groups, EOS, MCP, etc.). Complements existing work: SmartModules transform data after it is already accepted; this stops bad data **before** it hits the log.

---

## Problem

Streamfy accepts any byte payload on produce. There is:

- No schema object in control-plane metadata (`TopicSpec` has compression, retention, deduplication — no schema reference).
- No produce-path validation in `crates/streamfy-spu/src/services/public/produce_handler.rs`.
- No CLI surface for registering or inspecting schemas.
- No error code for schema mismatch (see `crates/streamfy-protocol/src/link/error_code.rs`).

That means:

1. **Poison data** lands in topics and only fails later in consumers / SmartModules.
2. **Contract evolution** between producers and consumers is informal (docs, tribal knowledge).
3. **Connectors and multi-team platforms** cannot enforce “this topic is JSON matching X”.
4. Competing systems (Kafka + Schema Registry / Redpanda / Pulsar schema) treat this as table-stakes for production.

Today a producer can write `"not-json"` into a topic every other service expects to be structured JSON, and Streamfy will happily store it.

## Goal

Ship a **built-in Schema Registry** as a first-class Streamfy object, and optionally **validate records on produce** against the schema bound to a topic.

MVP (Phase 1) is enough to be useful and demoable:

- Register / list / get / delete **JSON Schema** subjects.
- Bind a subject (+ version or `latest`) to a topic.
- On produce, when a topic has a schema binding with `mode = enforce`, reject invalid records with a clear error.
- CLI: `streamfy schema …` and topic create/config flags.

Later phases: Avro/Protobuf, compatibility modes, consume-time decode helpers, registry HTTP API.

---

## Design

### Data model

New control-plane object: **Schema** (subject-versioned), stored like SmartModules / TableFormats.

```text
SchemaSpec {
  subject: String,           // e.g. "orders-value"
  schema_type: SchemaType,   // JsonSchema | Avro | Protobuf  (MVP: JsonSchema only)
  schema: String,            // schema body (JSON text for JsonSchema)
  references: Vec<SchemaRef> // optional, empty in MVP
}

SchemaStatus {
  version: u32,              // monotonically increasing per subject
  fingerprint: String,       // sha256 of normalized schema body
  created_at: i64,
}
```

Topic binding (on `TopicSpec` or a sidecar config map — prefer optional field on `TopicSpec` with a new protocol min_version):

```text
TopicSchemaConfig {
  key_subject: Option<SchemaBinding>,    // optional in MVP; can skip keys first
  value_subject: Option<SchemaBinding>,
}

SchemaBinding {
  subject: String,
  version: Option<u32>,      // None = latest at bind time, or resolve latest on each produce (config)
  mode: SchemaMode,          // Enforce | Warn | Disabled
}

enum SchemaMode {
  Enforce,   // reject invalid produce
  Warn,      // accept + log/metric (Phase 2)
  Disabled,
}
```

Compatibility policy (per subject, Phase 2):

```text
enum Compatibility {
  None,          // MVP default — any new version allowed
  Backward,      // new schema can read old data
  Forward,
  Full,
}
```

### Validation placement

Validate **on the SPU produce path**, after the request is decoded and before append:

```text
Producer
  → SPU handle_produce_request
      → handle_produce_topic
          → [NEW] resolve TopicSchemaConfig from global context
          → for each record value (and key if configured):
                decode UTF-8 / treat as JSON
                validate against cached jsonschema
                on failure → ErrorCode::SchemaValidationFailed
          → existing SmartModule + append path
```

Key files today:

| Area | Path |
|------|------|
| Produce entry | `crates/streamfy-spu/src/services/public/produce_handler.rs` |
| Topic metadata | `crates/streamfy-controlplane-metadata/src/topic/spec.rs` |
| Error codes | `crates/streamfy-protocol/src/link/error_code.rs` |
| SC public API pattern | `crates/streamfy-sc/src/services/public_api/smartmodule/` or `tableformat/` |
| Metadata object pattern | `crates/streamfy-controlplane-metadata/src/smartmodule/` or `tableformat/` |
| CLI pattern | `crates/streamfy-cli/src/client/smartmodule/` |
| Admin object list/create | `crates/streamfy-sc-schema/src/` |

**Do not** validate only in the CLI — clients using the Rust SDK / other producers must get the same guarantees from the SPU.

### Caching

SPUs already receive topic/partition metadata from the SC. Extend that so each SPU has:

1. `TopicSpec` including `schema` binding.
2. A local **schema cache**: `HashMap<(subject, version), Arc<CompiledSchema>>`.
3. Lazy fetch of schema body from SC (or push via existing watch/metadata sync) on first produce for that binding.
4. Invalidate on schema delete / new version when binding uses `latest`.

For MVP, embedding the compiled schema id + fingerprint on the partition/topic metadata that SPU already watches is enough; full schema body can be fetched once and cached in memory.

### Error surface

Add to `ErrorCode`:

```rust
#[streamfy(tag = /* next free, e.g. 80 */)]
#[error("schema validation failed for topic {topic}: {reason}")]
SchemaValidationFailed { topic: String, reason: String },

#[streamfy(tag = /* e.g. 81 */)]
#[error("schema subject not found: {0}")]
SchemaNotFound(String),

#[streamfy(tag = /* e.g. 82 */)]
#[error("schema already exists: {0}")]
SchemaAlreadyExists(String),
```

Produce response should set the partition-level `error_code` (same pattern as other produce failures) so the client SDK surfaces it without dropping the whole connection.

### CLI (MVP)

```bash
# Register a JSON Schema under a subject (creates version 1, or next version)
streamfy schema register orders-value --type json --file orders.schema.json

# List subjects / versions
streamfy schema list
streamfy schema get orders-value
streamfy schema get orders-value --version 2

# Delete (optional soft-delete later)
streamfy schema delete orders-value --version 2
streamfy schema delete orders-value --all

# Bind on topic create / update
streamfy topic create orders \
  --partitions 3 \
  --schema-value orders-value \
  --schema-mode enforce

streamfy topic update orders --schema-value orders-value --schema-mode enforce

# Describe shows binding
streamfy topic describe orders
```

### Client SDK

In `crates/streamfy` producer path:

- Map `SchemaValidationFailed` to a typed error (not a generic `Other`).
- Optionally: client-side pre-validation when schema is known (Phase 2; avoids round-trip) — server validation remains authoritative.

---

## Implementation plan (agent checklist)

### Phase 1 — MVP (do this first; >10 min, high value)

1. **Metadata crate**
   - Add `crates/streamfy-controlplane-metadata/src/schema/` with `SchemaSpec`, `SchemaStatus`, `SchemaType`, store/K8s wiring mirroring `tableformat` or `smartmodule`.
   - Add `TopicSchemaConfig` / `SchemaBinding` to `TopicSpec` behind a new `min_version` so old clients still decode.

2. **Protocol**
   - New `ErrorCode` variants for schema validation / not found / already exists.
   - Bump relevant API versions if metadata encoding requires it (follow existing `#[streamfy(min_version = N)]` pattern).

3. **SC admin API**
   - Create / delete / list / watch handlers under `streamfy-sc` public API (copy structure from `tableformat` or `smartmodule`).
   - Auth: reuse `ObjectType` + `TypeAction` / `InstanceAction` (add `ObjectType::Schema` in extended types).

4. **sc-schema + CLI**
   - Register `SchemaSpec` as an admin object.
   - Implement `streamfy schema register|list|get|delete`.
   - Extend `topic create` / `topic describe` with schema flags.

5. **SPU validation**
   - New small crate or module e.g. `streamfy-schema` (or `streamfy-spu/src/schema/`) with:
     - JSON Schema compile + validate using a maintained crate (`jsonschema` crate is fine).
     - Cache keyed by `(subject, version)` or fingerprint.
   - Hook into `handle_produce_partition` (or just before append in `handle_produce_topic`):
     - If no binding or mode ≠ Enforce → no-op.
     - For each record value: if empty, skip or fail based on schema `type` (document choice: **null/empty values skip key validation; empty value fails if schema requires object**).
     - On invalid: set `error_code = SchemaValidationFailed` and do not append that partition batch.

6. **Tests**
   - Unit: validate valid / invalid JSON against a sample schema.
   - Integration (SPU produce tests in `crates/streamfy-spu/src/services/public/tests/produce.rs` style):
     - Topic with enforce + valid payload → Ok + offset.
     - Topic with enforce + invalid payload → `SchemaValidationFailed`, no log growth.
     - Topic without schema → current behavior unchanged.
   - CLI smoke (optional bats under `tests/cli/` if easy).

7. **Docs**
   - Short section in README or `docs/` example: register schema → create topic → produce good/bad messages.

### Phase 2 — Production polish (follow-up)

- Compatibility modes (`BACKWARD` / `FORWARD` / `FULL`) on `schema register`.
- `Warn` mode + metrics counters (`streamfy_schema_validation_failures_total`) once observability lands.
- Key schema binding.
- Schema push to SPU via metadata watch (no lazy SC fetch).
- HTTP admin endpoints if Task #8 (HTTP Admin API) exists.

### Phase 3 — Multi-format

- Avro (Apache Avro rust) and Protobuf (file descriptor set).
- Confluent-compatible wire envelope optional mode (`magic + schema-id + payload`) for Kafka-proxy interop later.
- Client helpers: encode/decode with registered schema id in headers (`RecordHeader` already exists in protocol).

---

## Non-goals (MVP)

- Full Confluent Schema Registry HTTP API compatibility.
- Exactly-once interaction with schema evolution.
- Validating SmartModule intermediate outputs (could use same engine later).
- Replacing TableFormat (display) — schemas are about **contracts**, table formats about **presentation**.

---

## Success criteria

- [ ] `streamfy schema register` stores a versioned JSON Schema subject in the cluster.
- [ ] Topic can bind `value` subject with `enforce` mode.
- [ ] Produce of JSON matching the schema succeeds and is readable via consume.
- [ ] Produce of invalid JSON fails with `SchemaValidationFailed` and does **not** append to the partition log.
- [ ] Topics without schema config behave exactly as today (no regression on existing produce tests).
- [ ] Unit + at least one integration test green for the enforce path.
- [ ] `cargo check` / clippy clean for touched crates.

## Suggested agent workflow

1. Mirror `tableformat` (metadata + SC create/list/delete + CLI) for the Schema object — fastest path to admin CRUD.
2. Add topic binding fields + CLI flags.
3. Implement `streamfy-schema` validator + SPU hook + error codes.
4. Write tests last for the enforce path; run existing produce tests to prove no regression.
5. Stop after Phase 1 unless time remains; leave Phase 2/3 as TODOs in code comments only where necessary.

## Why this is a good agent task

- Touches **many layers** (metadata, SC, protocol, SPU, CLI) so it takes real time, not a one-file tweak.
- Patterns already exist (`tableformat`, `smartmodule`) so an agent can copy structure without inventing architecture.
- Clear success criteria and a natural stop line after Phase 1.
- High product value: production multi-team streaming almost always needs payload contracts.
