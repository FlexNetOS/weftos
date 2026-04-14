# ExoChain & Governance Compliance Audit

**Date**: 2026-04-04  
**Scope**: All `.rs` files in `clawft-kernel`, `clawft-graphify`, `clawft-core`, `clawft-weave`  
**Method**: Search for state-modifying operations, cross-reference against `chain_manager`/`ChainLoggable`/`chain.append` (ExoChain) and `GovernanceGate`/`EffectVector`/`governance.evaluate` (Governance)

## Legend

- **ExoChain?** — Does this file log state changes to the ExoChain?
- **Governance?** — Does this file check governance before mutations?
- **Risk**: Critical (auth/security boundary), High (data isolation), Medium (unaudited mutations), Low (internal bookkeeping)

## Already Covered (Good)

These files properly log to ExoChain and/or check governance:

| File | What | ExoChain | Governance |
|------|------|----------|------------|
| supervisor.rs | Service lifecycle (start/stop/restart) | YES | NO (acceptable — service ops are kernel-internal) |
| tree_manager.rs | Resource tree CRUD | YES | NO (tree mutations flow through governance at caller level) |
| dead_letter.rs | Dead letter queue events | YES | NO (reactive logging) |
| a2a.rs | Agent-to-agent messaging | YES | NO |
| governance.rs | Governance evaluation | YES (via ChainLoggable) | YES (is the governance engine) |
| chain.rs | Chain itself | YES | N/A |
| daemon.rs (weave) | RPC command dispatch | YES (many operations) | YES (governance gate on tool execution) |
| cognitive_tick.rs | DEMOCRITUS loop | YES (newly wired — EML events) | NO (EML is advisory, not a mutation gate) |
| console.rs | Boot logging | YES | NO |
| ipc.rs | IPC message delivery | YES (references chain) | NO |

## Gaps Found

### clawft-kernel

| File | Action | ExoChain? | Governance? | Risk |
|------|--------|-----------|-------------|------|
| hnsw_service.rs:insert | Insert vector embedding | NO | NO | **Medium** — unaudited vector mutations; embeddings influence search results |
| hnsw_service.rs:clear | Clear all vectors | NO | NO | **High** — bulk data destruction with no audit trail |
| hnsw_service.rs:save_to_file | Persist HNSW to disk | NO | NO | Low — data export, but no audit of when state was serialized |
| hnsw_service.rs:load_from_file | Load HNSW from disk | NO | NO | Low — could load stale/tampered data without chain record |
| profile_store.rs:create_profile | Create data isolation profile | NO | NO | **High** — unaudited data isolation boundary creation |
| profile_store.rs:delete_profile | Delete profile and all data | NO | NO | **Critical** — bulk data deletion with no audit trail |
| profile_store.rs:switch_profile | Switch active profile | NO | NO | **High** — changes which data is visible/writable |
| profile_store.rs:insert | Insert vector into profile | NO | NO | **Medium** — unaudited vector mutations within profile scope |
| causal.rs:add_node | Add causal graph node | NO | NO | **Medium** — graph mutations affect coherence analysis |
| causal.rs:remove_node | Remove causal graph node | NO | NO | **Medium** — graph mutations affect coherence analysis |
| causal.rs:link | Create causal edge | NO | NO | **Medium** — graph topology changes are unaudited |
| causal.rs:unlink | Remove causal edge | NO | NO | **Medium** — graph topology changes are unaudited |
| causal.rs:clear | Clear entire causal graph | NO | NO | **High** — bulk graph destruction |
| artifact_store.rs:store | Store content-addressed artifact | NO | NO | **Medium** — artifact ingestion is unaudited |
| artifact_store.rs:remove | Remove artifact | NO | NO | **Medium** — artifact deletion is unaudited |
| artifact_store.rs:release | Release artifact reference | NO | NO | Low — reference counting bookkeeping |
| cron.rs:add_job | Schedule recurring job | NO | NO | **High** — unaudited job scheduling could execute arbitrary actions |
| cron.rs:remove_job | Remove scheduled job | NO | NO | **Medium** — unaudited schedule modification |
| cron.rs:tick | Execute due jobs | NO | NO | **Medium** — job execution lacks audit trail |
| config_service.rs:set | Write config key-value | NO | NO | **High** — configuration changes alter system behavior |
| config_service.rs:delete | Delete config key | NO | NO | **High** — configuration deletion alters system behavior |
| config_service.rs:set_secret | Store encrypted secret | NO | NO | **Critical** — secret lifecycle is unaudited |
| auth_service.rs:register_credential | Register auth credential | NO | NO | **Critical** — credential registration has no audit trail |
| auth_service.rs:rotate_credential | Rotate credential | NO | NO | **Critical** — credential rotation is unaudited |
| auth_service.rs:request_token | Issue auth token | NO | NO | **Critical** — token issuance has no audit trail |
| auth_service.rs:revoke_token | Revoke auth token | NO | NO | **High** — token revocation is unaudited |
| auth_service.rs:authenticate | Authenticate user | NO | NO | **High** — auth attempts are not logged to chain |
| environment.rs:register | Register environment | NO | NO | **Medium** — environment creation alters governance scope |
| environment.rs:set_active | Switch active environment | NO | NO | **High** — changes governance risk thresholds |
| environment.rs:remove | Remove environment | NO | NO | **Medium** — environment deletion |
| container.rs:start_container | Start sidecar container | NO | NO | **Medium** — container lifecycle is unaudited |
| container.rs:stop_container | Stop sidecar container | NO | NO | **Medium** — container lifecycle is unaudited |
| container.rs:configure | Configure container | NO | NO | **Medium** — container config changes |
| app.rs:install | Install application | NO | NO | **High** — app installation changes system capabilities |
| app.rs:remove | Remove application | NO | NO | **High** — app removal |
| app.rs:transition_to | Change app state | NO | NO | **Medium** — app lifecycle transitions |
| app.rs:start | Start application agents | NO | NO | **High** — spawns agent processes |
| app.rs:stop | Stop application | NO | NO | **Medium** — stops agent processes |
| cluster.rs:add_peer | Add cluster peer | NO | NO | **High** — trust boundary expansion |
| cluster.rs:remove_peer | Remove cluster peer | NO | NO | **High** — trust boundary contraction |
| cluster.rs:update_state | Update peer state | NO | NO | **Medium** — peer state changes |
| process.rs:insert | Register process | NO | NO | **Medium** — process table mutation |
| process.rs:remove | Deregister process | NO | NO | **Medium** — process table mutation |
| process.rs:update_state | Change process state | NO | NO | **Medium** — process lifecycle |
| capability.rs:request_elevation | Request capability elevation | NO | NO | **High** — privilege escalation request is unaudited |
| agency.rs:add_child/remove_child | Agent hierarchy mutation | NO | NO | **Medium** — agent spawn tree changes |
| eml_coherence.rs:train | EML model training | YES (newly wired) | NO | Low — addressed by this sprint |
| eml_kernel.rs:train (all 6) | EML model training | YES (newly wired) | NO | Low — addressed by this sprint |
| hnsw_eml.rs:train_all | HNSW EML training | NO | NO | Low — internal model optimization |
| embedding.rs:embed | Generate embeddings | NO | NO | Low — stateless transform |
| persistence.rs:save/load | Kernel state persistence | NO | NO | **Medium** — state serialization boundary |
| topic.rs:publish/subscribe | Pub-sub messaging | NO | NO | Low — transient messaging |
| timer.rs:schedule | Schedule timer | NO | NO | Low — internal scheduling |
| heartbeat.rs:beat | Heartbeat emission | NO | NO | Low — health signaling |
| metrics.rs:record | Metrics recording | NO | NO | Low — telemetry |
| monitor.rs:check | Health monitoring | NO | NO | Low — read-only checks |
| reconciler.rs:reconcile | State reconciliation | NO | NO | **Medium** — auto-corrective mutations |
| wasm_runner/runner.rs:execute | Run WASM module | NO | NO | **High** — arbitrary code execution |
| wasm_runner/tools_fs.rs:* | Filesystem ops in sandbox | NO | NO | **Medium** — file mutations from WASM |
| wasm_runner/registry.rs:register | Register WASM tool | NO | YES (references governance) | **Medium** — partially covered |
| mesh.rs:* | Mesh peer management | NO | NO | **Medium** — network topology changes |
| mesh_service.rs:register/deregister | Mesh service registry | NO | NO | **Medium** — service advertisement |
| mesh_artifact.rs:store/fetch | Cross-node artifact sync | NO | NO | **Medium** — distributed state |
| mesh_ipc.rs:send | Cross-node messaging | NO | NO | **Medium** — inter-node communication |
| mesh_bootstrap.rs:bootstrap | Initial peer discovery | NO | NO | Low — startup-only |
| revocation.rs:revoke | Revoke capability | YES (EVENT_KIND_CAPABILITY_REVOKED) | NO | Already covered |

### clawft-graphify

| File | Action | ExoChain? | Governance? | Risk |
|------|--------|-----------|-------------|------|
| build.rs:build_graph | Build knowledge graph | NO | NO | **Medium** — graph construction from source |
| ingest.rs:ingest | Ingest source files | NO | NO | **Medium** — data ingestion pipeline |
| pipeline.rs:run | Run analysis pipeline | NO | NO | **Medium** — orchestrates analysis |
| cache.rs:store/evict | Cache management | NO | NO | Low — transient cache |
| hooks.rs:register/fire | Hook registration/execution | NO | NO | **Medium** — extensibility point |
| watch.rs:watch | File watcher setup | NO | NO | Low — observation only |
| eml_models.rs:train (all 4) | EML training | NO | NO | Low — model optimization |
| cluster.rs:detect_communities | Community detection | NO | NO | Low — read-only analysis |
| export/html.rs:export | HTML export | NO | NO | Low — read-only export |

### clawft-core

| File | Action | ExoChain? | Governance? | Risk |
|------|--------|-----------|-------------|------|
| agent/sandbox.rs:execute | Sandbox command execution | NO | NO | **High** — arbitrary command execution in sandbox |
| agent/loop_core.rs:run | Agent execution loop | NO | NO | **Medium** — agent lifecycle |
| session.rs:create/destroy | Session management | NO | NO | **Medium** — session lifecycle |
| pipeline/mutation.rs:apply | Apply mutations | NO | NO | **Medium** — state mutations |
| pipeline/permissions.rs:check | Permission checks | NO | NO | Low — read-only checks |
| tools/registry.rs:register | Tool registration | NO | NO | **Medium** — extends agent capabilities |
| workspace/mod.rs:create | Workspace creation | NO | NO | **Medium** — workspace lifecycle |
| workspace/config.rs:update | Workspace config | NO | NO | **Medium** — config changes |
| embeddings/hnsw_store.rs:insert | HNSW vector insert | NO | NO | **Medium** — vector mutations |
| embeddings/rvf_io.rs:write | RVF segment writing | YES (has chain.append) | NO | Already covered |
| policy_kernel.rs:evaluate | Policy evaluation | NO | NO | Low — read-only evaluation |

### clawft-weave

| File | Action | ExoChain? | Governance? | Risk |
|------|--------|-----------|-------------|------|
| commands/init_cmd.rs:init | Project initialization | NO | NO | **Medium** — creates project structure |
| commands/bench_eml.rs:* | Benchmark scoring | NO | NO | Low — benchmarking only |

## Summary Statistics

| Category | Count |
|----------|-------|
| **Critical** gaps (auth, secrets, data deletion) | 5 |
| **High** gaps (trust boundaries, config, apps) | 16 |
| **Medium** gaps (general state mutations) | 30+ |
| **Low** gaps (internal bookkeeping) | 15+ |
| Already covered | 10+ files |

## Top Priority Fixes

1. **auth_service.rs** — All credential and token operations MUST log to ExoChain. This is a security audit requirement.
2. **profile_store.rs** — Profile creation/deletion/switching crosses data isolation boundaries. Must log.
3. **config_service.rs** — Configuration changes (especially secrets) MUST be audited.
4. **hnsw_service.rs:clear** — Bulk data destruction must be audited.
5. **cron.rs** — Job scheduling is an execution vector; must be audited.
6. **cluster.rs** — Peer add/remove crosses trust boundaries; must be audited.
7. **app.rs** — App install/remove/start changes system capabilities; must be audited.
8. **capability.rs:request_elevation** — Privilege escalation requests must be logged.
9. **wasm_runner/runner.rs** — WASM execution should be logged (complements shell.exec logging).
10. **environment.rs:set_active** — Changing the active environment changes governance thresholds.

## Governance Gate Coverage

Only **2 locations** in the entire codebase actually evaluate governance:

1. `governance.rs` — The governance engine itself
2. `daemon.rs` — The RPC handler checks governance before tool execution

All other state mutations bypass governance entirely. This means an agent with IPC access can modify config, credentials, profiles, etc. without any governance check.

## Recommendations

1. Instrument the top-10 priority files with `chain_manager.append()` calls at each state-modifying public method.
2. Add governance gates to `auth_service.rs`, `config_service.rs`, `profile_store.rs`, and `capability.rs:request_elevation`.
3. Consider a `#[chain_logged]` proc macro to enforce audit logging at compile time for new state-modifying functions.
4. Add a CI check that flags new `pub fn` methods containing `insert`, `delete`, `create`, `remove`, `store`, `write`, `update`, `revoke`, `set` that lack chain logging.
