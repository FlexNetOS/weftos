//! Agent supervisor for process lifecycle management.
//!
//! The [`AgentSupervisor`] manages the full lifecycle of kernel-managed
//! agents: spawn, stop, restart, inspect, and watch. It wraps the
//! existing `AgentLoop` spawn mechanism without replacing it, adding
//! capability enforcement, resource tracking, and process table integration.

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use clawft_platform::Platform;

use crate::capability::AgentCapabilities;
use crate::error::{KernelError, KernelResult};
use crate::ipc::KernelIpc;
use crate::process::{Pid, ProcessEntry, ProcessState, ProcessTable, ResourceUsage};

/// Request to spawn a new supervised agent process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRequest {
    /// Unique identifier for the agent.
    pub agent_id: String,

    /// Capabilities to assign. If `None`, the supervisor's default
    /// capabilities are used.
    #[serde(default)]
    pub capabilities: Option<AgentCapabilities>,

    /// PID of the parent process (for tracking spawn lineage).
    #[serde(default)]
    pub parent_pid: Option<Pid>,

    /// Environment variables for the agent.
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Result of a successful agent spawn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResult {
    /// The PID assigned to the new process.
    pub pid: Pid,

    /// The agent identifier.
    pub agent_id: String,
}

/// Manages the lifecycle of kernel-managed agent processes.
///
/// The supervisor sits between the CLI/API surface and the core
/// `AgentLoop`, providing:
///
/// - **Spawn**: creates a process entry, assigns capabilities,
///   allocates a PID, and tracks the agent in the process table.
/// - **Stop**: signals cancellation (graceful) or immediate termination.
/// - **Restart**: stops then re-spawns with the same configuration.
/// - **Inspect**: returns full process entry with capabilities and
///   resource usage.
/// - **Watch**: returns a receiver for process state changes.
///
/// The supervisor does not own the actual `AgentLoop` execution; that
/// remains the responsibility of the caller (kernel boot or CLI).
/// Instead, the supervisor manages the process table entries and
/// provides the cancellation tokens that control agent lifecycle.
pub struct AgentSupervisor<P: Platform> {
    process_table: Arc<ProcessTable>,
    kernel_ipc: Arc<KernelIpc>,
    default_capabilities: AgentCapabilities,
    _platform: PhantomData<P>,
}

impl<P: Platform> AgentSupervisor<P> {
    /// Create a new agent supervisor.
    ///
    /// # Arguments
    ///
    /// * `process_table` - Shared process table (also held by Kernel)
    /// * `kernel_ipc` - IPC subsystem for sending lifecycle signals
    /// * `default_capabilities` - Capabilities assigned to agents that
    ///   don't specify their own
    pub fn new(
        process_table: Arc<ProcessTable>,
        kernel_ipc: Arc<KernelIpc>,
        default_capabilities: AgentCapabilities,
    ) -> Self {
        Self {
            process_table,
            kernel_ipc,
            default_capabilities,
            _platform: PhantomData,
        }
    }

    /// Spawn a new supervised agent process.
    ///
    /// This creates a process table entry and returns the assigned PID.
    /// The actual agent execution (AgentLoop) must be started separately
    /// by the caller using the returned `SpawnResult` and the
    /// cancellation token from the process entry.
    ///
    /// # Errors
    ///
    /// Returns `KernelError::ProcessTableFull` if the process table
    /// has reached its maximum capacity.
    pub fn spawn(&self, request: SpawnRequest) -> KernelResult<SpawnResult> {
        let caps = request
            .capabilities
            .unwrap_or_else(|| self.default_capabilities.clone());

        info!(
            agent_id = %request.agent_id,
            parent_pid = ?request.parent_pid,
            "spawning supervised agent"
        );

        let entry = ProcessEntry {
            pid: 0, // Will be set by insert()
            agent_id: request.agent_id.clone(),
            state: ProcessState::Starting,
            capabilities: caps,
            resource_usage: ResourceUsage::default(),
            cancel_token: CancellationToken::new(),
            parent_pid: request.parent_pid,
        };

        let pid = self.process_table.insert(entry)?;

        debug!(pid, agent_id = %request.agent_id, "agent spawned");

        Ok(SpawnResult {
            pid,
            agent_id: request.agent_id,
        })
    }

    /// Stop a supervised agent process.
    ///
    /// If `graceful` is true, the process is moved to `Stopping` state
    /// and its cancellation token is cancelled, allowing the agent to
    /// finish its current work. If `graceful` is false, the process is
    /// immediately moved to `Exited(-1)`.
    ///
    /// Stopping an already-exited process is idempotent and returns `Ok`.
    ///
    /// # Errors
    ///
    /// Returns `KernelError::ProcessNotFound` if the PID is not in
    /// the process table.
    pub fn stop(&self, pid: Pid, graceful: bool) -> KernelResult<()> {
        let entry = self
            .process_table
            .get(pid)
            .ok_or(KernelError::ProcessNotFound { pid })?;

        // Already exited -- idempotent
        if matches!(entry.state, ProcessState::Exited(_)) {
            warn!(pid, "stop called on already-exited process");
            return Ok(());
        }

        if graceful {
            info!(pid, "gracefully stopping agent");
            // Transition to Stopping, then cancel the token
            let _ = self.process_table.update_state(pid, ProcessState::Stopping);
            entry.cancel_token.cancel();
        } else {
            info!(pid, "force stopping agent");
            entry.cancel_token.cancel();
            let _ = self
                .process_table
                .update_state(pid, ProcessState::Exited(-1));
        }

        Ok(())
    }

    /// Restart a supervised agent process.
    ///
    /// Stops the existing process (gracefully), then spawns a new one
    /// with the same agent_id and capabilities. The new process gets
    /// a fresh PID; the old entry remains in the table with
    /// `Exited(0)` state.
    ///
    /// The `parent_pid` of the new process is set to the restarted
    /// PID, creating a restart lineage.
    ///
    /// # Errors
    ///
    /// Returns `KernelError::ProcessNotFound` if the PID is not in
    /// the process table.
    pub fn restart(&self, pid: Pid) -> KernelResult<SpawnResult> {
        let old_entry = self
            .process_table
            .get(pid)
            .ok_or(KernelError::ProcessNotFound { pid })?;

        info!(pid, agent_id = %old_entry.agent_id, "restarting agent");

        // Stop the old process
        self.stop(pid, true)?;

        // Mark as cleanly exited if not already
        if !matches!(old_entry.state, ProcessState::Exited(_)) {
            let _ = self
                .process_table
                .update_state(pid, ProcessState::Exited(0));
        }

        // Spawn replacement with same config
        let request = SpawnRequest {
            agent_id: old_entry.agent_id.clone(),
            capabilities: Some(old_entry.capabilities.clone()),
            parent_pid: Some(pid),
            env: HashMap::new(),
        };

        self.spawn(request)
    }

    /// Inspect a supervised agent process.
    ///
    /// Returns a clone of the full [`ProcessEntry`] including
    /// capabilities and resource usage.
    ///
    /// # Errors
    ///
    /// Returns `KernelError::ProcessNotFound` if the PID is not in
    /// the process table.
    pub fn inspect(&self, pid: Pid) -> KernelResult<ProcessEntry> {
        self.process_table
            .get(pid)
            .ok_or(KernelError::ProcessNotFound { pid })
    }

    /// List processes filtered by state.
    pub fn list_by_state(&self, state: ProcessState) -> Vec<ProcessEntry> {
        self.process_table
            .list()
            .into_iter()
            .filter(|e| e.state == state)
            .collect()
    }

    /// List all running agent processes (excludes kernel PID 0).
    pub fn list_agents(&self) -> Vec<ProcessEntry> {
        self.process_table
            .list()
            .into_iter()
            .filter(|e| e.pid != 0)
            .collect()
    }

    /// Get a reference to the shared process table.
    pub fn process_table(&self) -> &Arc<ProcessTable> {
        &self.process_table
    }

    /// Get a reference to the IPC subsystem.
    pub fn ipc(&self) -> &Arc<KernelIpc> {
        &self.kernel_ipc
    }

    /// Get the default capabilities assigned to new agents.
    pub fn default_capabilities(&self) -> &AgentCapabilities {
        &self.default_capabilities
    }

    /// Count running processes (excluding kernel PID 0).
    pub fn running_count(&self) -> usize {
        self.process_table
            .list()
            .iter()
            .filter(|e| e.pid != 0 && e.state == ProcessState::Running)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clawft_core::bus::MessageBus;

    fn make_supervisor() -> AgentSupervisor<clawft_platform::NativePlatform> {
        let process_table = Arc::new(ProcessTable::new(16));
        let bus = Arc::new(MessageBus::new());
        let ipc = Arc::new(KernelIpc::new(bus));
        AgentSupervisor::new(process_table, ipc, AgentCapabilities::default())
    }

    fn simple_request(agent_id: &str) -> SpawnRequest {
        SpawnRequest {
            agent_id: agent_id.to_owned(),
            capabilities: None,
            parent_pid: None,
            env: HashMap::new(),
        }
    }

    #[test]
    fn spawn_creates_process_entry() {
        let sup = make_supervisor();
        let result = sup.spawn(simple_request("agent-1")).unwrap();

        assert!(result.pid > 0);
        assert_eq!(result.agent_id, "agent-1");

        let entry = sup.inspect(result.pid).unwrap();
        assert_eq!(entry.agent_id, "agent-1");
        assert_eq!(entry.state, ProcessState::Starting);
    }

    #[test]
    fn spawn_uses_default_capabilities() {
        let sup = make_supervisor();
        let result = sup.spawn(simple_request("agent-1")).unwrap();

        let entry = sup.inspect(result.pid).unwrap();
        assert!(entry.capabilities.can_spawn);
        assert!(entry.capabilities.can_ipc);
        assert!(entry.capabilities.can_exec_tools);
    }

    #[test]
    fn spawn_uses_custom_capabilities() {
        let sup = make_supervisor();
        let caps = AgentCapabilities {
            can_spawn: false,
            can_ipc: false,
            can_exec_tools: true,
            can_network: true,
            ..Default::default()
        };

        let request = SpawnRequest {
            agent_id: "restricted".to_owned(),
            capabilities: Some(caps.clone()),
            parent_pid: None,
            env: HashMap::new(),
        };

        let result = sup.spawn(request).unwrap();
        let entry = sup.inspect(result.pid).unwrap();
        assert!(!entry.capabilities.can_spawn);
        assert!(!entry.capabilities.can_ipc);
        assert!(entry.capabilities.can_network);
    }

    #[test]
    fn spawn_with_parent_pid() {
        let sup = make_supervisor();
        let parent = sup.spawn(simple_request("parent")).unwrap();

        let request = SpawnRequest {
            agent_id: "child".to_owned(),
            capabilities: None,
            parent_pid: Some(parent.pid),
            env: HashMap::new(),
        };

        let result = sup.spawn(request).unwrap();
        let entry = sup.inspect(result.pid).unwrap();
        assert_eq!(entry.parent_pid, Some(parent.pid));
    }

    #[test]
    fn spawn_fails_when_table_full() {
        let process_table = Arc::new(ProcessTable::new(2));
        let bus = Arc::new(MessageBus::new());
        let ipc = Arc::new(KernelIpc::new(bus));
        let sup: AgentSupervisor<clawft_platform::NativePlatform> =
            AgentSupervisor::new(process_table, ipc, AgentCapabilities::default());

        sup.spawn(simple_request("a1")).unwrap();
        sup.spawn(simple_request("a2")).unwrap();
        let result = sup.spawn(simple_request("a3"));
        assert!(result.is_err());
    }

    #[test]
    fn stop_graceful() {
        let sup = make_supervisor();
        let result = sup.spawn(simple_request("agent-1")).unwrap();

        // Move to Running first (Starting -> Running -> Stopping)
        sup.process_table()
            .update_state(result.pid, ProcessState::Running)
            .unwrap();

        sup.stop(result.pid, true).unwrap();

        let entry = sup.inspect(result.pid).unwrap();
        assert_eq!(entry.state, ProcessState::Stopping);
        assert!(entry.cancel_token.is_cancelled());
    }

    #[test]
    fn stop_force() {
        let sup = make_supervisor();
        let result = sup.spawn(simple_request("agent-1")).unwrap();

        // Move to Running first
        sup.process_table()
            .update_state(result.pid, ProcessState::Running)
            .unwrap();

        sup.stop(result.pid, false).unwrap();

        let entry = sup.inspect(result.pid).unwrap();
        assert!(entry.cancel_token.is_cancelled());
    }

    #[test]
    fn stop_already_exited_is_idempotent() {
        let sup = make_supervisor();
        let result = sup.spawn(simple_request("agent-1")).unwrap();

        // Move to exited
        sup.process_table()
            .update_state(result.pid, ProcessState::Exited(0))
            .unwrap();

        // Should succeed without error
        sup.stop(result.pid, true).unwrap();
    }

    #[test]
    fn stop_nonexistent_pid_fails() {
        let sup = make_supervisor();
        let result = sup.stop(999, true);
        assert!(result.is_err());
    }

    #[test]
    fn restart_creates_new_process() {
        let sup = make_supervisor();
        let original = sup.spawn(simple_request("agent-1")).unwrap();

        // Move to Running so it can be stopped
        sup.process_table()
            .update_state(original.pid, ProcessState::Running)
            .unwrap();

        let restarted = sup.restart(original.pid).unwrap();

        // New PID, same agent_id
        assert_ne!(restarted.pid, original.pid);
        assert_eq!(restarted.agent_id, "agent-1");

        // New process has parent_pid pointing to old PID
        let new_entry = sup.inspect(restarted.pid).unwrap();
        assert_eq!(new_entry.parent_pid, Some(original.pid));
    }

    #[test]
    fn restart_preserves_capabilities() {
        let sup = make_supervisor();
        let caps = AgentCapabilities {
            can_spawn: false,
            can_network: true,
            ..Default::default()
        };

        let request = SpawnRequest {
            agent_id: "restricted".to_owned(),
            capabilities: Some(caps),
            parent_pid: None,
            env: HashMap::new(),
        };

        let original = sup.spawn(request).unwrap();
        sup.process_table()
            .update_state(original.pid, ProcessState::Running)
            .unwrap();

        let restarted = sup.restart(original.pid).unwrap();
        let entry = sup.inspect(restarted.pid).unwrap();
        assert!(!entry.capabilities.can_spawn);
        assert!(entry.capabilities.can_network);
    }

    #[test]
    fn list_by_state() {
        let sup = make_supervisor();
        let r1 = sup.spawn(simple_request("a1")).unwrap();
        let r2 = sup.spawn(simple_request("a2")).unwrap();
        sup.spawn(simple_request("a3")).unwrap();

        // Move first two to Running
        sup.process_table()
            .update_state(r1.pid, ProcessState::Running)
            .unwrap();
        sup.process_table()
            .update_state(r2.pid, ProcessState::Running)
            .unwrap();

        let running = sup.list_by_state(ProcessState::Running);
        assert_eq!(running.len(), 2);

        let starting = sup.list_by_state(ProcessState::Starting);
        assert_eq!(starting.len(), 1);
    }

    #[test]
    fn list_agents_excludes_kernel() {
        let sup = make_supervisor();

        // Insert kernel PID 0
        let kernel_entry = ProcessEntry {
            pid: 0,
            agent_id: "kernel".to_owned(),
            state: ProcessState::Running,
            capabilities: AgentCapabilities::default(),
            resource_usage: ResourceUsage::default(),
            cancel_token: CancellationToken::new(),
            parent_pid: None,
        };
        sup.process_table().insert_with_pid(kernel_entry).unwrap();

        // Spawn an agent
        sup.spawn(simple_request("agent-1")).unwrap();

        let agents = sup.list_agents();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].agent_id, "agent-1");
    }

    #[test]
    fn running_count() {
        let sup = make_supervisor();
        let r1 = sup.spawn(simple_request("a1")).unwrap();
        let r2 = sup.spawn(simple_request("a2")).unwrap();
        sup.spawn(simple_request("a3")).unwrap();

        assert_eq!(sup.running_count(), 0); // All Starting

        sup.process_table()
            .update_state(r1.pid, ProcessState::Running)
            .unwrap();
        assert_eq!(sup.running_count(), 1);

        sup.process_table()
            .update_state(r2.pid, ProcessState::Running)
            .unwrap();
        assert_eq!(sup.running_count(), 2);
    }

    #[test]
    fn default_capabilities_accessor() {
        let sup = make_supervisor();
        let caps = sup.default_capabilities();
        assert!(caps.can_spawn);
        assert!(caps.can_ipc);
        assert!(caps.can_exec_tools);
    }

    #[test]
    fn spawn_request_serde_roundtrip() {
        let request = SpawnRequest {
            agent_id: "test".to_owned(),
            capabilities: Some(AgentCapabilities {
                can_spawn: false,
                ..Default::default()
            }),
            parent_pid: Some(5),
            env: HashMap::from([("KEY".into(), "VALUE".into())]),
        };

        let json = serde_json::to_string(&request).unwrap();
        let restored: SpawnRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.agent_id, "test");
        assert_eq!(restored.parent_pid, Some(5));
        assert!(!restored.capabilities.unwrap().can_spawn);
    }

    #[test]
    fn spawn_result_serde_roundtrip() {
        let result = SpawnResult {
            pid: 42,
            agent_id: "agent-42".to_owned(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let restored: SpawnResult = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.pid, 42);
        assert_eq!(restored.agent_id, "agent-42");
    }
}
