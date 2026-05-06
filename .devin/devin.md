# Devin workspace guide — FlexNetOS/weftos

This document tells Devin (and any other AI agent or new contributor) how to
build, test, lint, and format the WeftOS workspace correctly. The
canonical entrypoint is **`scripts/build.sh`** — never run cargo directly.

Bootstrap a fresh environment with [`./setup.sh`](./setup.sh).

## Mandatory build entrypoint

> **MANDATORY: Use `scripts/build.sh` for ALL build, test, check, and lint
> operations.** Do NOT run `cargo build`, `cargo test`, `cargo check`, or
> `cargo clippy` directly unless debugging a specific compilation issue
> that requires direct cargo flags not exposed by the script. If that
> happens, **extend `scripts/build.sh`** with the new capability so future
> builds use it.

This rule is restated in `CLAUDE.md` (lines 43–47) and is non-negotiable —
the script wraps cargo, WASI, and browser-WASM builds with consistent flag
sets so every contributor and CI lane is exercising the same code paths.

## `scripts/build.sh` subcommands

| Command | Purpose |
|---|---|
| `scripts/build.sh native` | Release build of `weft` and `weaver` binaries. |
| `scripts/build.sh native-debug` | Debug build (faster compile, slower runtime). |
| `scripts/build.sh test` | `cargo test --workspace`. |
| `scripts/build.sh check` | `cargo check --workspace` — fast type-check. |
| `scripts/build.sh clippy` | `cargo clippy --workspace -- -D warnings`. |
| `scripts/build.sh wasi` | `wasm32-wasip2` build with `--profile release-wasm`. |
| `scripts/build.sh browser` | `wasm32-unknown-unknown` + `wasm-bindgen` → `crates/clawft-wasm/www/pkg/`. |
| `scripts/build.sh all` | Native + WASI + browser. |
| `scripts/build.sh gate` | 11-check phase gate (run before committing). |

The `gate` subcommand runs: `cargo fmt --check`, `cargo check`,
`cargo clippy -- -D warnings`, `cargo test`, native release build of
`weft` + `weaver`, WASI build, browser build, browser bundle size
assertion (<300 KB raw, <120 KB gzipped), crate-publish dry-run,
`cargo doc --no-deps`, and schema/manifest validation. Run it before
opening a PR.

Use `scripts/build.sh --dry-run <subcommand>` to preview the cargo command
without executing, and `scripts/build.sh --features <list> <subcommand>`
to add feature flags.

## Workspace shape

- ~40 crates under `crates/` (workspace members in the root `Cargo.toml`).
- `gui/src-tauri/` is **excluded** from the workspace (`[workspace] exclude
  = ["gui/src-tauri"]`). Tauri's bundle is built from inside that directory
  with its own toolchain. Do not add it back to the workspace.
- Toolchain is pinned to **Rust 1.93** in `rust-toolchain.toml` (edition
  **2024** requires 1.85+).
- Two WASM targets:
  - `wasm32-wasip2` (NOT `wasm32-wasi`) — server / CLI WASM, plugins, edge.
  - `wasm32-unknown-unknown` — browser GUI bundle, post-processed by
    `wasm-bindgen`.
- WASM builds use the `release-wasm` profile (size-optimized: `opt-level =
  "z"`, `lto = true`, `codegen-units = 1`, `strip = true`).

## Naming taxonomy

The biggest source of newcomer confusion: the **product** is "WeftOS", the
internal **crates** are `clawft-*` (legacy name kept for crates.io
stability and ADR-001 lockstep semver), and the **binaries** are `weft`
(the kernel daemon, built from `clawft-cli`) and `weaver` (the CLI /
orchestration tool, built from `clawft-weave`). Don't grep for
`weftos-cli` — it doesn't exist.

## Commit-time checklist

1. `scripts/build.sh gate` passes (the full 11-check gate).
2. `scripts/build.sh test` passes.
3. No new occurrences of `ruvnet/`, `weave-logic-ai/`, or
   `weftos/weave-logic-ai-tap` in the diff (the canonical GitHub slug is
   `FlexNetOS/weftos`):
   ```bash
   rg -n 'ruvnet/|weave-logic-ai/|weftos/weave-logic-ai-tap|ruvnet/claude-flow' \
      --glob '!CHANGELOG.md' --glob '!**/*.lock'
   ```

## CI / release model

Lockstep semver (ADR-001): a single `[workspace.package] version` bumps
all ~36 internal crates together. Releases are driven by **cargo-dist**
on git-tag pushes (`.github/workflows/release.yml`). Phase gates
(`.github/workflows/pr-gates.yml`) enforce the browser bundle size
budget. See knowledge note "weftos: lockstep semver (ADR-001)" for the
full release procedure.
