# 16 — Dependency Audit: Dead Deps, Build Time & Binary Size

**Date:** 2026-07-05  
**Scope:** Full Cargo workspace (`Cargo.toml` + all member crates)  
**Tools:** `cargo-machete` 0.9.2, manual source verification, `Cargo.lock` analysis  
**Workspace:** ~50 crates, **155** `[workspace.dependencies]` entries, **827** locked packages

---

## Applied

### Phase A — low-risk cleanup

| Change | Status |
|---|---|
| Remove dead direct deps | Done |
| Remove unused workspace pins | Done |
| Feature-gate `streamfy-benchmark` | Done |
| Move `k8-types` under optional `k8s` feature | Done |
| Move `tempfile` to dev-deps in `streamfy-stream-dispatcher` | Done |

### Phase B/C — high-risk / structural (applied)

| Change | Status | Notes |
|---|---|---|
| **Slim crate defaults** | Done | CLI default = `consumer,producer-file-io` only; SPU/run default = no smartengine |
| **Client crypto default `rustls-ring`** | Done | Was `rustls-aws`; use `--features rustls-aws` for aws-lc-rs |
| **`tui` → `ratatui` 0.29** | Done | Drops dual crossterm 0.25 line from our direct graph |
| **`once_cell` → `std::sync::{LazyLock,OnceLock}`** | Done | Direct dep removed from workspace members |
| **`directories` → `dirs` in cluster** | Done | Single home-dir crate in first-party code |
| **cdk/smdk `generate` feature** | Done | Default off; `cargo-generate` + vendored openssl only with `--features generate` |
| **`openssl_tls` system OpenSSL + `openssl_vendored` opt-in** | Done | `native_tls` still vendors (CLI identity helpers) |
| **Makefile product feature sets** | Done | `make build-cli` / `build-run` / `build-cdk` pass full features explicitly |

**Product builds (full weight):**

```bash
# Full CLI (k8s + benchmark)
cargo build -p streamfy-cli --release \
  --features "consumer,k8s,producer-file-io,benchmark"

# Full runner / SPU with SmartModules (wasmtime)
cargo build -p streamfy-run --release --features spu_smartengine
cargo build -p streamfy-spu --release --features smartengine

# Kits with project scaffolding
cargo build -p cdk --features generate
cargo build -p smdk -p smartmodule-development-kit --features generate

# Or use make (encodes full product features)
make build-cli build-run build-cdk build-smdk
```

**Slim defaults (faster local iteration):**

```bash
cargo build -p streamfy-cli          # consumer + file IO only
cargo build -p streamfy-spu          # no wasmtime
cargo build -p streamfy-run          # no wasmtime
cargo build -p cdk                   # no cargo-generate
```

---

## 1. Executive summary

| Category | Count | Action |
|---|---|---|
| **Confirmed dead direct deps** (safe to remove after `cargo check`) | ~15 | Remove in a cleanup PR |
| **False positives** from machete (must keep) | ~10 | Ignore / document |
| **Unused workspace.dependency declarations** | 8 | Delete from root `Cargo.toml` |
| **Easy wins for build time / binary size** | several | Feature-gate or drop optional weight |
| **Duplicate package versions in lockfile** | 66 packages | Opportunistically unify |

There is meaningful low-hanging fruit: a handful of true dead dependencies, several always-on heavy optional features (especially **benchmark**, **k8s**, **wasmtime/smartengine**, **openssl vendored**, **aws-lc-rs**), and unused workspace-level dependency pins that add noise.

---

## 2. Methodology

1. Ran `cargo machete` over the workspace.
2. Manually verified each hit against source (`rg` for imports, attributes, and proc-macro expansions).
3. Scanned root `[workspace.dependencies]` for entries never referenced by any member `Cargo.toml`.
4. Inspected default features of main binaries (`streamfy`, `streamfy-sc`, `streamfy-spu`, `streamfy-run`, `cdk`, `smdk`).
5. Counted multi-version packages in `Cargo.lock`.

> **Caveat:** `cargo-machete` is a static name-based search. It misses renames (`streamfy-package-index` → `streamfy_index`), serde `with = "…"` modules, and crates only referenced from generated macro code (`inventory`). Every “remove” item below was human-verified.

Re-run anytime:

```bash
cargo install cargo-machete
cargo machete
```

For higher confidence (slower, needs nightly for `cargo-udeps`):

```bash
cargo +nightly udeps --all-targets
```

---

## 3. Confirmed dead dependencies (safe to remove)

These have **no source usage** in the declaring crate (beyond comments / unrelated identifiers).

### 3.1 High confidence — remove

| Crate | Dependency | Notes |
|---|---|---|
| `streamfy` | `parking_lot` | Declared in `crates/streamfy/Cargo.toml`; no `parking_lot::` usage. (Still used under `streamfy-stream-dispatcher` `local` feature.) |
| `streamfy-cli` | `home` | No `home::` usage. Conflicting identifier: local module `client/home`. `home` is used in `streamfy-cli-common` only. |
| `streamfy-cli` | `current_platform` | No usage in CLI sources. Used in `streamfy-cli-common` / `cdk` / version-manager. |
| `streamfy-stream-dispatcher` | `async-lock` | No `async_lock` usage; local store uses `parking_lot` (feature-gated). |
| `streamfy-benchmark` | `thiserror` | No `thiserror` / `#[derive(Error)]` usage. |
| `streamfy-controlplane` | `tracing` | No `tracing` macros/imports; crate is thin API/types. |
| `streamfy-smartmodule` | `tracing` | No usage in smartmodule sources. |
| `streamfy-test` | `nix` | Declared with `features = ["process"]`; no `nix::` usage. |
| `smartmodule-development-kit` | `dirs` | No `dirs::` usage. |
| `smartmodule-development-kit` | `toml` | No `toml::` usage (SmartModule.toml parsed elsewhere / via other types). |
| `cdk` | `streamfy` | No direct `streamfy::` usage. Already listed in `[package.metadata.cargo-udeps.ignore]` — remove dep **and** the ignore entry. |
| `examples/partitioning-simple` | `anyhow` | Not used in example sources. |

### 3.2 `openssl` on `cdk` / `smdk` — keep (not dead)

```toml
openssl = { workspace = true, features = ["vendored"] } # cargo-generate requires openssl
```

Not imported in source, but **intentionally forced** so `cargo-generate` links against OpenSSL (vendored). Removing it can break generate on systems without system OpenSSL. Treat as **build-time cost**, not dead code (see §5).

### 3.3 Suggested cleanup patch sketch

```toml
# crates/streamfy/Cargo.toml — remove
# parking_lot = { workspace = true }

# crates/streamfy-cli/Cargo.toml — remove
# home = { workspace = true }
# current_platform = { workspace = true }

# crates/streamfy-stream-dispatcher/Cargo.toml — remove
# async-lock = { workspace = true }

# crates/streamfy-benchmark/Cargo.toml — remove
# thiserror = { workspace = true }

# crates/streamfy-controlplane/Cargo.toml — remove
# tracing = { workspace = true }

# crates/streamfy-smartmodule/Cargo.toml — remove
# tracing = { workspace = true }

# crates/streamfy-test/Cargo.toml — remove
# nix = { workspace = true, features = ["process"] }

# crates/smartmodule-development-kit/Cargo.toml — remove
# dirs = { workspace = true }
# toml = { workspace = true }

# crates/cdk/Cargo.toml — remove streamfy + udeps ignore
# streamfy = { workspace = true }

# examples/partitioning-simple/Cargo.toml — remove
# anyhow = { workspace = true }
```

After edits:

```bash
cargo check --workspace
cargo test -p streamfy-stream-dispatcher -p streamfy-cli -p streamfy-test --lib
cargo machete
```

---

## 4. False positives (do **not** remove)

| Reported unused | Why it is needed |
|---|---|
| `streamfy-package-index` in cli / cli-common / extension-common / channel-cli | Lib is renamed: `[lib] name = "streamfy_index"`. Heavily used as `streamfy_index::…`. |
| `serde-tuple-vec-map` in `streamfy-cli-common` | Used via serde attribute: `with = "tuple_vec_map"`. |
| `humantime-serde` in smartengine / connector-package | Used via `with = "humantime_serde"` on serde fields. |
| `inventory` in `streamfy-test` | Proc-macro `streamfy-test-derive` expands to `inventory::submit! { … }` in the consumer crate; must be a direct dep. |
| `md-5` in `streamfy-test` | Crate renames to `md5`; used in `tests/concurrent/util.rs`. |
| `serde` in json/sink test connectors | Needed for `#[connector(config)]` / `#[serde(…)]` derive expansion. |
| `openssl` in cdk/smdk | Link-time requirement for `cargo-generate` (see §3.2). |

---

## 5. Unused `[workspace.dependencies]` declarations

These are defined in the root `Cargo.toml` but **never referenced** by any member crate’s `Cargo.toml`:

| Dependency | Spec | Recommendation |
|---|---|---|
| `const_format` | `"0.2"` | Remove |
| `crossbeam-channel` | `"0.5"` | Remove |
| `futures-channel` | `"0.3"` | Remove |
| `getrandom` | `"0.2.15"` | Remove (still pulled transitively) |
| `k8-diff` | `"0.1.2"` | Remove |
| `mime` | `"0.3"` | Remove |
| `wasm-bindgen` | `"0.2.100"` | Remove unless planning WASM client work soon |
| `wasmparser` | `"0.235.0"` | Remove (or wire into smartengine validation if intended) |

Cleaning these does not shrink binaries by itself, but reduces workspace drift and mistaken “add this dep” copy-paste.

---

## 6. Easy wins — better build time & smaller binaries

Ordered by **impact / effort** (highest first).

### 6.1 Feature-gate `streamfy-benchmark` on the CLI (easy, high impact)

**Today:** `streamfy-cli` always depends on `streamfy-benchmark` (hdrhistogram, rayon, madato, etc.) and exposes `streamfy benchmark` unconditionally.

**Change:**

```toml
# crates/streamfy-cli/Cargo.toml
[features]
default = ["consumer", "k8s", "producer-file-io"]
benchmark = ["dep:streamfy-benchmark"]

[dependencies]
streamfy-benchmark = { workspace = true, optional = true }
```

Gate the `Benchmark` clap variant with `#[cfg(feature = "benchmark")]`.

| Effect | Notes |
|---|---|
| CLI compile time | Drops rayon + hdrhistogram + madato subgraph when unused |
| Binary size | Meaningful for release CLI artifacts |
| UX | Ship `streamfy-bench` as separate binary, or enable feature in CI/perf images only |

### 6.2 Make `k8s` non-default for slim CLI builds (easy, high impact for some artifacts)

**Today:** `default = […, "k8s", …]` pulls `k8-client`, `k8-config`, `streamfy-cluster`, `fluvio-helm`, etc.

Keep the feature, but offer documented slim profiles:

```bash
# full (current)
cargo build -p streamfy-cli --release

# slim local-dev CLI (no k8s cluster management)
cargo build -p streamfy-cli --release --no-default-features \
  --features "consumer,producer-file-io"
```

Also: `k8-types` is **not** optional in `streamfy-cli` even though only `profile/sync/k8.rs` uses it — make it part of the `k8s` feature.

### 6.3 Keep `smartengine` off the CLI; optional on SPU (already partial — tighten)

| Binary | Default `smartengine`? | Weight |
|---|---|---|
| `streamfy-cli` | **No** (good) | — |
| `streamfy-spu` | **Yes** (`default = ["smartengine"]`) | **wasmtime 38 + wasi-common** — among the heaviest deps in the tree |
| `streamfy-run` | **Yes** (`spu_smartengine`) | Same, via SPU |

For smaller SPU / edge images:

```bash
cargo build -p streamfy-spu --release --no-default-features
cargo build -p streamfy-run --release --no-default-features
```

Document this in release/Docker builds. Consider flipping defaults only if product requires SM-less SPU as primary.

### 6.4 `openssl` vendored on `cdk` / `smdk` (easy-medium, compile-time killer)

`features = ["vendored"]` compiles OpenSSL from source on every clean build of the kits.

Options:

1. **Prefer system OpenSSL** in developer docs (`OPENSSL_DIR` / pkg-config) and only vendor in release CI.
2. Check whether a newer `cargo-generate` can use **rustls** and drop the forced openssl dep entirely.
3. Split `cdk generate` / `smdk generate` into a tiny helper crate so day-to-day `cdk deploy` does not rebuild openssl.

### 6.5 Crypto provider: `aws-lc-rs` vs `ring` (easy switch, size tradeoff)

Client default:

```toml
default = ["rustls", "compress", "rustls-aws"]
```

`aws-lc-rs` is large (native crypto). For size-sensitive builds:

```bash
cargo build -p streamfy --no-default-features --features "rustls,compress,rustls-ring"
```

Validate performance/security requirements before changing the workspace default.

### 6.6 Replace unmaintained `tui` 0.19 with `ratatui` (medium, size + maintenance)

`tui` 0.19 pulls **crossterm 0.25** while the CLI also uses **crossterm 0.28** → **two crossterm major lines** in the lockfile.

Migration to `ratatui` (maintained fork) unifies crossterm and may drop old bitflags/unicode-width variants.

Scope: `streamfy-cli` consume table UI (`client/consume/table_format.rs`).

### 6.7 Consolidate `dirs` and `directories` (easy, small)

Both are used:

- `dirs::home_dir` — channel, hub-protocol, version-manager, streamfy config
- `directories::BaseDirs` — streamfy-cluster local paths

Pick one (prefer `directories` or std-only `HOME` helpers) to drop a small dep and eliminate dual `dirs` 5.x/6.x if transitive versions collapse.

### 6.8 Migrate `once_cell` → `std::sync::{OnceLock, LazyLock}` (easy-medium, many call sites)

Edition is **2024**; `LazyLock` is stable. `once_cell` appears in **~12+ crates**. Removing it reduces direct deps and slightly simplifies the graph (still may remain transitive).

### 6.9 Move test-only deps where possible

Example: `tempfile` in `streamfy-stream-dispatcher` is only used under `#[cfg(test)]` in `metadata/local.rs`.

For **unit tests**, `tempfile` can be a **dev-dependency** (available during `cargo test`, omitted from normal lib builds of dependents in some cases — verify with `cargo tree -p streamfy-stream-dispatcher`). Worth auditing other “always normal” test helpers (`portpicker`, `trybuild` is already dev in most places).

### 6.10 Do not default-enable heavy connector / plugin surfaces

Ensure production Docker images for `streamfy-sc` / `streamfy-spu` / `streamfy-run` use explicit `--features` and avoid accidental `smartengine` + full cluster tooling when not needed.

---

## 7. Medium / strategic opportunities

### 7.1 Dual HTTP stacks: `ureq` + `reqwest` + `http` 0.2/1.x

- `ureq` (pinned `=2.9.7`) used in CLI/common paths
- `reqwest` used in release-tools
- Lockfile has **http 0.2 + 1.x**, **hyper 0.14 + 1.x**

Unifying on one client for first-party code reduces duplicate protocol stacks. Lower priority than wasmtime/openssl/benchmark.

### 7.2 Duplicate lockfile versions (66 packages)

Notable multi-version packages (build-time bloat when both are compiled for a target):

| Package | Versions seen |
|---|---|
| `crossterm` | 0.25.0, 0.28.1 |
| `dirs` / `dirs-sys` | 5.x + 6.x |
| `http` / `http-body` / `hyper` | 0.2/0.4/0.14 vs 1.x |
| `rustls` | 0.22.x + 0.23.x |
| `syn` | 1.x + 2.x |
| `rand` | 0.8 + 0.9 |
| `nix` | 0.29 + 0.30 |
| `parking_lot` | 0.11 + 0.12 |
| `windows-*` | many (mostly inert on macOS/Linux hosts) |

Actions: upgrade `tui`→`ratatui`, bump crates still on old `http`/`rustls`, and periodically run `cargo update` + `cargo tree -d`.

### 7.3 `cargo-generate` only for generate subcommands

`cargo-generate` pulls a large git/template stack (gix-*, etc.). Only `cdk generate` and `smdk generate` need it. Long-term: optional feature `generate` on those bins, or a separate `cdk-generate` binary published only to developers.

### 7.4 Workspace deps used by a single crate

Many workspace pins exist for a single consumer (e.g. `madato`, `tui`, `zip`, `x509-parser`, `timeago`). That is fine for version centralization; not dead. Only remove if the feature itself is removed.

---

## 8. Dependency weight map (main products)

```
streamfy (client lib)
├── rustls + aws-lc-rs          [default, heavy crypto]
├── compression                 [default]
├── streamfy-stream-dispatcher/local
└── smartengine                 [optional — wasmtime]

streamfy-cli (bin: streamfy)
├── consumer + producer-file-io [default]
├── k8s → cluster + k8-client   [default, heavy]
├── streamfy-benchmark          [ALWAYS ON — should be optional]
├── tui + crossterm (dual ver)
├── mimalloc
└── streamfy-package-index      [used via streamfy_index]

streamfy-spu
├── smartengine → wasmtime 38   [default, very heavy]
├── storage, socket, protocol
└── mimalloc

streamfy-sc
├── k8-client, stream-dispatcher k8+local
└── no wasmtime (lighter than SPU)

streamfy-run
└── sc + spu (+ smartengine by default)

cdk / smdk
├── cargo-generate              [very heavy]
├── openssl/vendored            [slow compile]
└── (cdk) unused streamfy dep
```

---

## 9. Recommended action plan

### Phase A — cleanup PR (1–2 hours, low risk)

1. Remove confirmed dead direct deps (§3.1).
2. Remove unused workspace pins (§5).
3. Remove `cdk`’s unused `streamfy` dep and its udeps ignore.
4. `cargo check --workspace && cargo machete`.

### Phase B — binary/build profile PR (half day)

1. Feature-gate `streamfy-benchmark` on CLI (§6.1).
2. Make `k8-types` part of `k8s` feature; document slim CLI build (§6.2).
3. Document / use SPU & run builds without smartengine where appropriate (§6.3).
4. CI matrix: `full` vs `slim` release artifacts if product wants both.

### Phase C — structural (1–3 days)

1. `tui` → `ratatui` (§6.6).
2. OpenSSL strategy for kits (§6.4).
3. `once_cell` → std (§6.8).
4. Optional `generate` feature for cdk/smdk (§7.3).
5. Lockfile duplicate reduction pass (§7.2).

---

## 10. Verification checklist

```bash
# Unused direct deps
cargo machete

# What pulls a heavy crate
cargo tree -i wasmtime
cargo tree -i openssl-sys
cargo tree -i aws-lc-rs
cargo tree -i cargo-generate
cargo tree -i streamfy-benchmark

# Duplicate versions
cargo tree -d

# Slim builds
cargo build -p streamfy-cli --release --no-default-features \
  --features "consumer,producer-file-io"
cargo build -p streamfy-spu --release --no-default-features
cargo build -p streamfy --release --no-default-features \
  --features "rustls,compress,rustls-ring"

# Policy (already configured)
cargo deny check
```

Suggested size measurement after changes:

```bash
cargo build -p streamfy-cli --release
cargo build -p streamfy-spu --release
ls -lh target/release/streamfy target/release/streamfy-spu
# compare with slim feature sets
```

---

## 11. Appendix — full `cargo machete` raw report (annotated)

```
streamfy:                 parking_lot              → REMOVE
cdk:                      openssl                  → KEEP (vendored for cargo-generate)
cdk:                      streamfy                 → REMOVE
smdk:                     dirs, toml               → REMOVE
smdk:                     openssl                  → KEEP
streamfy-cli-common:      serde-tuple-vec-map      → KEEP (serde with=)
streamfy-cli-common:      streamfy-package-index   → KEEP (as streamfy_index)
streamfy-cli:             current_platform, home   → REMOVE
streamfy-cli:             streamfy-package-index   → KEEP (as streamfy_index)
streamfy-smartmodule:     tracing                  → REMOVE
streamfy-stream-dispatcher: async-lock             → REMOVE
streamfy-benchmark:       thiserror                → REMOVE
streamfy-controlplane:    tracing                  → REMOVE
streamfy-extension-common: streamfy-package-index  → KEEP (as streamfy_index)
streamfy-test:            inventory                → KEEP (macro expansion)
streamfy-test:            md-5                     → KEEP (import as md5)
streamfy-test:            nix                      → REMOVE
streamfy-connector-package: humantime-serde        → KEEP (serde with=)
streamfy-smartengine:     humantime-serde          → KEEP (serde with=)
streamfy-channel-cli:     streamfy-package-index   → KEEP (as streamfy_index)
partitioning-simple:      anyhow                   → REMOVE
json-test-connector:      serde                    → KEEP (derive)
sink-test-connector:      serde                    → KEEP (derive)
```

---

## 12. Out of scope / not recommended

- Removing `mimalloc` from SC/SPU/CLI: small binary cost, real runtime benefit for allocation-heavy streaming.
- Removing `wasmtime` entirely: required for SmartModules product surface.
- Mass-deleting single-use workspace pins solely for “cleanliness” without need — central versions are useful.
- Force-changing default crypto or smartengine without product/sign-off.

---

**Owner suggestion:** start with Phase A (dead dep removal) as a mechanical PR, then Phase B feature-gates for measurable artifact size wins.
