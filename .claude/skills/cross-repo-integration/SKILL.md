---
name: cross-repo-integration
description: Map and execute the FlexNetOS/weftos ↔ FlexNetOS/ruvector integration topology. Use when a task spans both repos — e.g. wiring weft kernel to ruvector's mcp-brain-server, calling AgentDB HNSW from a clawft-* crate, or surfacing ReasoningBank trajectories in weaver. weftos is the RUNTIME; ruvector is the BRAIN.
triggers:
  - cross-repo
  - ruvector integration
  - brain integration
  - mcp-brain-server
  - pi.ruv.io
  - reasoningbank
  - agentdb
  - hnsw from weftos
  - share to brain
  - distill trajectory
---

# Cross-Repo Integration — WeftOS (runtime) ↔ RuVector (brain)

## Topology

```
                ┌──────────────────────────────┐
                │         WeftOS               │
                │  (FlexNetOS/weftos, master)  │
                │                              │
                │  weft daemon  ◄── kernel  ──►│
                │  weaver CLI       services   │
                │  clawft-* crates             │
                └──────────────┬───────────────┘
                               │ JSON-RPC / HTTP / MCP
                               ▼
                ┌──────────────────────────────┐
                │        RuVector              │
                │  (FlexNetOS/ruvector, main)  │
                │                              │
                │  mcp-brain-server  ◄── REST  │
                │  AgentDB + HNSW              │
                │  ReasoningBank (sona)        │
                │  prime-radiant (witness)     │
                └──────────────────────────────┘
```

This skill lives in **weftos** (the runtime). Read the matching skill in
**ruvector** at `.claude/skills/cross-repo-integration/SKILL.md` for the
brain side.

## What weftos provides

| Capability | Crate | Wire format |
|---|---|---|
| Kernel daemon | `crates/clawft-cli` (binary `weft`) | Unix socket JSON-RPC |
| Orchestration CLI | `crates/clawft-weave` (binary `weaver`) | RPC client |
| Cognitive substrate | `crates/clawft-substrate` | path-keyed state tree |
| Surface (UI IR) | `crates/clawft-surface` | declarative UI |
| Knowledge graph extraction | `crates/clawft-graphify` | tree-sitter codebase topology |
| LLM provider abstraction | `crates/clawft-llm` (11 providers) | trait-based |
| WASM kernel | `crates/clawft-wasm` | `wasm32-unknown-unknown` |
| Egui UI shell | `crates/clawft-gui-egui` | native + wasm |

**Public surfaces ruvector can call:**

1. JSON-RPC over the daemon's Unix socket (default discovery: walk up
   from CWD for `.weftos/`, then `~/.clawft/` fallback). Methods are
   namespaced: `kernel.*`, `agent.*`, `cluster.*`, `chain.*`,
   `substrate.*`, `app.*`, `health.*`. See
   `crates/clawft-rpc/src/protocol.rs` for `runtime_dir()` resolution.
2. The substrate state tree — `substrate.subscribe` to a topic to
   receive `StateDelta`s. ruvector's brain consumes these for memory
   distillation.
3. The MCP tool catalog at `crates/clawft-kernel/src/wasm_runner/catalog.rs`
   exposes the same methods to WASM plugins.

## What ruvector consumes from weftos

A ruvector pipeline that wants to learn from a weft runtime currently
has these touch points:

- `crates/clawft-substrate` adapters — emit `StateDelta`s that ruvector's
  ingestion consumes for `ReasoningBank` trajectories.
- `crates/clawft-kernel/src/causal*` — causal-graph snapshots that map
  cleanly onto ruvector's `mincut` / `prime-radiant` graph store.
- `crates/clawft-weave/src/commands/graphify_cmd.rs` — produces the same
  tree-sitter graph that ruvector's `ruvector-cnn` ingests.

## What weftos consumes from ruvector

Add a `RuVectorBrainProvider` in `crates/clawft-llm/` that wraps the
`pi-brain` MCP / `mcp-brain-server` HTTP API. Until then, the wiring is:

- Register the `pi-brain` MCP server in the agent-runtime MCP config
  (`~/.codex/mcp.json`, `.cursor-mcp.json`, or equivalent).
- Use the methods `brain_status`, `brain_search`, `brain_share`,
  `brain_list`, `brain_drift`, `brain_partition` from agent prompts /
  weaver scripts.

## Integration verification checklist

Run these after any change that touches the weftos ↔ ruvector contract.

### 1. Daemon discovers correct runtime dir

```bash
# from a weftos workspace
cd /path/to/some/project
mkdir -p .weftos
weaver kernel start
weaver kernel status   # should report PID + sock at $PWD/.weftos/runtime/
weaver kernel shutdown
```

`runtime_dir()` resolution order: `WEFTOS_RUNTIME_DIR` env →
walked-up `.weftos/` → `~/.clawft/`. Test all three.

### 2. Build via `scripts/build.sh` (mandatory)

> **WeftOS rule:** never run raw `cargo build/test/check/clippy`. Use
> `scripts/build.sh` for ALL operations. Add new flags to the script
> rather than bypassing it.

```bash
scripts/build.sh check     # cargo check --workspace
scripts/build.sh test      # cargo test --workspace (use after every code change)
scripts/build.sh gate      # the 11-check phase gate (run before commit)
scripts/build.sh native    # release build of weft + weaver
scripts/build.sh wasi      # wasm32-wasip2
scripts/build.sh browser   # wasm32-unknown-unknown + wasm-bindgen
```

### 3. Brain provider can reach ruvector

```bash
# start ruvector mcp-brain-server in another shell
# (cd /path/to/ruvector && cargo run -p mcp-brain-server)

curl -fsS http://localhost:7333/v1/status | jq
weaver brain ping || echo "[weave brain ping is a planned subcommand]"
```

### 4. Substrate → brain ingestion sanity check

```bash
weaver substrate dump --topic substrate/cognitive | head -20
# pipe a few deltas to mcp-brain-server's ingestion endpoint
```

### 5. Cross-repo schema drift check

If a kernel JSON-RPC method's params/result type changes, ruvector
clients break silently. To detect:

```bash
weaver kernel dump-rpc-schema > /tmp/weftos-rpc.json     # planned subcommand
diff /tmp/weftos-rpc.json /path/to/ruvector/crates/mcp-brain/schemas/weftos.rpc.json
```

Add this diff to `scripts/build.sh gate` once both subcommands exist.

## Versioning policy

- **weftos** is lockstep semver (ADR-001) — every crate in the workspace
  shares one version. Bumping any crate bumps all of them.
- A weftos release pins a specific ruvector `mcp-brain-server` major
  version. The pin lives in:
  - `crates/clawft-llm/Cargo.toml` once the brain client is implemented;
  - the docs site `package.json` for any browser-side consumption.
- Pre-1.0, **minor = breaking, patch = non-breaking** (Cargo 0.x convention).

## Common failure modes

| Symptom | Likely cause | Fix |
|---|---|---|
| `cargo wasm` link errors about `tokio::net` / `mio` / `getrandom` | a default-enabled dep pulled in syscalls not available on `wasm32` | gate the dep behind the `native` feature; add a `#[cfg(target_arch = "wasm32")]` shim |
| Browser bundle exceeds 300KB raw / 120KB gzipped | new dep not gated under `native` | same as above; `wasm-size` gate in `pr-gates.yml` will fail until fixed |
| `weft status` returns "no kernel running" but daemon is up | `WEFTOS_RUNTIME_DIR` mismatch | unset env var or set it explicitly to the project's `.weftos/runtime/` |
| weaver can't see `pi-brain` MCP | brain server unreachable / MCP config out of date | `curl http://localhost:7333/v1/status`; re-register MCP |
| `scripts/build.sh gate` fails on WASM size | someone added a heavy dep without browser gate | bisect the diff; gate offending crate behind `native` feature |
| New `unsafe` block | review by hand even though `unsafe_op_in_unsafe_fn = allow` | ruvector's lint policy applies to research velocity, not safety |

## Where to look first

- `crates/clawft-rpc/src/protocol.rs` — wire types + `runtime_dir()`.
- `crates/clawft-kernel/src/lib.rs` — module index, K0–K6 phase model.
- `crates/clawft-llm/` — provider abstraction (where the brain client
  goes).
- `crates/clawft-substrate/` — state tree + ontology adapters.
- `scripts/build.sh` — canonical build entrypoint.

## Related skills

- `weftos-build-deploy` — release engineering across all channels.
- `weftos-docs-deploy` — Fumadocs site deploy.
- `weftos-api-docs` — rustdoc API reference deploy.
- `self-learning-loop` (this repo) — Implement → Validate → Optimize → Distill cycle.

## Forbidden actions

- Do **not** run raw `cargo` commands except for ad-hoc debugging. Use
  `scripts/build.sh`. If a flag is missing, **add it to the script.**
- Do **not** publish a single `clawft-*` crate independently. Lockstep
  semver assumes the workspace ships together.
- Do **not** push tags from a non-master branch. Releases trigger off
  master tags.
- Do **not** commit `crates/clawft-wasm/www/.env-keys.json` (it is
  populated by `scripts/build.sh serve` and gitignored).
- Do **not** modify generated `target/` artifacts.
