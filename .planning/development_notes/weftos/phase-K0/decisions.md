# Phase K0: Kernel Foundation -- Development Notes

**Start**: 2026-03-01
**Status**: Complete
**Gate**: check + test + clippy = PASS

## Decisions

### 1. KernelConfig split across crates
**Problem**: KernelConfig must be in the root Config (clawft-types) but AgentCapabilities is defined in clawft-kernel, creating a circular dependency.

**Decision**: Define a minimal `KernelConfig` in `clawft-types/src/config/kernel.rs` with only serializable fields (enabled, max_processes, health_check_interval_secs). The kernel crate defines `KernelConfigExt` that wraps the base config and adds `AgentCapabilities`.

**Rationale**: Avoids circular dependency while keeping serde-compatible config in clawft-types where it belongs.

### 2. DashMap for concurrent collections
**Decision**: Added `dashmap = "6"` as a workspace dependency. Used for ProcessTable and ServiceRegistry.

**Rationale**: Lock-free concurrent reads are essential for a kernel that multiple subsystems query simultaneously. DashMap is well-established and provides the HashMap-like API needed.

### 3. ProcessTable PID allocation
**Decision**: PIDs are monotonically increasing from AtomicU64, starting at 1 (PID 0 reserved for the kernel process). PIDs are never reused within a session.

**Rationale**: Simplicity and debuggability. PID reuse adds complexity (race conditions with stale references) for no benefit in a single-session kernel.

### 4. Boot sequence owns AppContext
**Decision**: Kernel::boot() creates AppContext internally and holds it as `Option<AppContext<P>>`. The `take_app_context()` method allows extracting it for agent loop consumption.

**Rationale**: The kernel needs to extract Arc references (bus, tools, etc.) from AppContext before it's consumed by into_agent_loop(). Holding it as Option allows this two-phase pattern.

### 5. Console REPL is stubbed
**Decision**: Only boot event types (BootEvent, BootPhase, LogLevel) and output formatting are implemented. The interactive REPL loop is not implemented in K0.

**Rationale**: Interactive stdin handling with async is complex and requires careful signal handling. The boot event types provide immediate value for `weft kernel boot` output. The REPL can be added later.

### 6. Pre-existing clippy fixes
**Decision**: Fixed all pre-existing clippy issues in clawft-types, clawft-core, clawft-cli, and clawft-services as part of K0. These were collapsible-if, derivable-impls, match-like-matches-macro, map_or->is_none_or, and manual-pattern-char-comparison lints.

**Rationale**: The project requires clippy-clean builds. Fixing pre-existing issues is necessary for the gate to pass.

### 7. Two-layer cluster architecture
**Problem**: WeftOS must connect all ephemeral instances — native, browser, edge, IoT — but ruvector's distributed crates use `std::net::SocketAddr` which doesn't compile to WASM.

**Decision**: Two-layer architecture. `ClusterMembership` is a universal peer tracker that compiles on all platforms (uses `Option<String>` addresses and `NodePlatform` enum). `ClusterService` (ruvector-powered, native-only) wraps `ruvector_cluster::ClusterManager` behind `#[cfg(feature = "cluster")]` and syncs state into `ClusterMembership`. Browser/edge nodes join via WebSocket to a coordinator and get full cluster visibility.

**Rationale**: Keeps the universal layer WASM-compatible while still leveraging ruvector's tested coordination primitives (consistent hashing, discovery, shard routing) on native coordinator nodes.

### 8. Ruvector-core feature-gating
**Problem**: All 3 ruvector distributed crates (`ruvector-cluster`, `ruvector-raft`, `ruvector-replication`) list `ruvector-core` as a dependency, but none of them import from it. `ruvector-core` pulls in heavy deps (hnsw, quantization) that bloat the build and are not needed for coordination primitives.

**Decision**: Feature-gate `ruvector-core` as optional behind a `vector-store` feature in all 3 crates. Pushed to `weave-logic-ai/ruvector` fork.

**Rationale**: Allows using the coordination primitives (cluster, consensus, replication) without pulling in the entire vector store stack. When vector operations are needed, enable `vector-store`.

### 9. ClusterMembership always present
**Decision**: `ClusterMembership` is a field on `Kernel<P>` and is always created during boot, even without the `cluster` feature. It tracks the local node at minimum.

**Rationale**: All kernel subsystems and CLI commands can query cluster state uniformly. Without `cluster` feature, the membership contains only the local node. With `cluster`, the `ClusterService` syncs discovered native nodes into it. Browser nodes that join via WS also get registered here.

## What Was Skipped

1. ~~**Ruvector integration**~~ -- **Done**: `ClusterMembership` (universal) + `ClusterService` (native, feature-gated) integrated. `weaver cluster` CLI added. See decisions 7-9 above.
2. ~~**Exo-resource-tree**~~ -- **Done**: `exo-resource-tree` crate created with core types, tree CRUD, Merkle recomputation, and namespace bootstrap. Integrated into kernel boot behind `exochain` feature gate. CLI commands `weaver resource {tree, inspect, stats}` added. See decisions 13 below.
3. **Interactive console REPL** -- only event types and formatting implemented
4. ~~**CronService wrapper**~~ -- **Done**: `CronService` registered at boot as a `SystemService`. See `crates/clawft-kernel/src/cron.rs` and boot.rs step 5b.

## Files Created

| File | Lines | Purpose |
|------|-------|---------|
| `crates/clawft-kernel/Cargo.toml` | 46 | Crate manifest |
| `crates/clawft-kernel/src/lib.rs` | 47 | Crate root with re-exports |
| `crates/clawft-kernel/src/boot.rs` | ~330 | Kernel struct, boot/shutdown, state machine |
| `crates/clawft-kernel/src/process.rs` | ~280 | ProcessTable, ProcessEntry, PID allocation |
| `crates/clawft-kernel/src/service.rs` | ~200 | SystemService trait, ServiceRegistry |
| `crates/clawft-kernel/src/ipc.rs` | ~190 | KernelIpc, KernelMessage, MessageTarget |
| `crates/clawft-kernel/src/capability.rs` | ~200 | AgentCapabilities, IpcScope, ResourceLimits |
| `crates/clawft-kernel/src/health.rs` | ~200 | HealthSystem, HealthStatus, OverallHealth |
| `crates/clawft-kernel/src/console.rs` | ~200 | BootEvent, BootPhase, LogLevel, BootLog |
| `crates/clawft-kernel/src/config.rs` | ~60 | KernelConfigExt (wraps types-level config) |
| `crates/clawft-kernel/src/error.rs` | ~55 | KernelError, KernelResult |
| `crates/clawft-types/src/config/kernel.rs` | ~75 | KernelConfig (base, in types crate) |
| `crates/clawft-cli/src/commands/kernel_cmd.rs` | ~240 | CLI kernel subcommand |

## Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` (workspace) | Added clawft-kernel member, dashmap workspace dep |
| `crates/clawft-types/src/config/mod.rs` | Added kernel module, KernelConfig field to Config |
| `crates/clawft-types/src/config/voice.rs` | Fixed derivable-impls clippy lints |
| `crates/clawft-core/src/agent/loop_core.rs` | Fixed collapsible-if clippy lints |
| `crates/clawft-core/src/agent/verification.rs` | Fixed collapsible-if and char comparison lints |
| `crates/clawft-services/src/api/delegation.rs` | Fixed map_or->is_none_or lint |
| `crates/clawft-cli/src/main.rs` | Added Kernel subcommand |
| `crates/clawft-cli/src/commands/mod.rs` | Added kernel_cmd module |
| `crates/clawft-cli/src/commands/agent.rs` | Fixed collapsible-if lint |
| `crates/clawft-cli/src/commands/gateway.rs` | Fixed collapsible-if lints |
| `crates/clawft-cli/src/help_text.rs` | Added kernel help topic |
| `crates/clawft-cli/Cargo.toml` | Added clawft-kernel dependency |

### Decision 10: Exochain Fork Strategy

**Context**: ExoChain upstream uses `serde_cbor` (unmaintained) and has
native-only deps (libp2p, graphql) that block WASM compilation.

**Decision**: Fork to weave-logic-ai/exochain. Replace serde_cbor with
ciborium. Feature-gate native-only crates.

**Rationale**: Minimal diff from upstream, ciborium is the maintained
successor, feature gates preserve functionality for native builds.

### Decision 11: Local Chain First, Global Chain Deferred

**Context**: Full exochain includes local + global root chain with
cross-node consensus.

**Decision**: K0 implements local chain only (chain_id=0). Global root
chain (chain_id=1) deferred to K6 when ruvector-raft consensus is
integrated.

**Rationale**: Local chain provides immediate value (audit trail) without
distributed consensus complexity. K6 timeline aligns with networking
maturity.

### Decision 12: RVF ExoChain Segment Types (0x40-0x42)

**Context**: ExoChain events need persistent storage in RVF format.

**Decision**: Reserve segment types 0x40 (ExochainEvent), 0x41
(ExochainCheckpoint), 0x42 (ExochainProof) in rvf-types. 64-byte
ExoChainHeader prefix with CBOR payload.

**Rationale**: Consistent with existing RVF segment architecture.
64-byte header provides efficient fixed-offset parsing. CBOR payload
preserves schema flexibility.

### Decision 13: ERT K0 Scope — Tree CRUD and Merkle Only

**Context**: The exo-resource-tree spec (Doc 13) covers tree CRUD,
permission engine, delegation certificates, and CLI integration.

**Decision**: K0 implements tree CRUD, Merkle recomputation, bootstrap,
and mutation log. Permission engine (check()) stubs to Allow. Delegation
is type-only. CLI covers tree/inspect/stats.

**Rationale**: Tree structure is prerequisite for all permission checks.
Permission engine requires capability model (K1). Stubs allow K1 to
enable gradually without breaking K0 code.

### Decision 14: TreeManager Facade

**Context**: Three subsystems (ChainManager, ResourceTree, MutationLog) existed as
islands — ChainManager appended hash-linked events, ResourceTree stored nodes with
Merkle hashes, MutationLog recorded DAG-backed mutations, but none communicated.
Boot created both chain and tree but didn't log boot phases to chain. Service
registration had `register_with_tree()` but it didn't create chain events.

**Decision**: Create `TreeManager` facade (`crates/clawft-kernel/src/tree_manager.rs`)
that holds all three subsystems behind a unified API. Every tree mutation atomically:
modifies tree → appends chain event → stores `chain_seq` metadata on the node →
appends MutationEvent to log. This ensures two-way traceability: tree nodes know
their chain event sequence number, chain events reference tree paths in their payload.

**Rationale**: Guarantees that the tree and chain can never drift out of sync. The
facade pattern keeps the individual subsystems simple while enforcing the invariant
that every mutation is auditable. Enables `weaver chain verify` to prove integrity
and `weaver resource inspect` to show chain provenance for each node.

### Decision 15: Boot-to-Chain Audit Trail

**Context**: Kernel boot executed multiple phases (init, config, services, cluster,
tree bootstrap) but none of these were recorded in the hash chain. The chain only
contained events appended after boot by explicit user actions.

**Decision**: Log each boot phase as a chain event during startup:
- `boot.init` — kernel version
- `boot.config` — max_processes, health_interval
- `boot.services` — registered service count
- `tree.bootstrap` — node count, root hash (via TreeManager)
- `boot.cluster` — node_id
- `boot.ready` — elapsed_ms, tree_root_hash, process/service counts

Additionally, shutdown emits `kernel.shutdown` with tree_root_hash, chain_seq,
and tree_nodes, then creates a chain checkpoint.

**Rationale**: Makes the entire kernel lifecycle auditable. `weaver chain local`
now shows a complete boot trace from genesis through ready state. Shutdown
checkpointing enables future K1 work on persistent state recovery.

## Test Summary

- 66 new tests in clawft-kernel (process table, service registry, health, IPC, boot, console, config, capability)
- 8 new tests in tree_manager (bootstrap, insert, remove, update_meta, register_service, stats, checkpoint, chain integrity)
- 1 new test in chain (verify_integrity)
- 2 new tests in kernel_cmd (format_bytes, args parsing)
- All pre-existing tests continue to pass

## K0 Completion

**Gate passed**: 2026-03-01
- `scripts/build.sh check` -- PASS
- `scripts/build.sh test` -- PASS (all workspace tests)
- `scripts/build.sh clippy` -- PASS (zero warnings)
- Manual verification of exochain boot/chain/tree/checkpoint -- PASS

K1 (Supervisor + RBAC + ExoChain Integration) started immediately after K0 gate.

## K1 Decisions (continued)

### Decision 16: cognitum-gate-tilezero Integration (K1, not deferred)

**Context**: K1 plan originally deferred TileZero to K2. User explicitly pulled it into K1:
"we do not want to defer this."

**Decision**: Implement `TileZeroGate` in `gate.rs` behind `#[cfg(feature = "tilezero")]`.
The adapter wraps `Arc<TileZero>` + optional `Arc<ChainManager>`, maps our gate parameters
to `ActionContext`, bridges async `TileZero::decide()` to sync `GateBackend::check()` via
`tokio::task::block_in_place`, and logs gate.permit/gate.defer/gate.deny chain events.

**Rationale**: TileZero is the coherence arbiter for the ruvector ecosystem. Having it wired
as an alternative `GateBackend` means permission decisions can be based on TileZero's
256-tile coherence model (three stacked filters: Structural/Shift/Evidence) rather than just
binary capability checks. Ed25519-signed PermitTokens and blake3-chained WitnessReceipts
provide cryptographic auditability.

### Decision 17: SHAKE-256 Chain Hashing (rvf-crypto)

**Context**: Chain.rs used raw SHA-256 (`sha2::Sha256`) and critically did NOT include
the payload in `compute_event_hash()`. This meant payloads could be swapped without breaking
the hash chain — a real integrity vulnerability. User requested proper hashing using the
ruvector ecosystem's crypto primitives.

**Decision**: Replace `sha2` with `rvf-crypto` (SHAKE-256). Add `payload_hash` field to
`ChainEvent`. The hash scheme is now:

```
payload_hash = SHAKE-256(canonical_json_bytes(payload)) | [0; 32] if None
hash = SHAKE-256(sequence ‖ chain_id ‖ prev_hash ‖ source ‖ 0x00 ‖ kind ‖ 0x00 ‖ timestamp ‖ payload_hash)
```

Null-byte separators between source/kind prevent domain collisions.

**Rationale**:
1. **Payload integrity**: Every chain event now commits to its payload content
2. **2-way verification**: Given an event, verify chain link (prev_hash) AND content (payload_hash) independently
3. **Ecosystem alignment**: SHAKE-256 is the canonical hash for RVF witness chains
4. **Cross-service verification**: Two services producing the same event get the same payload_hash
5. **Domain separation**: Null-byte separators prevent "foo" + "bar.baz" colliding with "foo.bar" + "baz"

### Decision 18: bootstrap_fresh adds /kernel/agents

**Context**: K1 added `register_agent()` which inserts nodes under `/kernel/agents/`,
but `bootstrap_fresh()` in exo-resource-tree only created 7 namespaces (missing /kernel/agents).
This caused ParentNotFound errors in tree_manager tests.

**Decision**: Add `/kernel/agents` to `bootstrap_fresh()` (now 8 namespaces + root = 9 nodes).

**Rationale**: Agent nodes are first-class kernel resources. The namespace must exist at
bootstrap time, consistent with `/kernel/services` and `/kernel/processes`.

### Decision 19: Unified SHAKE-256 Merkle Hashing (exo-resource-tree)

**Context**: After migrating chain.rs from SHA-256 to SHAKE-256 (Decision 17), the
exo-resource-tree crate still used `sha2::Sha256` for Merkle hash computation. This created
a mixed-hash ecosystem where tree root hashes (SHA-256) flowed into chain event payloads
(SHAKE-256), breaking the single-hash-family invariant.

**Decision**: Replace `sha2` with `rvf-crypto` in exo-resource-tree. `recompute_merkle()`
now uses `rvf_crypto::hash::shake256_256` — the same primitive used by chain.rs and the
RVF witness chain.

**Hash scheme**:
```
node_hash = SHAKE-256(sorted_child_hashes || sorted_metadata_kv_bytes)
```

**Rationale**:
1. **Ecosystem coherence**: One hash function (SHAKE-256) across chain events, Merkle tree,
   and RVF witness chains
2. **Cross-verification**: Tree root hash in a chain event can be independently recomputed
   using the same primitive — no hash-family translation needed
3. **rvf-crypto canonical**: SHAKE-256 is the ecosystem standard per rvf-crypto's public API
4. **No backward compat needed**: K0 chain data is ephemeral (no persistent Merkle hashes
   survived across sessions pre-K1 chain persistence)

### Decision 20: NodeScoring — Trust/Performance Vectors at Base Node Level

**Context**: WeftOS needs comparable trust/performance data across all entities (agents,
services, namespaces, devices) for optimal path selection, concurrent task comparison
("gamification"), and training data generation.

**Decision**: Embed a 6-dimensional `NodeScoring` vector (24 bytes) directly on every
`ResourceNode`: trust, performance, difficulty, reward, reliability, velocity. All
dimensions are f32 in [0.0, 1.0], defaulting to 0.5 (neutral). The scoring bytes flow
into the SHAKE-256 Merkle hash (`child_hashes || scoring_bytes || metadata_kv`), making
scores tamper-evident. `recompute_all()` aggregates children's scoring into parents via
reward-weighted mean. TreeManager provides `update_scoring()`, `blend_scoring()` (EMA),
`find_similar()` (cosine similarity ranking), and `rank_by_score()` (weighted linear).
`MutationEvent::UpdateScoring` records old/new scoring for full audit trail.

**Rationale**:
1. **Universality**: Every node type inherits scoring — no per-type special cases
2. **Tamper evidence**: Scoring flows into Merkle hash, so modifying scores without
   updating the tree is detectable
3. **Aggregation**: Parent nodes automatically reflect aggregate child performance
4. **Compact**: 24 bytes per node — minimal memory and serialization overhead
5. **No new deps**: Uses only std f32 ops and existing serde; no external crates added
6. **Training-ready**: Scores are on-chain via MutationEvent + ChainEvent for ML pipelines

**Deferred to K2**: Supervisor exit-triggered scoring blend, gate decision trust nudges,
`weaver resource score/rank` CLI commands and daemon RPC handlers.
