# Phase K1: Supervisor + RBAC

**Phase ID**: K1
**Workstream**: W-KERNEL
**Duration**: Week 3-4
**Goal**: Implement agent supervisor with spawn/stop/restart lifecycle and per-agent capability-based access control

---

## S -- Specification

### What Changes

This phase adds an agent supervisor that manages the full lifecycle of agents (spawn, stop, restart, inspect) through the kernel's process table, and enforces per-agent capabilities (RBAC) on tool calls, IPC access, and resource consumption. The supervisor wraps the existing `AgentLoop` spawn mechanism without replacing it.

### Files to Create

| File | Purpose |
|---|---|
| `crates/clawft-kernel/src/supervisor.rs` | `AgentSupervisor` with spawn/stop/restart/inspect/watch |

### Files to Modify

| File | Change |
|---|---|
| `crates/clawft-kernel/src/capability.rs` | Add capability enforcement logic, `CapabilityChecker` |
| `crates/clawft-kernel/src/process.rs` | Add `set_capabilities()`, resource usage tracking hooks |
| `crates/clawft-kernel/src/lib.rs` | Re-export supervisor types |
| `crates/clawft-core/src/agent/loop_core.rs` | Add pre-tool-call hook point for capability checking |
| `crates/clawft-core/src/tools/registry.rs` | Add `filtered_tools()` method that applies capability filter |

### Key Types

**AgentSupervisor** (`supervisor.rs`):
```rust
pub struct AgentSupervisor<P: Platform> {
    process_table: Arc<ProcessTable>,
    kernel_ipc: Arc<KernelIpc>,
    default_capabilities: AgentCapabilities,
    _platform: PhantomData<P>,
}

pub struct SpawnRequest {
    pub agent_id: String,
    pub capabilities: Option<AgentCapabilities>,
    pub parent_pid: Option<Pid>,
    pub env: HashMap<String, String>,
}

pub struct SpawnResult {
    pub pid: Pid,
    pub agent_id: String,
}

impl<P: Platform> AgentSupervisor<P> {
    pub async fn spawn(&self, request: SpawnRequest) -> Result<SpawnResult>;
    pub async fn stop(&self, pid: Pid, graceful: bool) -> Result<()>;
    pub async fn restart(&self, pid: Pid) -> Result<SpawnResult>;
    pub async fn inspect(&self, pid: Pid) -> Result<ProcessEntry>;
    pub async fn watch(&self, pid: Pid) -> tokio::sync::watch::Receiver<ProcessState>;
    pub fn list_by_state(&self, state: ProcessState) -> Vec<ProcessEntry>;
}
```

**CapabilityChecker** (`capability.rs`):
```rust
pub struct CapabilityChecker {
    process_table: Arc<ProcessTable>,
}

impl CapabilityChecker {
    pub fn check_tool_access(&self, pid: Pid, tool_name: &str) -> Result<()>;
    pub fn check_ipc_target(&self, from_pid: Pid, to_pid: Pid) -> Result<()>;
    pub fn check_service_access(&self, pid: Pid, service_name: &str) -> Result<()>;
    pub fn check_resource_limit(&self, pid: Pid, resource: ResourceType) -> Result<()>;
}

pub enum ResourceType {
    Memory(u64),
    CpuTime(u64),
    OpenFiles(u32),
    ConcurrentTools(u32),
}
```

**Capability file format** (loaded via `--capabilities <path>`):
```json
{
  "sandbox": {
    "allow_shell": false,
    "allow_network": true,
    "allowed_paths": ["/workspace"],
    "denied_paths": ["/etc", "/root"]
  },
  "permissions": {
    "tools": ["read_file", "write_file", "search"],
    "deny_tools": ["shell_exec", "delete_file"]
  },
  "ipc_scope": {
    "type": "topic",
    "topics": ["build-status", "test-results"]
  },
  "resource_limits": {
    "max_memory_mb": 256,
    "max_cpu_seconds": 300,
    "max_open_files": 50,
    "max_concurrent_tools": 4
  },
  "service_access": ["memory", "cron"]
}
```

### CLI Commands

```
weft agent spawn --capabilities <path>   -- Spawn agent with specific capabilities
weft agent stop <pid>                     -- Gracefully stop agent by PID
weft agent restart <pid>                  -- Restart agent (preserves PID mapping)
weft agent inspect <pid>                  -- Show agent details, capabilities, resource usage
```

---

## P -- Pseudocode

### Agent Spawn

```
fn AgentSupervisor::spawn(request):
    // 1. Resolve capabilities
    caps = request.capabilities ?? self.default_capabilities

    // 2. Check resource limits (global)
    if process_table.count_running() >= kernel_config.max_processes:
        return Err(MaxProcessesReached)

    // 3. Allocate PID and create process entry
    pid = process_table.allocate_pid()
    entry = ProcessEntry {
        pid,
        agent_id: request.agent_id,
        state: Starting,
        capabilities: caps,
        resource_usage: ResourceUsage::zero(),
        cancel_token: CancellationToken::new(),
        parent_pid: request.parent_pid,
    }
    process_table.insert(entry)

    // 4. Create filtered tool registry
    filtered_tools = tool_registry.filtered_tools(caps.permissions)

    // 5. Spawn AgentLoop (reuse existing mechanism)
    agent_loop = AgentLoop::new(agent_config, platform, bus, pipeline, ...)
    tokio::spawn(async {
        process_table.update_state(pid, Running)
        result = agent_loop.run(cancel_token).await
        process_table.update_state(pid, Exited(result.code()))
    })

    return SpawnResult { pid, agent_id }
```

### Capability Check on Tool Call

```
fn CapabilityChecker::check_tool_access(pid, tool_name):
    entry = process_table.get(pid)?
    caps = entry.capabilities

    // Check deny list first (deny overrides allow)
    if tool_name in caps.permissions.deny_tools:
        return Err(ToolDenied(tool_name))

    // If allow list is non-empty, tool must be in it
    if caps.permissions.tools is not empty:
        if tool_name not in caps.permissions.tools:
            return Err(ToolNotAllowed(tool_name))

    // Check sandbox policy
    if tool_name == "shell_exec" and not caps.sandbox.allow_shell:
        return Err(ShellDenied)

    // Check concurrent tool limit
    if entry.resource_usage.concurrent_tools >= caps.resource_limits.max_concurrent_tools:
        return Err(ConcurrentToolLimitReached)

    Ok(())
```

### Graceful Stop

```
fn AgentSupervisor::stop(pid, graceful):
    entry = process_table.get(pid)?

    if graceful:
        // Signal cancellation, let agent finish current tool call
        process_table.update_state(pid, Stopping)
        entry.cancel_token.cancel()

        // Wait up to 30s for clean exit
        timeout(30s, wait_for_exit(pid)).await
        if still running:
            // Force kill
            drop(entry.cancel_token)
    else:
        // Immediate stop
        drop(entry.cancel_token)
        process_table.update_state(pid, Exited(-1))
```

---

## A -- Architecture

### Component Relationships

```
AgentSupervisor<P>
  |
  +-- ProcessTable (shared with Kernel)
  |     +-- ProcessEntry { capabilities, cancel_token }
  |
  +-- CapabilityChecker
  |     +-- reads ProcessTable capabilities
  |     +-- called by AgentLoop pre-tool hook
  |
  +-- AgentLoop (existing, created per spawn)
        +-- ToolRegistry.filtered_tools(capabilities)
        +-- pre_tool_call_hook -> CapabilityChecker
```

### Integration Points

1. **AgentLoop hook**: Add `pre_tool_call` hook in `loop_core.rs`. If a `CapabilityChecker` is set, it's called before every tool execution. This is a trait-object callback, not a hard dependency on `clawft-kernel`.

2. **ToolRegistry filtering**: `filtered_tools()` returns a view of the registry that excludes tools not in the agent's permission list. This is a new method on `ToolRegistry`, not a modification of existing methods.

3. **Existing SandboxEnforcer reuse**: The `SandboxPolicy` in `AgentCapabilities` is the same type from `clawft-plugin`. `SandboxEnforcer` continues to work as-is for filesystem sandboxing; capabilities add tool-level and resource-level access control on top.

### State Machine: Process Lifecycle

```
Starting --> Running --> Stopping --> Exited(0)
    |            |           |
    |            |           +-------> Exited(-1)  [force kill]
    |            |
    |            +--------> Suspended --> Running  [resume]
    |                           |
    |                           +-----> Stopping
    |
    +--------------------> Exited(1)  [boot failure]
```

### Ruvector Integration (Doc 07)

When the `ruvector-supervisor` feature gate is enabled, ruvector crates replace the
custom capability checking and resource tracking subsystems. The custom implementations
remain as fallbacks when the feature gate is disabled. See `07-ruvector-deep-integration.md`
for full adapter code.

| Custom Component | Ruvector Replacement | Feature Gate | Benefit |
|---|---|---|---|
| `CapabilityChecker` (binary Permit/Deny) | `cognitum-gate-tilezero::TileZero` (three-way Permit/Defer/Deny) | `ruvector-supervisor` | Defer enables escalation to supervisor or human; PermitTokens provide cryptographic proof |
| `ResourceUsage` (manual tracking) | `ruvector-cognitive-container::EpochController` | `ruvector-supervisor` | Budget system with `try_budget()`, `consume()`, partial results on exhaustion |
| (none -- new capability) | `rvf-crypto` witness receipts on Deny | `ruvector-crypto` | Every denied action gets a tamper-evident receipt in the witness chain |

**ExoChain references**: `exo-identity::Did` provides persistent agent identity across
restarts (DIDs survive PID reallocation). `exo-consent::Gatekeeper` trait maps directly
to the capability checking interface and can serve as an alternative to the TileZero gate.

Cross-reference: `07-ruvector-deep-integration.md`, Section 3 "Phase K1: Supervisor + RBAC".

---

## R -- Refinement

### Edge Cases

1. **Spawn with unknown capability file**: Return clear error with file path; don't use defaults silently
2. **Stop already-exited process**: Idempotent; return Ok with warning log
3. **Restart preserves PID mapping**: New agent gets new PID but supervisor tracks the restart lineage via `parent_pid`
4. **Capability hot-update**: Not supported in K1 (future work). Requires agent restart to change capabilities.
5. **Orphan processes**: When parent PID's agent exits, children continue running. Supervisor tracks but doesn't auto-kill (configurable in future)
6. **Resource limit exceeded during tool call**: Tool call is aborted, agent receives error result, agent continues (not killed)

### Backward Compatibility

- Existing `weft agent` commands continue to work. New `--capabilities` flag is optional
- `AgentLoop` works without capability checker (None case). Only when kernel is active does the checker get installed
- `ToolRegistry` existing methods unchanged; `filtered_tools()` is additive

### Error Handling

- `SupervisorError` enum with variants: `MaxProcessesReached`, `ProcessNotFound`, `InvalidStateTransition`, `CapabilityDenied`, `SpawnFailed`
- All errors include PID and agent_id for debugging
- Failed spawns clean up process table entry

---

## C -- Completion

### Exit Criteria

- [ ] `AgentSupervisor` spawns agent, creates `ProcessEntry` with capabilities
- [ ] `CapabilityChecker` blocks tool calls not in permission list
- [ ] `CapabilityChecker` blocks denied tools even if in allow list
- [ ] Graceful stop signals cancellation and waits for clean exit
- [ ] Force stop immediately terminates agent
- [ ] Restart creates new agent preserving capability configuration
- [ ] `weft agent inspect <pid>` shows capabilities and resource usage
- [ ] Resource limits are checked (memory, cpu, concurrent tools)
- [ ] `filtered_tools()` correctly excludes tools not in capabilities
- [ ] Pre-tool-call hook works in AgentLoop without kernel dependency
- [ ] Existing agent spawning works unchanged when kernel is disabled
- [ ] All workspace tests pass (`scripts/build.sh test`)
- [ ] Clippy clean (`scripts/build.sh clippy`)

### Testing Verification

```bash
# Supervisor unit tests
cargo test -p clawft-kernel -- supervisor

# Capability enforcement tests
cargo test -p clawft-kernel -- capability

# Integration: spawn with capabilities
cargo test -p clawft-kernel -- test_supervised_agent

# Regression check
scripts/build.sh test

# CLI smoke test
cargo run --bin weft -- agent spawn --capabilities test-caps.json
cargo run --bin weft -- kernel ps
cargo run --bin weft -- agent inspect 1
cargo run --bin weft -- agent stop 1
```
