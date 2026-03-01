//! Kernel configuration types.
//!
//! These types are defined in `clawft-types` so they can be embedded
//! in the root [`Config`](super::Config) without creating a circular
//! dependency with `clawft-kernel`.

use serde::{Deserialize, Serialize};

/// Default maximum number of concurrent processes.
fn default_max_processes() -> u32 {
    64
}

/// Default health check interval in seconds.
fn default_health_check_interval_secs() -> u64 {
    30
}

/// Kernel subsystem configuration.
///
/// Embedded in the root `Config` under the `kernel` key. All fields
/// have sensible defaults so that existing configuration files parse
/// without errors.
///
/// # Example JSON
///
/// ```json
/// {
///   "kernel": {
///     "enabled": false,
///     "max_processes": 128,
///     "health_check_interval_secs": 15
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    /// Whether the kernel subsystem is enabled.
    ///
    /// When `false` (the default), kernel subsystems do not activate
    /// unless explicitly invoked via `weave kernel` CLI commands.
    #[serde(default)]
    pub enabled: bool,

    /// Maximum number of concurrent processes in the process table.
    #[serde(default = "default_max_processes", alias = "maxProcesses")]
    pub max_processes: u32,

    /// Interval (in seconds) between periodic health checks.
    #[serde(
        default = "default_health_check_interval_secs",
        alias = "healthCheckIntervalSecs"
    )]
    pub health_check_interval_secs: u64,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_processes: default_max_processes(),
            health_check_interval_secs: default_health_check_interval_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_kernel_config() {
        let cfg = KernelConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.max_processes, 64);
        assert_eq!(cfg.health_check_interval_secs, 30);
    }

    #[test]
    fn deserialize_empty() {
        let cfg: KernelConfig = serde_json::from_str("{}").unwrap();
        assert!(!cfg.enabled);
        assert_eq!(cfg.max_processes, 64);
    }

    #[test]
    fn deserialize_camel_case() {
        let json = r#"{"maxProcesses": 128, "healthCheckIntervalSecs": 15}"#;
        let cfg: KernelConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.max_processes, 128);
        assert_eq!(cfg.health_check_interval_secs, 15);
    }

    #[test]
    fn serde_roundtrip() {
        let cfg = KernelConfig {
            enabled: true,
            max_processes: 256,
            health_check_interval_secs: 10,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: KernelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.enabled, cfg.enabled);
        assert_eq!(restored.max_processes, cfg.max_processes);
    }
}
