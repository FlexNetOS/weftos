//! Agent capabilities and resource limits.
//!
//! Defines the permission model for kernel-managed agents. Each agent
//! process has an [`AgentCapabilities`] that governs what IPC scopes,
//! tool categories, and resource budgets the agent is allowed.

use serde::{Deserialize, Serialize};

/// Resource limits for an agent process.
///
/// Enforced by the kernel's process table when updating resource
/// usage counters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Maximum memory (in bytes) the agent is allowed to consume.
    #[serde(default = "default_max_memory", alias = "maxMemoryBytes")]
    pub max_memory_bytes: u64,

    /// Maximum CPU time (in milliseconds) before the agent is killed.
    #[serde(default = "default_max_cpu", alias = "maxCpuTimeMs")]
    pub max_cpu_time_ms: u64,

    /// Maximum number of tool calls the agent may make.
    #[serde(default = "default_max_tool_calls", alias = "maxToolCalls")]
    pub max_tool_calls: u64,

    /// Maximum number of IPC messages the agent may send.
    #[serde(default = "default_max_messages", alias = "maxMessages")]
    pub max_messages: u64,
}

fn default_max_memory() -> u64 {
    256 * 1024 * 1024 // 256 MiB
}

fn default_max_cpu() -> u64 {
    300_000 // 5 minutes
}

fn default_max_tool_calls() -> u64 {
    1000
}

fn default_max_messages() -> u64 {
    5000
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: default_max_memory(),
            max_cpu_time_ms: default_max_cpu(),
            max_tool_calls: default_max_tool_calls(),
            max_messages: default_max_messages(),
        }
    }
}

/// IPC scope defining which message targets an agent may communicate with.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum IpcScope {
    /// Agent may communicate with all other agents.
    #[default]
    All,
    /// Agent may only communicate with its parent.
    ParentOnly,
    /// Agent may communicate with a specified set of PIDs.
    Restricted(Vec<u64>),
    /// Agent may not send IPC messages.
    None,
}


/// Capabilities assigned to an agent process.
///
/// Governs what the agent is allowed to do within the kernel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentCapabilities {
    /// Whether the agent can spawn child processes.
    #[serde(default = "default_true", alias = "canSpawn")]
    pub can_spawn: bool,

    /// Whether the agent can send/receive IPC messages.
    #[serde(default = "default_true", alias = "canIpc")]
    pub can_ipc: bool,

    /// Whether the agent can execute tools.
    #[serde(default = "default_true", alias = "canExecTools")]
    pub can_exec_tools: bool,

    /// Whether the agent can make network requests.
    #[serde(default, alias = "canNetwork")]
    pub can_network: bool,

    /// IPC scope restriction.
    #[serde(default, alias = "ipcScope")]
    pub ipc_scope: IpcScope,

    /// Resource limits for this agent.
    #[serde(default, alias = "resourceLimits")]
    pub resource_limits: ResourceLimits,
}

fn default_true() -> bool {
    true
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            can_spawn: true,
            can_ipc: true,
            can_exec_tools: true,
            can_network: false,
            ipc_scope: IpcScope::default(),
            resource_limits: ResourceLimits::default(),
        }
    }
}

impl AgentCapabilities {
    /// Check whether the agent is allowed to send a message to the given PID.
    pub fn can_message(&self, target_pid: u64) -> bool {
        if !self.can_ipc {
            return false;
        }
        match &self.ipc_scope {
            IpcScope::All => true,
            IpcScope::ParentOnly => false, // Caller must check parent separately
            IpcScope::Restricted(pids) => pids.contains(&target_pid),
            IpcScope::None => false,
        }
    }

    /// Check whether the resource usage is within limits.
    pub fn within_limits(&self, memory: u64, cpu: u64, tools: u64, msgs: u64) -> bool {
        memory <= self.resource_limits.max_memory_bytes
            && cpu <= self.resource_limits.max_cpu_time_ms
            && tools <= self.resource_limits.max_tool_calls
            && msgs <= self.resource_limits.max_messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_capabilities() {
        let caps = AgentCapabilities::default();
        assert!(caps.can_spawn);
        assert!(caps.can_ipc);
        assert!(caps.can_exec_tools);
        assert!(!caps.can_network);
        assert_eq!(caps.ipc_scope, IpcScope::All);
    }

    #[test]
    fn default_resource_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 256 * 1024 * 1024);
        assert_eq!(limits.max_cpu_time_ms, 300_000);
        assert_eq!(limits.max_tool_calls, 1000);
        assert_eq!(limits.max_messages, 5000);
    }

    #[test]
    fn can_message_all_scope() {
        let caps = AgentCapabilities::default();
        assert!(caps.can_message(1));
        assert!(caps.can_message(999));
    }

    #[test]
    fn can_message_restricted_scope() {
        let caps = AgentCapabilities {
            ipc_scope: IpcScope::Restricted(vec![1, 2, 3]),
            ..Default::default()
        };
        assert!(caps.can_message(1));
        assert!(caps.can_message(2));
        assert!(!caps.can_message(4));
    }

    #[test]
    fn can_message_none_scope() {
        let caps = AgentCapabilities {
            ipc_scope: IpcScope::None,
            ..Default::default()
        };
        assert!(!caps.can_message(1));
    }

    #[test]
    fn can_message_ipc_disabled() {
        let caps = AgentCapabilities {
            can_ipc: false,
            ..Default::default()
        };
        assert!(!caps.can_message(1));
    }

    #[test]
    fn within_limits_ok() {
        let caps = AgentCapabilities::default();
        assert!(caps.within_limits(1000, 1000, 10, 10));
    }

    #[test]
    fn within_limits_exceeded() {
        let caps = AgentCapabilities {
            resource_limits: ResourceLimits {
                max_memory_bytes: 100,
                max_cpu_time_ms: 100,
                max_tool_calls: 5,
                max_messages: 5,
            },
            ..Default::default()
        };
        assert!(!caps.within_limits(200, 50, 3, 3)); // memory exceeded
        assert!(!caps.within_limits(50, 200, 3, 3)); // cpu exceeded
        assert!(!caps.within_limits(50, 50, 10, 3)); // tools exceeded
        assert!(!caps.within_limits(50, 50, 3, 10)); // messages exceeded
    }

    #[test]
    fn serde_roundtrip_capabilities() {
        let caps = AgentCapabilities {
            can_spawn: false,
            can_ipc: true,
            can_exec_tools: false,
            can_network: true,
            ipc_scope: IpcScope::Restricted(vec![1, 2]),
            resource_limits: ResourceLimits {
                max_memory_bytes: 1024,
                max_cpu_time_ms: 500,
                max_tool_calls: 10,
                max_messages: 20,
            },
        };
        let json = serde_json::to_string(&caps).unwrap();
        let restored: AgentCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, caps);
    }

    #[test]
    fn deserialize_empty_capabilities() {
        let caps: AgentCapabilities = serde_json::from_str("{}").unwrap();
        assert!(caps.can_spawn);
        assert!(caps.can_ipc);
        assert!(caps.can_exec_tools);
        assert!(!caps.can_network);
    }
}
