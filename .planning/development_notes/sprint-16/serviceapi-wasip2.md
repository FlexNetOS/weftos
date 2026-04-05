# Sprint 16: ServiceApi Trait + wasip2 Migration

Date: 2026-04-04

## Feature 1: KernelServiceApi (ADR-035)

### What was done

Added `KernelServiceApi` -- the concrete `ServiceApi` implementation backed by `ServiceRegistry`. This is the production implementation that protocol adapters (Shell, MCP, HTTP) hold via `Arc<dyn ServiceApi>`.

### Files modified

- `crates/clawft-kernel/src/service.rs` -- Added `KernelServiceApi` struct + `ServiceApi` trait impl + 9 tests
- `crates/clawft-kernel/src/lib.rs` -- Added `KernelServiceApi` to public re-exports

### Design decisions

- `KernelServiceApi::call()` verifies the service exists in the registry before dispatching. Returns a JSON envelope with `{service, method, params, status: "dispatched"}`. Full method-table dispatch will be wired when service method registrations land (K4).
- `list_services()` uses `registry.snapshot()` to avoid holding DashMap refs across async health checks. Each service's `health_check()` is awaited to populate the `healthy` field.
- `health()` delegates directly to the service's `health_check()` method.
- `registry()` accessor exposes the underlying registry for callers that need direct access (e.g., governance gate integration).

### Test coverage

9 new tests covering:
- Call dispatching to registered services
- Unknown service error path
- List services (populated and empty)
- Health check (found and not found)
- End-to-end with ShellAdapter
- End-to-end with McpAdapter
- Registry accessor

### What remains (K4+)

- Wire `call()` to actual service method tables (currently returns acknowledgement)
- Add GovernanceGate enforcement before dispatch
- Add gRPC adapter binding
- Add HTTP/REST adapter binding (per ADR-035 roadmap)

---

## Feature 2: wasip2 Migration (ADR-044)

### What was done

Migrated the WASI build target from `wasm32-wasip1` to `wasm32-wasip2` across the entire codebase. This completes the W49 work item from Sprint 11 Symposium (TD-13, HP-9).

### Files modified

- `scripts/build.sh` -- `cmd_wasi()` and `cmd_gate()` target changed to `wasm32-wasip2`
- `.cargo/config.toml` -- Target and alias updated to `wasm32-wasip2`
- `crates/clawft-wasm/src/lib.rs` -- Doc comments + `capabilities()` platform string
- `crates/clawft-wasm/src/platform.rs` -- Doc comments
- `crates/clawft-wasm/src/http.rs` -- Doc comments
- `crates/clawft-wasm/src/fs.rs` -- Doc comments
- `crates/clawft-wasm/src/env.rs` -- Doc comments
- `scripts/bench/wasm-size.sh` -- Default path updated
- `scripts/build/cross-compile.sh` -- Example target updated
- `.github/workflows/release-wasi.yml` -- Already targeted wasip2 (no change needed)

### What was already done

- `scripts/bench/wasm-size-gate.sh` -- Already referenced wasip2
- `scripts/bench/wasm-twiggy.sh` -- Already referenced wasip2
- `scripts/build/wasm-opt.sh` -- Already referenced wasip2
- `.github/workflows/release-wasi.yml` -- Already targeted wasip2

### Migration notes

- The WASM crate uses stub implementations for HTTP, FS, and env -- no WASI preview 1 APIs were being called directly, so no API migration was needed at the code level.
- The `dlmalloc` dependency (`cfg(target_arch = "wasm32")`) is target-arch gated, not target-triple gated, so it works for both wasip1 and wasip2.
- Browser WASM (`wasm32-unknown-unknown`) is unaffected by this change.
- All 41 clawft-wasm tests pass after migration.

### Pre-existing test failures (not related)

- `clawft-cli::version_output` -- hardcoded `0.1.0` vs actual `0.4.3`
- `clawft-kernel::boot::tests::services_accessible` -- service count mismatch (6 vs 5)
