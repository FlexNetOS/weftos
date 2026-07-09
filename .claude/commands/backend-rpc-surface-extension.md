---
name: backend-rpc-surface-extension
description: Workflow command scaffold for backend-rpc-surface-extension in weftos.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /backend-rpc-surface-extension

Use this workflow when working on **backend-rpc-surface-extension** in `weftos`.

## Goal

Add a new backend RPC method (e.g., substrate.list, node.register, control.set_enabled) and wire it through the daemon, protocol, and tests.

## Common Files

- `crates/clawft-kernel/src/substrate_service.rs`
- `crates/clawft-kernel/src/node_registry.rs`
- `crates/clawft-weave/src/daemon.rs`
- `crates/clawft-weave/src/protocol.rs`
- `crates/clawft-weave/tests/*.rs`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Implement new method in kernel/service (e.g., substrate_service.rs, node_registry.rs, control.rs).
- Add protocol types and wire handler in daemon.rs.
- Update protocol.rs with params/result structs.
- Add or update integration/unit tests (tests/*.rs).
- Optionally update GUI or VSCode extension to use new RPC.

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.