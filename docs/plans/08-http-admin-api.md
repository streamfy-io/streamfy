# Task 8: HTTP Admin API & Health Checks

## Status: Not Started
## Priority: Medium | Effort: Small

## Problem

All administration is done via the CLI (`streamfy topic create`, `streamfy cluster start`, etc.) or the Rust client SDK. There is no HTTP API for:

- Programmatic cluster management from non-Rust applications.
- Health checks for load balancers and Kubernetes liveness/readiness probes.
- Integration with infrastructure-as-code tools, dashboards, and monitoring systems.

Currently, K8s health checks rely on process liveness only — there is no application-level readiness check.

## Goal

Add an HTTP REST API to the SC for cluster administration and health checks.

## Design

### Health Endpoints (Phase 1)

Both SC and SPU expose:

```
GET /healthz          → 200 OK (liveness)
GET /readyz           → 200 OK when ready to serve, 503 otherwise
GET /readyz?verbose   → JSON with component status details
```

Readiness criteria:
- **SC**: connected to metadata store (K8s or local), at least 1 SPU registered.
- **SPU**: connected to SC, all assigned replicas are online.

### Admin API (Phase 2)

SC exposes a REST API on a configurable port (default `:9080`):

```
GET    /api/v1/topics                    → list topics
POST   /api/v1/topics                    → create topic
GET    /api/v1/topics/:name              → describe topic
DELETE /api/v1/topics/:name              → delete topic
GET    /api/v1/topics/:name/partitions   → list partitions
GET    /api/v1/spus                      → list SPUs
GET    /api/v1/smartmodules              → list SmartModules
POST   /api/v1/smartmodules              → upload SmartModule
GET    /api/v1/cluster/status            → cluster overview
```

### Implementation

- Use `axum` (already common in Rust async ecosystem) for the HTTP server.
- Handlers delegate to existing SC service logic (`create.rs`, `delete.rs`, `list.rs` in `services/public_api/`).
- Auth: reuse existing X.509 or add optional Bearer token auth.
- OpenAPI spec generated via `utoipa` for documentation.

## Key Files to Modify

- `crates/streamfy-sc/src/start.rs` — start HTTP server alongside gRPC
- `crates/streamfy-sc/src/` — new `http/` module with axum routes
- `crates/streamfy-spu/src/start.rs` — add `/healthz` and `/readyz`
- `k8-util/helm/streamfy-app/` — update K8s deployment with readiness/liveness probes

## Dependencies

- `axum = "0.8"`, `utoipa = "5"`, `utoipa-axum = "0.2"`

## Success Criteria

- `curl localhost:9080/healthz` returns 200 on a healthy SC/SPU.
- `curl localhost:9080/api/v1/topics` returns JSON list of topics.
- K8s readiness probe uses `/readyz` and correctly gates traffic.
- OpenAPI spec is available at `/api/v1/openapi.json`.
