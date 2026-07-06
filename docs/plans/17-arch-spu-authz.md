# Architecture Change C: SPU Trust Boundary (Data-Plane AuthZ)

## Status: Not Started
## Priority: Critical security | Effort: Medium–Large

## Problem

SC enforces X.509 + RBAC on admin. SPU always uses `RootAuthorization` and has `//TODO: add X509Authenticator` in `start.rs`. Auth context is created on connect but **produce/fetch handlers never check it**. Anyone who can reach an SPU address can read/write data.

## Goal

Close the data-plane trust boundary: authenticated identity + **topic/partition-scoped authorization** on Produce, Fetch, StreamFetch, and offset APIs.

## Design decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Transport auth | mTLS + existing `X509Authenticator` | Parity with SC proxy |
| AuthZ model | Extend beyond SC `ObjectType` to **topic actions** (Produce, Consume) | Admin CRUD ≠ data path |
| Local/dev | `--require-client-auth=false` / Root when TLS off | Do not break single-node DX |
| Audit | Structured events phase 2 | Separate from enforcement |

## Phases

1. Wire SPU TLS authenticator (same path as SC)
2. Map cert CN → scopes → topic produce/consume allow
3. Enforce in public handlers; reject with clear error codes
4. Optional encryption at rest (see `02-security.md`) — separate from authz

## Success criteria

- Unauthenticated produce/fetch rejected when auth required
- Scope-limited principal cannot access other topics
- Existing unauthenticated local cluster mode still works when configured
