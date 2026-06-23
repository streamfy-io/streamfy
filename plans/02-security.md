# Task 2: Wire SPU Authentication & Add Encryption at Rest

## Status: Not Started
## Priority: High | Effort: Medium

## Problem

1. **SPU auth is not wired.** There is a `TODO: add X509Authenticator` in `crates/streamfy-spu/src/start.rs`. The SC has X.509 cert-based auth, but the SPU accepts unauthenticated connections. Any client can produce/consume directly from an SPU if they know the address.

2. **No encryption at rest.** Data segments are stored as plaintext files on disk. Anyone with filesystem access can read all messages.

3. **No audit logging.** There is no record of who produced/consumed what, when.

## Goal

- SPU enforces mTLS authentication on all client connections.
- Segment files are encrypted at rest using AES-256-GCM with key management support.
- Auth events (connect, produce, consume, denied) are logged in a structured format.

## Scope

### Phase 1: SPU mTLS Auth

- Wire `X509Authenticator` in SPU's `start.rs` (the auth crate already supports it).
- Reject connections that don't present a valid client certificate.
- Map certificate CN to scopes using the same scope-binding JSON the SC uses.
- Add `--require-client-auth` flag (default `true` when TLS is enabled).

### Phase 2: Encryption at Rest

- Add an `EncryptedSegmentWriter` wrapper around the segment writer in `streamfy-storage`.
- Use AES-256-GCM with a per-segment nonce stored in the segment header.
- Key management: support file-based keys initially, with a trait for external KMS (AWS KMS, HashiCorp Vault) later.
- Encryption should be configurable per-topic (`encryption: { enabled: true, key_id: "..." }`).

### Phase 3: Audit Log

- Emit structured JSON audit events for: client connect/disconnect, produce, consume, topic create/delete, auth denied.
- Write to a dedicated `__audit` internal topic or a configurable file sink.
- Include: timestamp, client_id, cert_cn, action, topic, partition, result.

## Key Files to Modify

- `crates/streamfy-spu/src/start.rs` — wire X509Authenticator
- `crates/streamfy-auth/src/x509/` — ensure SPU-side compatibility
- `crates/streamfy-storage/src/segment.rs` — add encryption layer
- `crates/streamfy-sc/src/services/public_api/` — audit event emission

## Success Criteria

- SPU rejects connections without valid client certs when TLS is enabled.
- Segment files on disk are unreadable without the encryption key.
- Audit log captures all produce/consume operations with caller identity.
