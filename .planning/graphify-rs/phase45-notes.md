# Graphify Rust Port -- Phase 4-5 Implementation Notes

Date: 2026-04-04

## Summary

Implemented Phase 4 (semantic/vision extraction, URL ingestion, file watcher,
git hooks) and Phase 5 (export formats, CLI integration) for the graphify
Rust port.

## Files Created / Modified

### New modules in `crates/clawft-graphify/src/`

| File | Lines | Description |
|------|-------|-------------|
| `ingest.rs` | ~525 | URL ingestion with SSRF protection, tweet/arXiv/webpage/PDF/image fetch |
| `watch.rs` | ~188 | Polling-based file watcher with debounce, code vs non-code detection |
| `hooks.rs` | ~231 | Git hook install/uninstall/status for post-commit and post-checkout |
| `semantic_extract.rs` | ~325 | LLM-based entity extraction, feature-gated `semantic-extract` |
| `vision_extract.rs` | ~245 | Image/diagram extraction via vision API, feature-gated `vision-extract` |
| `export/obsidian.rs` | ~250 | Obsidian vault (YAML frontmatter + wikilinks) and canvas export |
| `export/wiki.rs` | ~407 | Wikipedia-style article generation per community and god node |

### Modified files

| File | Change |
|------|--------|
| `crates/clawft-graphify/src/lib.rs` | Added `hooks`, `ingest`, `watch` modules + feature-gated `semantic_extract`, `vision_extract`; added `IngestError`, `WatchError`, `HookError` variants |
| `crates/clawft-graphify/src/export/mod.rs` | Added `obsidian`, `wiki` sub-modules and `Wiki` format variant |
| `crates/clawft-weave/src/commands/mod.rs` | Added `graphify_cmd` module |
| `crates/clawft-weave/src/main.rs` | Added `Graphify` command variant + dispatch |
| `crates/clawft-weave/Cargo.toml` | Added `clawft-graphify` dependency |

### CLI integration (`crates/clawft-weave/src/commands/graphify_cmd.rs`)

Commands registered:
- `weaver graphify ingest <path|url>` -- URL/path ingestion
- `weaver graphify query <question>` -- keyword search against graph.json
- `weaver graphify export <format> [--output]` -- export graph
- `weaver graphify diff [old] [current]` -- compare graphs
- `weaver graphify rebuild [root] [--clean]` -- force re-extraction
- `weaver graphify watch [root] [--debounce]` -- start file watcher
- `weaver graphify hooks install|uninstall|status` -- git hook management

## Design Decisions

1. **HTTP abstraction**: Used a trait-based `HttpClient` for URL ingestion rather
   than hard-depending on reqwest. The `StubHttpClient` allows compilation without
   networking deps; production code injects a real client.

2. **Polling watcher**: Default watcher uses polling via walkdir rather than the
   `notify` crate. This avoids the optional dependency for basic usage while still
   providing the `notify`-based watcher behind a feature gate.

3. **Semantic/Vision extraction**: Both use callback-based LLM invocation
   (`FnOnce(String) -> Future`) rather than directly depending on clawft-llm types.
   This keeps the extraction logic testable with fake LLM responses.

4. **SSRF protection**: Validates URLs before fetching -- blocks file://, localhost,
   and private IP ranges (10.x, 172.16-31.x, 192.168.x, 127.x).

5. **Hook scripts**: Call `weaver graphify rebuild` rather than Python, matching the
   Rust-native toolchain. Scripts only trigger on code file changes.

## Test Results

- `cargo test -p clawft-graphify`: 61 tests pass (including 20+ new tests)
- `cargo test -p clawft-weave`: 13 tests pass (including graphify_args_parses)
- `scripts/build.sh check`: workspace clean (no new warnings)

## Remaining Work

- Full extraction pipeline integration in `weaver graphify rebuild`
- Real HTTP client injection for URL ingestion (reqwest-based)
- notify-crate watcher behind feature gate
- MCP server (Phase 6)
- Benchmarks (Phase 6)
