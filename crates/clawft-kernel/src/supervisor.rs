//! Agent supervisor for process lifecycle management.
//!
//! The [`AgentSupervisor`] manages the full lifecycle of kernel-managed
//! agents: spawn, stop, restart, inspect, and watch. It wraps the
//! existing `AgentLoop` spawn mechanism without replacing it, adding
//! capability enforcement, resource tracking, and process table integration.

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use dashmap::DashMap;
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
    running_agents: Arc<DashMap<Pid, tokio::task::JoinHandle<()>>>,
    a2a_router: Option<Arc<crate::a2a::A2ARouter>>,
    cron_service: Option<Arc<crate::cron::CronService>>,
    #[cfg(feature = "exochain")]
    tree_manager: Option<Arc<crate::tree_manager::TreeManager>>,
    #[cfg(feature = "exochain")]
    chain_manager: Option<Arc<crate::chain::ChainManager>>,
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
            running_agents: Arc::new(DashMap::new()),
            a2a_router: None,
            cron_service: None,
            #[cfg(feature = "exochain")]
            tree_manager: None,
            #[cfg(feature = "exochain")]
            chain_manager: None,
            _platform: PhantomData,
        }
    }

    /// Configure A2A router and cron service.
    ///
    /// When set, `spawn_and_run` will create per-agent inboxes via the
    /// A2ARouter and pass the cron service handle to the agent work loop.
    pub fn with_a2a_router(
        mut self,
        a2a_router: Arc<crate::a2a::A2ARouter>,
        cron_service: Arc<crate::cron::CronService>,
    ) -> Self {
        self.a2a_router = Some(a2a_router);
        self.cron_service = Some(cron_service);
        self
    }

    /// Get the A2A router (if configured).
    pub fn a2a_router(&self) -> Option<&Arc<crate::a2a::A2ARouter>> {
        self.a2a_router.as_ref()
    }

    /// Get the cron service (if configured).
    pub fn cron_service(&self) -> Option<&Arc<crate::cron::CronService>> {
        self.cron_service.as_ref()
    }

    /// Configure exochain integration (tree + chain managers).
    ///
    /// When set, agent spawn/stop/restart events are recorded in
    /// the resource tree and hash chain.
    #[cfg(feature = "exochain")]
    pub fn with_exochain(
        mut self,
        tree_manager: Option<Arc<crate::tree_manager::TreeManager>>,
        chain_manager: Option<Arc<crate::chain::ChainManager>>,
    ) -> Self {
        self.tree_manager = tree_manager;
        self.chain_manager = chain_manager;
        self
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

    /// Spawn a supervised agent and run its work as a tokio task.
    ///
    /// Unlike [`spawn`], this method also:
    /// 1. Transitions the process to `Running`
    /// 2. Registers the agent in the resource tree (if exochain enabled)
    /// 3. Spawns a tokio task to execute the provided work closure
    /// 4. On completion: transitions to `Exited`, unregisters from tree,
    ///    logs chain events, and cleans up the task handle
    ///
    /// The `work` closure receives the assigned PID and a
    /// [`CancellationToken`]; it should return an exit code (0 = success).
    ///
    /// # Errors
    ///
    /// Returns `KernelError::ProcessTableFull` if the process table
    /// has reached its maximum capacity.
    pub fn spawn_and_run<F, Fut>(
        &self,
        request: SpawnRequest,
        work: F,
    ) -> KernelResult<SpawnResult>
    where
        F: FnOnce(Pid, CancellationToken) -> Fut,
        Fut: std::future::Future<Output = i32> + Send + 'static,
    {
        // 1. Create process entry via existing spawn()
        let result = self.spawn(request)?;
        let pid = result.pid;

        let entry = self
            .process_table
            .get(pid)
            .ok_or(KernelError::ProcessNotFound { pid })?;
        let cancel_token = entry.cancel_token.clone();

        // 2. Register in resource tree (exochain)
        #[cfg(feature = "exochain")]
        if let Some(ref tm) = self.tree_manager
            && let Err(e) = tm.register_agent(&result.agent_id, pid, &entry.capabilities)
        {
            warn!(error = %e, pid, "failed to register agent in resource tree");
        }

        // 3. Transition to Running
        let _ = self
            .process_table
            .update_state(pid, ProcessState::Running);

        // 4. Spawn tokio task
        let process_table = Arc::clone(&self.process_table);
        let running_agents = Arc::clone(&self.running_agents);
        let agent_id = result.agent_id.clone();
        #[cfg(feature = "exochain")]
        let tree_manager = self.tree_manager.clone();
        #[cfg(feature = "exochain")]
        let chain_manager = self.chain_manager.clone();

        let future = work(pid, cancel_token);
        let handle = tokio::spawn(async move {
            let exit_code = future.await;

            // Transition to Exited
            let _ = process_table.update_state(pid, ProcessState::Exited(exit_code));

            // Unregister from tree
            #[cfg(feature = "exochain")]
            if let Some(ref tm) = tree_manager
                && let Err(e) = tm.unregister_agent(&agent_id, pid, exit_code)
            {
                tracing::warn!(error = %e, pid, "failed to unregister agent from tree");
            }

            // Log exit chain event
            #[cfg(feature = "exochain")]
            if let Some(ref cm) = chain_manager {
                cm.append(
                    "supervisor",
                    "agent.exit",
                    Some(serde_json::json!({
                        "agent_id": agent_id,
                        "pid": pid,
                        "exit_code": exit_code,
                    })),
                );
            }

            // Remove from running agents map
            running_agents.remove(&pid);

            info!(pid, exit_code, agent_id = %agent_id, "agent task completed");
        });

        self.running_agents.insert(pid, handle);

        info!(pid, agent_id = %result.agent_id, "agent spawned and running");

        Ok(result)
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
            // Transition to Stopping, then cancel the token.
            // The spawned task (if any) will detect cancellation,
            // exit, and handle tree/chain cleanup.
            let _ = self.process_table.update_state(pid, ProcessState::Stopping);
            entry.cancel_token.cancel();
        } else {
            info!(pid, "force stopping agent");
            entry.cancel_token.cancel();
            let _ = self
                .process_table
                .update_state(pid, ProcessState::Exited(-1));

            // Abort the running task handle (cleanup won't run)
            if let Some((_, handle)) = self.running_agents.remove(&pid) {
                handle.abort();
            }

            // Since the spawned task was aborted, do tree/chain
            // cleanup directly here.
            #[cfg(feature = "exochain")]
            {
                if let Some(ref tm) = self.tree_manager {
                    let _ = tm.unregister_agent(&entry.agent_id, pid, -1);
                }
                if let Some(ref cm) = self.chain_manager {
                    cm.append(
                        "supervisor",
                        "agent.force_stop",
                        Some(serde_json::json!({
                            "agent_id": entry.agent_id,
                            "pid": pid,
                        })),
                    );
                }
            }
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

        let result = self.spawn(request)?;

        // Log restart chain event linking old PID to new PID
        #[cfg(feature = "exochain")]
        if let Some(ref cm) = self.chain_manager {
            cm.append(
                "supervisor",
                "agent.restart",
                Some(serde_json::json!({
                    "agent_id": result.agent_id,
                    "old_pid": pid,
                    "new_pid": result.pid,
                })),
            );
        }

        Ok(result)
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

    /// Get the number of actively tracked running agent tasks.
    pub fn running_task_count(&self) -> usize {
        self.running_agents.len()
    }

    /// Abort all running agent tasks (used during forced shutdown).
    pub fn abort_all(&self) {
        for entry in self.running_agents.iter() {
            entry.value().abort();
        }
        self.running_agents.clear();
    }

    /// Get the tree manager (when exochain feature is enabled).
    #[cfg(feature = "exochain")]
    pub fn tree_manager(&self) -> Option<&Arc<crate::tree_manager::TreeManager>> {
        self.tree_manager.as_ref()
    }

    /// Get the chain manager (when exochain feature is enabled).
    #[cfg(feature = "exochain")]
    pub fn chain_manager(&self) -> Option<&Arc<crate::chain::ChainManager>> {
        self.chain_manager.as_ref()
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

    #[tokio::test]
    async fn spawn_and_run_executes_work() {
        let sup = make_supervisor();

        let result = sup
            .spawn_and_run(simple_request("runner-1"), |_pid, _cancel| async { 0 })
            .unwrap();

        assert!(result.pid > 0);
        assert_eq!(result.agent_id, "runner-1");

        // Process should be Running immediately after spawn_and_run
        let entry = sup.inspect(result.pid).unwrap();
        assert_eq!(entry.state, ProcessState::Running);

        // Wait for the task to complete
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Process should be Exited after work completes
        let entry = sup.inspect(result.pid).unwrap();
        assert!(matches!(entry.state, ProcessState::Exited(0)));

        // Running task should be cleaned up
        assert_eq!(sup.running_task_count(), 0);
    }

    #[tokio::test]
    async fn spawn_and_run_respects_cancellation() {
        let sup = make_supervisor();

        let result = sup
            .spawn_and_run(simple_request("cancellable"), |_pid, cancel| async move {
                cancel.cancelled().await;
                42
            })
            .unwrap();

        assert_eq!(sup.running_task_count(), 1);

        // Stop the agent
        sup.stop(result.pid, true).unwrap();

        // Wait for the task to detect cancellation and exit
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let entry = sup.inspect(result.pid).unwrap();
        assert!(matches!(entry.state, ProcessState::Exited(42)));
        assert_eq!(sup.running_task_count(), 0);
    }

    #[tokio::test]
    async fn spawn_and_run_force_stop_aborts() {
        let sup = make_supervisor();

        let result = sup
            .spawn_and_run(simple_request("force-me"), |_pid, cancel| async move {
                cancel.cancelled().await;
                0
            })
            .unwrap();

        // Force stop should abort the task immediately
        sup.stop(result.pid, false).unwrap();

        let entry = sup.inspect(result.pid).unwrap();
        assert!(matches!(entry.state, ProcessState::Exited(-1)));
        assert_eq!(sup.running_task_count(), 0);
    }

    #[tokio::test]
    async fn abort_all_clears_running_agents() {
        let sup = make_supervisor();

        sup.spawn_and_run(simple_request("a1"), |_pid, cancel| async move {
            cancel.cancelled().await;
            0
        })
        .unwrap();
        sup.spawn_and_run(simple_request("a2"), |_pid, cancel| async move {
            cancel.cancelled().await;
            0
        })
        .unwrap();

        assert_eq!(sup.running_task_count(), 2);

        sup.abort_all();

        assert_eq!(sup.running_task_count(), 0);
    }
}
