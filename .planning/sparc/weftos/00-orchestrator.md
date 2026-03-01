# SPARC Orchestrator: WeftOS Kernel Workstream

**Workstream ID**: W-KERNEL
**Date**: 2026-02-28
**Status**: Planning
**Estimated Duration**: 17+ weeks (K0-K6 + addenda)
**Source Analysis**: `.planning/development_notes/openfang-comparison.md`, existing kernel primitives audit

---

## 1. Workstream Summary

Compose clawft's existing kernel primitives (MessageBus, SandboxEnforcer, PermissionResolver, CronService, AgentLoop, Platform traits) into a first-class OS abstraction. WeftOS wraps the framework in a kernel layer, enabling process management, per-agent RBAC, agent-to-agent IPC, WASM tool sandboxing, container integration, and an application framework.

### CLI Naming: `weave` + `weft`

Two entry points following the textile metaphor:

- **`weft`** (existing) = the threads woven through. Agent/virtual layer: spawn agents, send IPC, manage tools, run apps. This is the user-facing agent interface.
- **`weave`** (new) = the act of combining on the loom. OS/physical layer: kernel, process table, services, resource tree, networking, environments. This is the system administration interface.

Both binaries link to the same `clawft-cli` crate. `weave` is a thin alias that sets `CLI_MODE=os` so subcommand routing knows which namespace to expose. Existing `weft` commands are 100% backward compatible.

### Approach

- **New crate, not new framework**: `clawft-kernel` composes existing crates; no rewrites
- **Kernel wraps AppContext**: The `Kernel<P>` struct owns an `AppContext<P>` and adds process table, service registry, IPC, and capability enforcement
- **Incremental adoption**: Each phase adds a kernel subsystem; existing `weft` commands continue to work unchanged
- **Dual CLI**: OS commands via `weave`, agent commands via `weft` (see Section 12)

### Crate Scope

| In Scope (new + modified) | Consumed (read-only) |
|---|---|
| `clawft-kernel` (new) | `clawft-types` |
| `clawft-core` (supervisor hooks) | `clawft-platform` |
| `clawft-cli` (new commands) | `clawft-plugin` |
| `clawft-services` (IPC wiring) | `clawft-security` |
| | `clawft-llm` |
| | `clawft-channels` |

---

## 2. Phase Summary

| Phase | ID | Title | Goal | Duration |
|---|---|---|---|---|
| 0 | K0 | Kernel Foundation | New `clawft-kernel` crate with boot, process table, service registry, health | 2 weeks |
| 1 | K1 | Supervisor + RBAC | Agent supervisor with spawn/stop/restart, per-agent capabilities | 2 weeks |
| 2 | K2 | A2A IPC | Agent-to-agent messaging, pub/sub topics, JSON-RPC wire format | 2 weeks |
| 3 | K3 | WASM Sandbox | Wasmtime tool execution, fuel metering, memory limits | 2 weeks |
| 4 | K4 | Containers | Alpine image, sidecar service orchestration | 1 week |
| 5 | K5 | App Framework | Application manifests, lifecycle, external framework interop | 2 weeks |
| 6 | K6 | Distributed Fabric | Multi-node cluster, cross-node IPC, cryptographic filesystem, governance | 6 weeks |

---

## 3. Dependencies

### Internal Dependencies (Phase-to-Phase)

```
K0 (Foundation) -- no deps
  |
  +---> K1 (Supervisor/RBAC) -- depends on K0 (process table, capabilities)
  |         |
  |         +---> K2 (A2A IPC) -- depends on K1 (agent PIDs, capability checks)
  |                   |
  |                   +---> K5 (App Framework) -- depends on K1+K2
  |                             |
  |                             +---> K6 (Distributed Fabric) -- depends on K0+K1+K2+K5
  |
  +---> K3 (WASM Sandbox) -- depends on K0 (service registry)  [parallel w/ K1]
  |
  +---> K4 (Containers) -- depends on K0 (service registry)    [parallel w/ K1]
```

### Parallelization Opportunities

- **K1, K3, K4** can run in parallel after K0 completes
- **K2** requires K1 (needs process table PIDs for message routing)
- **K5** requires K1+K2 (apps spawn agents, agents communicate)

### External Dependencies

- **Wasmtime** (K3): `wasmtime` crate for WASM tool execution
- **Bollard** (K4): Docker API for container management (optional, behind feature flag)
- **No other workstream blocking**: W-KERNEL is self-contained

### New Cargo Dependencies

| Dependency | Version | Scope | Introduced In |
|---|---|---|---|
| `wasmtime` | 27.0 | `clawft-kernel` (optional, `wasm-sandbox` feature) | K3 |
| `bollard` | 0.17 | `clawft-kernel` (optional, `containers` feature) | K4 |
| `dashmap` | 6.0 | `clawft-kernel` | K0 |
| `exo-core` | 0.1 | `exo-resource-tree` | K0 |
| `exo-identity` | 0.1 | `exo-resource-tree` | K0 |
| `exo-consent` | 0.1 | `exo-resource-tree` | K0 |
| `exo-dag` | 0.1 | `exo-resource-tree` | K0 |

---

## 4. Interface Contracts

### 4.1 Kernel Trait (`boot.rs`)

```rust
pub struct Kernel<P: Platform> {
    state: KernelState,
    app_context: Option<AppContext<P>>,
    process_table: Arc<ProcessTable>,
    service_registry: Arc<ServiceRegistry>,
    ipc: Arc<KernelIpc>,
    health: HealthSystem,
}

pub enum KernelState {
    Booting,
    Running,
    ShuttingDown,
    Halted,
}
```

### 4.2 Process Table (`process.rs`)

```rust
pub type Pid = u64;

pub struct ProcessTable {
    next_pid: AtomicU64,
    entries: DashMap<Pid, ProcessEntry>,
}

pub struct ProcessEntry {
    pub pid: Pid,
    pub agent_id: String,
    pub state: ProcessState,
    pub capabilities: AgentCapabilities,
    pub resource_usage: ResourceUsage,
    pub cancel_token: CancellationToken,
    pub parent_pid: Option<Pid>,
}

pub enum ProcessState {
    Starting,
    Running,
    Suspended,
    Stopping,
    Exited(i32),
}
```

### 4.3 Service Registry (`service.rs`)

```rust
pub trait SystemService: Send + Sync {
    fn name(&self) -> &str;
    fn service_type(&self) -> ServiceType;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn health_check(&self) -> HealthStatus;
}

pub enum ServiceType {
    Core,       // MessageBus, MemoryStore
    Plugin,     // PluginHost, ChannelAdapter
    Cron,       // CronService
    Api,        // Axum server
    Custom(String),
}
```

### 4.4 Capability System (`capability.rs`)

```rust
pub struct AgentCapabilities {
    pub sandbox: SandboxPolicy,
    pub permissions: UserPermissions,
    pub ipc_scope: IpcScope,
    pub resource_limits: ResourceLimits,
    pub service_access: Vec<String>,
}

pub enum IpcScope {
    None,
    Explicit(Vec<Pid>),
    Topic(Vec<String>),
    All,
}

pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_seconds: u64,
    pub max_open_files: u32,
    pub max_concurrent_tools: u32,
}
```

### 4.5 IPC Protocol (`ipc.rs`)

```rust
pub struct KernelMessage {
    pub id: String,
    pub from: Pid,
    pub to: MessageTarget,
    pub payload: MessagePayload,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
}

pub enum MessageTarget {
    Pid(Pid),
    Topic(String),
    Broadcast,
    Service(String),
}

pub enum MessagePayload {
    Text(String),
    Json(serde_json::Value),
    ToolCall { name: String, args: serde_json::Value },
    ToolResult { call_id: String, result: serde_json::Value },
    Signal(ProcessSignal),
}
```

---

## 5. File Ownership Matrix

### K0: Kernel Foundation

| File | Action | Owner |
|---|---|---|
| `crates/clawft-kernel/Cargo.toml` | Create | K0 |
| `crates/clawft-kernel/src/lib.rs` | Create | K0 |
| `crates/clawft-kernel/src/boot.rs` | Create | K0 |
| `crates/clawft-kernel/src/process.rs` | Create | K0 |
| `crates/clawft-kernel/src/service.rs` | Create | K0 |
| `crates/clawft-kernel/src/ipc.rs` | Create | K0 |
| `crates/clawft-kernel/src/capability.rs` | Create | K0 |
| `crates/clawft-kernel/src/health.rs` | Create | K0 |
| `crates/clawft-kernel/src/config.rs` | Create | K0 |
| `Cargo.toml` (workspace) | Modify | K0 |
| `crates/clawft-types/src/config/mod.rs` | Modify | K0 |
| `crates/clawft-cli/src/main.rs` | Modify | K0 |
| `crates/clawft-cli/src/commands/mod.rs` | Modify | K0 |
| `crates/clawft-cli/src/help_text.rs` | Modify | K0 |
| `docs/architecture/adr-028-weftos-kernel.md` | Create | K0 |

### K1: Supervisor + RBAC

| File | Action | Owner |
|---|---|---|
| `crates/clawft-kernel/src/supervisor.rs` | Create | K1 |
| `crates/clawft-kernel/src/capability.rs` | Modify | K1 |
| `crates/clawft-kernel/src/process.rs` | Modify | K1 |
| `crates/clawft-core/src/agent/loop_core.rs` | Modify | K1 |
| `crates/clawft-core/src/tools/registry.rs` | Modify | K1 |

### K2: A2A IPC

| File | Action | Owner |
|---|---|---|
| `crates/clawft-kernel/src/a2a.rs` | Create | K2 |
| `crates/clawft-kernel/src/topic.rs` | Create | K2 |
| `crates/clawft-kernel/src/ipc.rs` | Modify | K2 |
| `crates/clawft-services/src/delegation/mod.rs` | Modify | K2 |
| `crates/clawft-services/src/mcp/server.rs` | Modify | K2 |

### K3: WASM Sandbox

| File | Action | Owner |
|---|---|---|
| `crates/clawft-kernel/src/wasm_runner.rs` | Create | K3 |
| `crates/clawft-kernel/Cargo.toml` | Modify | K3 |
| `crates/clawft-core/src/tools/registry.rs` | Modify | K3 |
| `crates/clawft-core/src/agent/sandbox.rs` | Modify | K3 |

### K4: Containers

| File | Action | Owner |
|---|---|---|
| `crates/clawft-kernel/src/container.rs` | Create | K4 |
| `crates/clawft-kernel/Dockerfile.alpine` | Create | K4 |
| `crates/clawft-kernel/docker-compose.yml` | Create | K4 |
| `crates/clawft-kernel/src/service.rs` | Modify | K4 |

### K5: App Framework

| File | Action | Owner |
|---|---|---|
| `crates/clawft-kernel/src/app.rs` | Create | K5 |
| `crates/clawft-cli/src/main.rs` | Modify | K5 |

### K6: Distributed Fabric

| File | Action | Owner |
|---|---|---|
| `crates/clawft-kernel/src/cluster.rs` | Create | K6 |
| `crates/clawft-kernel/src/cross_node_ipc.rs` | Create | K6 |
| `crates/clawft-kernel/src/filesystem.rs` | Create | K6 |
| `crates/clawft-kernel/src/environment.rs` | Create | K6 |
| `crates/clawft-kernel/src/learning_loop.rs` | Create | K6 |
| `crates/clawft-kernel/src/governance.rs` | Create | K6 |
| See `08-ephemeral-os-architecture.md` for complete file list | | |

### Resource Tree (Doc 13)

| File | Action | Owner |
|---|---|---|
| `crates/exo-resource-tree/Cargo.toml` | Create | K0 |
| `crates/exo-resource-tree/src/lib.rs` | Create | K0 |
| `crates/exo-resource-tree/src/tree.rs` | Create | K0 |
| `crates/exo-resource-tree/src/permission.rs` | Create | K1 |
| `crates/exo-resource-tree/src/delegation.rs` | Create | K1 |
| `crates/clawft-kernel/src/boot.rs` | Modify | K0 |
| `crates/clawft-kernel/src/capability.rs` | Modify | K1 |

---

## 6. Testing Contracts

### Unit Tests (per module)

| Phase | Module | Tests |
|---|---|---|
| K0 | `boot.rs` | Boot sequence state transitions (Booting -> Running -> ShuttingDown -> Halted) |
| K0 | `process.rs` | Process table CRUD, PID allocation, state transitions |
| K0 | `service.rs` | Service registration, lookup, lifecycle |
| K0 | `health.rs` | Health check aggregation, degraded state detection |
| K1 | `supervisor.rs` | Agent spawn, stop, restart, resource limit enforcement |
| K1 | `capability.rs` | Capability check pass/fail, IPC scope filtering |
| K2 | `a2a.rs` | Message routing, delivery confirmation, timeout handling |
| K2 | `topic.rs` | Subscribe, unsubscribe, publish, wildcard matching |
| K3 | `wasm_runner.rs` | WASM load, execute, fuel exhaustion, memory limit |
| K4 | `container.rs` | Container service start/stop lifecycle |
| K5 | `app.rs` | Manifest parsing, app lifecycle state machine |
| K0 | `tree.rs` | ResourceNode CRUD, parent-child relationships, Merkle root computation |
| K1 | `permission.rs` | Permission check walk, delegation cert shortcut, policy evaluation |
| K1 | `delegation.rs` | DelegationCert creation, expiration, revocation |

### Integration Tests

| Phase | Test | Description |
|---|---|---|
| K0 | `kernel_boot` | Full boot sequence with CronService and PluginHost registered |
| K1 | `supervised_agent` | Spawn agent with capabilities, verify tool calls filtered by RBAC |
| K2 | `agent_messaging` | Two agents exchange messages, verify IPC scope enforcement |
| K2 | `topic_pubsub` | Agent publishes to topic, subscribers receive, non-subscribers don't |
| K3 | `wasm_tool_sandbox` | Tool executes in WASM sandbox, verify isolation from host filesystem |
| K5 | `app_lifecycle` | Install -> start -> stop application, verify clean shutdown |
| K0 | `resource_tree_boot` | Kernel boots with resource tree, services registered as tree nodes |
| K1 | `tree_permission_check` | Agent spawn creates tree leaf, permission check enforces RBAC |

### Phase Gate Checks (automated)

| # | Check | Command |
|---|---|---|
| 1 | Workspace builds | `scripts/build.sh check` |
| 2 | All tests pass | `scripts/build.sh test` |
| 3 | Clippy clean | `scripts/build.sh clippy` |
| 4 | WASM check | `cargo check -p clawft-wasm --target wasm32-unknown-unknown --features browser` |
| 5 | Docs build | `cargo doc --no-deps` |

---

## 7. Risk Mitigation Strategy

| ID | Risk | Severity | Mitigation |
|---|---|---|---|
| R1 | `AppContext::into_agent_loop()` consumes context, blocking kernel wrapping | High | Extract and Arc-wrap shared services before consumption (same pattern as API layer) |
| R2 | Wasmtime binary size bloats native CLI | Medium | Feature-gate behind `wasm-sandbox`; not in default features |
| R3 | Process table contention under high agent count | Medium | Use `DashMap` for lock-free concurrent access |
| R4 | IPC message storms between agents | Medium | Per-agent rate limiting in IpcScope; backpressure via bounded channels |
| R5 | RBAC capability checks add latency to every tool call | Low | Capability bitmask for fast path; full check only for elevated permissions |
| R6 | Container integration requires Docker daemon | Low | Feature-gated behind `containers`; graceful error when Docker unavailable |
| R7 | Breaking changes to existing agent spawning | High | Supervisor wraps existing `AgentLoop::spawn()` without changing its interface |
| R8 | Circular dependency between kernel and core | Medium | Kernel depends on core; core does not depend on kernel. Kernel hooks via trait objects. |

---

## 8. Definition of Done

### Code Quality
- [ ] All public types have `///` doc comments
- [ ] No `#[allow(unused)]` except with documented reason
- [ ] All `unwrap()` / `expect()` calls have justification comments
- [ ] Clippy passes with `--deny warnings`

### Testing
- [ ] Unit test coverage for all new modules
- [ ] Integration tests for cross-module interactions
- [ ] All existing workspace tests pass (zero regressions)
- [ ] Phase gate script passes for each phase before merge

### Documentation
- [ ] ADR-028 written and linked from `docs/architecture/`
- [ ] Per-phase `decisions.md` in `.planning/development_notes/weftos/phase-K{N}/`
- [ ] Kernel guide at `docs/guides/kernel.md` (created in K5)
- [ ] All rustdoc builds without warnings

### Security
- [ ] Capability checks enforced at system boundaries
- [ ] WASM sandbox prevents filesystem escape
- [ ] IPC scope prevents unauthorized inter-agent communication
- [ ] No secrets in kernel config defaults

---

## 9. Agent Spawning Execution Plan

### Team Structure

| Agent | Type | Phases | Responsibility |
|---|---|---|---|
| `kernel-lead` | `coder` | K0-K5 | Kernel crate creation, boot sequence, process table |
| `rbac-agent` | `coder` | K1 | Supervisor, capability enforcement |
| `ipc-agent` | `coder` | K2 | A2A messaging, topic pub/sub |
| `sandbox-agent` | `coder` | K3 | WASM tool runner, fuel metering |
| `container-agent` | `coder` | K4 | Container integration, Dockerfile |
| `app-agent` | `coder` | K5 | Application framework, manifest parsing |
| `test-agent` | `tester` | K0-K5 | Integration tests, phase gate verification |
| `review-agent` | `reviewer` | K0-K5 | Code review, security audit per phase |
| `cluster-agent` | `coder` | K6 | Node fabric, service discovery, cross-node IPC |
| `governance-agent` | `coder` | K6 | Environment scoping, CGR integration, learning loop |
| `fs-agent` | `coder` | K6 | Cryptographic filesystem, storage backends |
| `inference-agent` | `coder` | K0, K5 | Inference service agent, model registry, training pipeline |
| `network-agent` | `coder` | K6 | Network service, pairing protocol, client gateways |
| `resource-tree-agent` | `coder` | K0-K1 | exo-resource-tree integration, permission engine |

### Execution Order

1. **Week 1-2**: `kernel-lead` executes K0 (foundation)
2. **Week 3-4**: `rbac-agent` (K1) + `sandbox-agent` (K3) + `container-agent` (K4) in parallel
3. **Week 5-6**: `ipc-agent` (K2) -- requires K1 complete
4. **Week 7-8**: K3 and K4 wrap up if not complete
5. **Week 9-10**: `app-agent` (K5) -- requires K1+K2 complete
6. **Week 11**: Integration testing, documentation, final review
7. `test-agent` and `review-agent` run continuously after each phase
8. **Week 12-17**: K6 agents execute distributed fabric (see 08-ephemeral-os-architecture.md)

---

## 10. Quality Gates Between Phases

### K0 -> K1 Gate
- [ ] `Kernel<P>` boots successfully with empty process table
- [ ] At least one `SystemService` registered (CronService wrapper)
- [ ] `weave kernel status` CLI command works
- [ ] ADR-028 committed

### K1 -> K2 Gate
- [ ] Agent spawn creates `ProcessEntry` with capabilities
- [ ] Tool calls filtered by `AgentCapabilities`
- [ ] `weave kernel ps` shows running agents
- [ ] Supervisor restart recovers crashed agent

### K2 -> K5 Gate
- [ ] Agent-to-agent message delivery works
- [ ] Topic pub/sub routes messages correctly
- [ ] IPC scope prevents unauthorized messaging
- [ ] `weft ipc send` CLI command works

### K3 Gate (independent)
- [ ] WASM tool loads and executes
- [ ] Fuel exhaustion terminates execution cleanly
- [ ] Memory limit prevents allocation bomb
- [ ] Host filesystem not accessible from sandbox

### K4 Gate (independent)
- [ ] Alpine container image builds
- [ ] Sidecar service starts/stops with kernel
- [ ] Container health checks propagate to kernel health

### K5 Final Gate
- [ ] Application manifest parsed and validated
- [ ] App install/start/stop lifecycle works
- [ ] App agents spawn with correct capabilities
- [ ] `weft app list` shows installed applications
- [ ] Full phase gate script passes

### K6 Gate (Distributed Fabric)
- [ ] Two-node cluster forms and discovers peers
- [ ] Cross-node IPC delivers messages via DID addressing
- [ ] Cryptographic filesystem creates and retrieves entries
- [ ] Environment-scoped governance enforces different risk thresholds
- [ ] Learning loop records trajectories and extracts patterns
- [ ] Browser node joins cluster via WebSocket

---

## 11. Key Reusable Code (no rewrites)

| What | File | Reused In |
|---|---|---|
| `AppContext::new()` | `crates/clawft-core/src/bootstrap.rs:104` | K0 boot.rs |
| `MessageBus` | `crates/clawft-core/src/bus.rs:29` | K0 ipc.rs |
| `AgentLoop` + CancellationToken | `crates/clawft-core/src/agent/loop_core.rs:130` | K1 supervisor.rs |
| `SandboxEnforcer` | `crates/clawft-core/src/agent/sandbox.rs:32` | K1 capability.rs |
| `SandboxPolicy` | `crates/clawft-plugin/src/sandbox.rs` | K1 capability.rs |
| `PermissionResolver` | `crates/clawft-core/src/pipeline/permissions.rs` | K1 capability.rs |
| `UserPermissions` | `crates/clawft-types/src/routing.rs` | K1 capability.rs |
| `CronService` | `crates/clawft-services/src/cron_service/` | K0 service.rs |
| `PluginHost` | `crates/clawft-channels/src/host.rs` | K0 service.rs |
| `ToolRegistry` | `crates/clawft-core/src/tools/registry.rs` | K1, K3 |
| `PluginSandbox` | `crates/clawft-wasm/src/sandbox.rs` | K3 wasm_runner.rs |
| `DelegationEngine` | `crates/clawft-services/src/delegation/mod.rs` | K2 a2a.rs |
| `MCP server/client` | `crates/clawft-services/src/mcp/` | K2 a2a.rs |
| Container tools | `crates/clawft-plugin-containers/src/lib.rs` | K4 container.rs |
| ClawHub | `crates/clawft-services/src/clawhub/` | K5 (future marketplace) |
| ExoChain crates | `exo-core`, `exo-identity`, `exo-consent`, `exo-dag` | K0 resource tree |

---

## 12. CLI Commands

### Naming Convention

| Binary | Layer | Metaphor |
|--------|-------|----------|
| `weave` | OS / physical | The loom that holds the structure |
| `weft` | Agent / virtual | The threads woven through |

**Rule of thumb**: if it manages the OS substrate (kernel, processes, services, resources, network, environments), use `weave`. If it manages agents, tools, IPC messages, or apps, use `weft`.

### `weave` -- OS Layer (new)

```
# Kernel (K0)
weave kernel status        -- kernel state, boot phases, uptime
weave kernel services      -- list system services with health
weave kernel ps            -- list agent process table

# Resource Tree (K0-K1, Doc 13)
weave resource tree        -- show resource tree
weave resource inspect <id>  -- node details + policies
weave resource grant <id> <did> <role>  -- add delegation cert
weave resource revoke <id> <did>        -- remove delegation
weave resource check <id> <did> <action> -- test permission

# Network (K6, Doc 12)
weave network peers        -- list paired/bonded peers
weave network pair <addr>  -- initiate pairing handshake
weave network bond <did>   -- bond with trusted peer
weave network discover     -- run capability discovery

# Environment (K6, Doc 09)
weave env list             -- list environments
weave env create <name> <class>  -- create environment
weave env switch <name>    -- switch active environment

# Sessions (Doc 12)
weave session list         -- list active sessions
weave session attach <id>  -- join existing session
weave session kill <id>    -- terminate session
```

### `weft` -- Agent Layer (existing, unchanged)

```
# Agent management (K1)
weft agent spawn --capabilities <path>
weft agent stop <pid>
weft agent restart <pid>
weft agent inspect <pid>

# IPC (K2)
weft ipc send <pid> <message>
weft ipc topics
weft ipc subscribe <pid> <topic>

# Applications (K5)
weft app install <path>
weft app start <name>
weft app stop <name>
weft app list
weft app inspect <name>

# Tools (existing)
weft tools list
weft tools search <query>
weft tools info <name>
```

---

## 13. Documentation Plan

| Phase | Document | Location |
|---|---|---|
| K0 | ADR-028: WeftOS Kernel Architecture | `docs/architecture/adr-028-weftos-kernel.md` |
| K0 | Phase K0 decisions | `.planning/development_notes/weftos/phase-K0/decisions.md` |
| K1 | Phase K1 decisions | `.planning/development_notes/weftos/phase-K1/decisions.md` |
| K2 | Phase K2 decisions | `.planning/development_notes/weftos/phase-K2/decisions.md` |
| K3 | Phase K3 decisions | `.planning/development_notes/weftos/phase-K3/decisions.md` |
| K4 | Phase K4 decisions | `.planning/development_notes/weftos/phase-K4/decisions.md` |
| K5 | Phase K5 decisions | `.planning/development_notes/weftos/phase-K5/decisions.md` |
| K5 | Kernel developer guide | `docs/guides/kernel.md` |
| All | Rustdoc on public types | In-source `///` comments |
| All | Help text | `crates/clawft-cli/src/help_text.rs` |

---

## 14. New Crate Structure

```
crates/clawft-kernel/
  Cargo.toml
  src/
    lib.rs           -- crate root, re-exports
    boot.rs          -- Kernel boot sequence (wraps AppContext)
    process.rs       -- Process table (PID tracking)
    ipc.rs           -- Extended IPC (agent-to-agent)
    service.rs       -- Service registry (SystemService trait)
    capability.rs    -- Per-agent capability scopes
    health.rs        -- Health checks
    config.rs        -- KernelConfig types
    supervisor.rs    -- Agent supervisor (K1)
    a2a.rs           -- A2A protocol (K2)
    topic.rs         -- Pub/sub topics (K2)
    wasm_runner.rs   -- Wasmtime tool execution (K3)
    container.rs     -- Sidecar management (K4)
    app.rs           -- Application manifest (K5)
```

```
crates/exo-resource-tree/
  Cargo.toml
  src/
    lib.rs           -- crate root, ResourceTree struct
    tree.rs          -- Tree operations, HashMap + parent index
    node.rs          -- ResourceId, ResourceKind, ResourceNode
    delegation.rs    -- DelegationCert, Role
    permission.rs    -- Permission check engine, ACL cache
    merkle.rs        -- Subtree Merkle root computation
```

---

## 15. Addendum Documents

These addendum specifications extend the core K0-K5 phases with advanced capabilities.
They are numbered 07+ and represent extensions that build on the core kernel.

| Document | Title | Relationship |
|----------|-------|-------------|
| `07-ruvector-deep-integration.md` | Deep ruvector Integration | Replaces reimplementation with ruvector crate dependencies across K0-K5 |
| `08-ephemeral-os-architecture.md` | Ephemeral OS Architecture | Extends kernel to distributed multi-tenant OS with K6 phase proposal |
| `09-environments-and-learning.md` | Environment-Scoped Governance & Self-Learning | Dev/staging/prod environments with self-learning governance loop |
| `10-agent-first-single-user.md` | Agent-First Single-User Architecture | Redefines all services as agents, introduces .agent.toml manifests, supervisor-first boot |
| `11-local-inference-agents.md` | Local Inference & Continuous Model Improvement | 4-tier model routing, GGUF inference via ruvllm, continuous improvement lifecycle |
| `12-networking-and-pairing.md` | Networking, Pairing, and Network Joining | DeFi-inspired networking, pairing handshake, trust/bonding, client access layer |
| `13-exo-resource-tree.md` | Exo-Resource-Tree: Hierarchical Resource Namespace | Unified resource tree on ExoChain substrate, everything-is-a-node, RBAC via tree walk |

### Integration with Core Phases

- **K0-K5**: Core single-node kernel (this orchestrator)
- **Doc 07**: Overlay -- enhances K0-K5 with ruvector crate integrations
- **Doc 08**: Extension -- adds K6 (Distributed Fabric) as new phase
- **Doc 09**: Extension -- adds environment scoping and learning loop to K1, K5, K6
- **Doc 10**: Overlay -- redefines K0 boot as agent-first, K1 supervisor manages agent processes
- **Doc 11**: Extension -- adds inference-service agent (K0 boot), training-service (K5), model sync (K6)
- **Doc 12**: Extension -- adds network-service agent (K6), client access gateways, session management
- **Doc 13**: Foundation -- provides the unified namespace tree that all kernel concepts (processes, services, IPC, capabilities) map to
