---
name: self-learning-loop
description: Run the FlexNetOS self-learning agent loop — Identify → Implement → Validate → Optimize → Distill — using the Attractor NLSpec as the blueprint and weftos's weaver + kernel as the runtime. Use when designing or extending agent behavior that should accumulate experience over multiple runs.
triggers:
  - self-learning
  - learning loop
  - identify implement validate optimize distill
  - reasoning bank
  - reasoningbank
  - attractor pipeline
  - phase gate
  - witness chain
  - distill trajectory
  - close the loop
---

# Self-Learning Loop — Attractor pipeline on weftos

## What this skill does

Defines the canonical self-learning agent cycle this repo executes,
based on the [strongdm/attractor](https://github.com/strongdm/attractor)
non-interactive coding agent NLSpec. weftos is the **runtime** side of
the loop — `weaver` orchestrates, `weft` executes; ruvector is the
memory + reasoning side. This skill is the weftos-side reference.

## The five-node cycle

```
   ┌──────────────┐
   │ 1. Identify  │  pick the next learnable signal (drift, failure, gap)
   └──────┬───────┘
          ▼
   ┌──────────────┐
   │ 2. Implement │  apply a change (code edit, parameter sweep, prompt tweak)
   └──────┬───────┘
          ▼
   ┌──────────────┐
   │ 3. Validate  │  scripts/build.sh gate — 11 checks must pass
   └──────┬───────┘
          ▼
   ┌──────────────┐
   │ 4. Optimize  │  tune, prune, reduce — keep what improves the verdict
   └──────┬───────┘
          ▼
   ┌──────────────┐
   │ 5. Distill   │  store the trajectory in ruvector's ReasoningBank
   └──────┬───────┘
          │  (next iteration starts from a richer memory)
          ▼ ── back to 1.
```

A single Attractor DOT graph (committed at `.attractor/integration.dot`
in Phase 3) encodes this cycle declaratively; this skill is the
human-readable reference.

## Where each node lives in weftos

| Node | Code | Notes |
|---|---|---|
| 1. Identify | `crates/clawft-kernel/src/causal*.rs` + `crates/clawft-kernel/src/cognitive_tick.rs` | DEMOCRITUS drift signals from the cognitive substrate. |
| 2. Implement | `crates/clawft-weave/src/commands/*.rs` (`weaver`) | The orchestration CLI applies the change. |
| 3. Validate | `scripts/build.sh gate` (11 checks) | The phase gate is the validation contract. |
| 4. Optimize | `crates/eml-core` + `crates/clawft-kernel/src/eml_*.rs` | EML coordinate descent for online tuning. |
| 5. Distill | external — ruvector's `ReasoningBank` over the brain MCP | weftos pushes trajectories; ruvector stores them. |

## Run it locally

### Prerequisites

```bash
# All cargo work goes through scripts/build.sh — never invoke cargo directly.
scripts/build.sh check
```

### One iteration of the loop

```bash
# 1. Identify — read drift signals from the cognitive substrate
weaver kernel start
weaver substrate dump --topic substrate/cognitive | jq '.drift_count'

# 2. Implement — apply a change (example: tune a kernel feature flag)
weaver kernel config set <key> <value>

# 3. Validate — run the phase gate (11 checks)
scripts/build.sh gate

# 4. Optimize — invoke EML coordinate descent
weaver eml tune --target validate_score --budget 20

# 5. Distill — push the trajectory to ruvector's brain
weaver brain share \
    --category solution \
    --title "tuned <key> from <old> to <new>" \
    --content "$(scripts/build.sh gate --json)" \
    --tags weftos,gate,trajectory
```

The exact CLI surface above is aspirational where it does not yet exist.
When implementing, **add subcommands to existing `weaver` commands**
rather than creating new top-level binaries — and **always wire build
flags through `scripts/build.sh`**.

## Gate as the validation contract

`scripts/build.sh gate` is the single source of truth for "did this
change pass". The 11 checks (in order):

Source of truth: `scripts/build.sh`, in the function dispatched by
`scripts/build.sh gate` (search for the comment `# 1. Workspace tests`
and walk down). Mirror this list — adding/removing items here without
updating the script (and vice versa) is a documented divergence and a
real bug. Comment-anchor pinning is preferred over line numbers because
the line numbers drift every time `build.sh` is touched.

1. `cargo test --workspace` — workspace test suite (hard fail).
2. `cargo build --release --bin weft --bin weaver` — release binaries
   for the daemon and orchestrator (hard fail).
3. WASI WASM — `cargo build --target wasm32-wasip2 --profile release-wasm
   -p clawft-wasm` (skipped if `wasm32-wasip2` is not installed).
4. Browser WASM: `clawft-types` — `cargo check --target
   wasm32-unknown-unknown --no-default-features --features browser`
   (soft fail; the script uses `cargo check`, not `cargo build`, to keep
   the gate fast).
5. Browser WASM: `clawft-platform` (same `cargo check` invocation, soft fail).
6. Browser WASM: `clawft-core` (same `cargo check` invocation, soft fail).
7. Browser WASM: `clawft-llm` (same `cargo check` invocation, soft fail).
8. Browser WASM: `clawft-tools` (same `cargo check` invocation, soft fail).
9. Browser WASM: `clawft-wasm` (same `cargo check` invocation, soft fail).
10. UI build — `(cd ui && npm run build)` (skipped if `ui/` is missing).
11. Voice feature — `cargo check --features voice -p clawft-plugin`
    (soft fail; tracks optional voice plugin compile).

Soft-fail steps record failure into the gate summary but do not abort
the gate; hard-fail steps abort. Treat any failure as a regression to
distill into the brain regardless of severity.

The gate is mandatory — do **not** skip it for "trivial" fixes. A node-3
verdict is invalid if the gate didn't run.

## Verdict policy

A trajectory is `pass` only if **all** of the following hold:

1. `scripts/build.sh gate` exits 0 (all 11 checks).
2. The browser bundle is still inside the 300KB / 120KB envelope.
3. No new `unsafe` block landed without a hand review.
4. The post-hoc score is strictly greater than the prior trajectory's
   score (no regressions).
5. ExoChain witness anchor written (when `feature = "exochain"` is on).

If any of those fails, the trajectory is recorded with `verdict = fail`
but **still** stored — failures train the bank too.

## Drift detection

The DEMOCRITUS two-tier loop (`feature = "ecc"`) drives drift detection:

| Tier | When | Cost |
|---|---|---|
| O(1) EML kernel | every cognitive tick (50ms) | `tick_budget_ratio = 0.3` |
| Lanczos fallback | when measured drift > threshold | `k` iterations × `m` graph edges |

If `kernel.cognitive.stats` reports rising `drift_count`, schedule a
fresh iteration of this loop. Do **not** drop `tick_interval_ms` below
20ms or raise `tick_budget_ratio` above 0.5.

## Versioning + reproducibility

- weftos is lockstep semver (ADR-001) — every trajectory records the
  workspace version.
- Each trajectory also records the ruvector workspace version it ran
  against. Cross-repo trajectories are un-replayable without both pins.
- The Attractor DOT graph at `.attractor/integration.dot` is the
  authoritative spec — when the implementation diverges, update the DOT
  first, then sync the code.

## Related skills

- `cross-repo-integration` — the topology this loop runs on top of.
- `weftos-build-deploy` — release engineering across all channels.
- `weftos-docs-deploy` — Fumadocs site deploy.
- `weftos-api-docs` — rustdoc API reference deploy.

## Forbidden actions

- Do **not** bypass `scripts/build.sh`. If a flag is missing, **add it
  to the script**, do not run raw `cargo`.
- Do **not** skip the gate at node 3 — the gate is the contract.
- Do **not** publish a single `clawft-*` crate independently. Lockstep
  semver assumes the workspace ships together.
- Do **not** commit `crates/clawft-wasm/www/.env-keys.json` (populated
  by `scripts/build.sh serve` and gitignored).
- Do **not** drop `tick_interval_ms` below 20ms; the substrate, GUI,
  and mesh heartbeats are loosely synchronized to the tick.
