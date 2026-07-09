# Task 9: Multi-Language SmartModule SDK (Python, Go, JS)

## Status: Not Started
## Priority: Low | Effort: Large

## Problem

SmartModules can only be written in Rust and compiled to WASM. This limits adoption:
- Most data engineers work in Python.
- Go and JavaScript are dominant in cloud-native and web ecosystems.
- The compile-to-WASM toolchain requires Rust knowledge and a nightly toolchain.

## Goal

Allow SmartModules to be written in Python, Go, and JavaScript while keeping the WASM execution model.

## Design

### Approach: Language-specific SDKs compiled to WASM

Each language SDK provides:
1. A template project with the SmartModule interface.
2. A compiler pipeline that produces a `.wasm` binary.
3. Host bindings for the Streamfy SmartModule ABI.

### Python

- Use **ComponentizeJS** or **wasm32-wasi** target with **Spin SDK** patterns.
- Alternatively, embed a Python interpreter in WASM via **RustPython** compiled to WASM — but this is heavy.
- Most practical: use **Extism** PDK for Python. Extism already supports Python-to-WASM with a lightweight runtime.

```python
from streamfy_smartmodule import smartmodule_filter

@smartmodule_filter
def filter_fn(record: bytes) -> bool:
    data = json.loads(record)
    return data.get("temperature", 0) > 30
```

### Go

- Go supports `GOOS=wasip1 GOARCH=wasm` natively since Go 1.21.
- Provide a Go SDK that implements the SmartModule ABI.

### JavaScript / TypeScript

- Use **javy** (Shopify's JS-to-WASM compiler) or **ComponentizeJS**.
- Provide an npm package `@streamfy/smartmodule` with TypeScript types.

### Common Infrastructure

- All languages compile to the same WASM ABI that `streamfy-smartengine` already executes.
- The SMDK (`smartmodule-development-kit`) gains a `--language` flag: `smdk build --language python`.
- No changes needed to the runtime — it's WASM all the way down.

## Key Files to Modify

- `crates/smartmodule-development-kit/src/build.rs` — support non-Rust build pipelines
- `crates/smartmodule-development-kit/src/generate.rs` — language-specific templates
- New repos or directories for each language SDK

## Risks

- WASM binary size and startup time may be much larger for non-Rust languages.
- Debugging experience will be poor compared to Rust (no source maps in WASM).
- Performance: Python-in-WASM is 10-100x slower than Rust-in-WASM for CPU-bound work.

## Success Criteria

- A Python SmartModule can be built with `smdk build --language python` and deployed.
- The Python SmartModule executes correctly in the same `streamfy-smartengine` runtime.
- Performance overhead is documented: latency and throughput compared to Rust equivalent.
