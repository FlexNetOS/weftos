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

## What Was Skipped

1. **Ruvector integration** -- feature-gated with TODO comments in boot.rs
2. **Exo-resource-tree** -- feature-gated with TODO comments in boot.rs
3. **Interactive console REPL** -- only event types and formatting implemented
4. **CronService wrapper** -- no built-in services registered at boot (ServiceRegistry starts empty)

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

## Test Summary

- 66 new tests in clawft-kernel (process table, service registry, health, IPC, boot, console, config, capability)
- 2 new tests in kernel_cmd (format_bytes, args parsing)
- All pre-existing tests continue to pass
