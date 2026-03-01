//! Kernel console: boot event types and output formatting.
//!
//! Provides [`BootEvent`], [`BootPhase`], and [`LogLevel`] types for
//! recording and displaying kernel boot output. The interactive REPL
//! loop is stubbed (requires complex stdin handling); only the event
//! types and output formatting are implemented.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Phase of the boot sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootPhase {
    /// Pre-boot initialization.
    Init,
    /// Loading configuration.
    Config,
    /// Registering system services.
    Services,
    /// Loading resource tree from DAG.
    ResourceTree,
    /// Spawning service agents.
    Agents,
    /// Network service discovery.
    Network,
    /// Boot complete, ready for commands.
    Ready,
}

impl BootPhase {
    /// Short tag string for console output (e.g. `[INIT]`).
    pub fn tag(&self) -> &'static str {
        match self {
            BootPhase::Init => "INIT",
            BootPhase::Config => "CONFIG",
            BootPhase::Services => "SERVICES",
            BootPhase::ResourceTree => "TREE",
            BootPhase::Agents => "AGENTS",
            BootPhase::Network => "NETWORK",
            BootPhase::Ready => "READY",
        }
    }
}

impl std::fmt::Display for BootPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tag())
    }
}

/// Log level for boot events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Debug-level messages (not shown in normal boot output).
    Debug,
    /// Informational messages (standard boot output).
    Info,
    /// Warning messages.
    Warn,
    /// Error messages.
    Error,
}

/// A single boot event recorded during kernel startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootEvent {
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Which boot phase generated the event.
    pub phase: BootPhase,
    /// Human-readable event message.
    pub message: String,
    /// Severity level.
    pub level: LogLevel,
}

impl BootEvent {
    /// Create a new info-level boot event.
    pub fn info(phase: BootPhase, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            phase,
            message: message.into(),
            level: LogLevel::Info,
        }
    }

    /// Create a new warning-level boot event.
    pub fn warn(phase: BootPhase, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            phase,
            message: message.into(),
            level: LogLevel::Warn,
        }
    }

    /// Create a new error-level boot event.
    pub fn error(phase: BootPhase, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            phase,
            message: message.into(),
            level: LogLevel::Error,
        }
    }

    /// Format this event for console display.
    ///
    /// Example: `  [INIT]      WeftOS v0.1.0 booting...`
    pub fn format_line(&self) -> String {
        let tag = self.phase.tag();
        format!("  [{tag:<10}] {}", self.message)
    }
}

/// Boot log: a recorded sequence of boot events.
///
/// Used to replay boot output when attaching to a running kernel
/// or for diagnostics.
#[derive(Debug, Clone, Default)]
pub struct BootLog {
    events: Vec<BootEvent>,
}

impl BootLog {
    /// Create an empty boot log.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Record a boot event.
    pub fn push(&mut self, event: BootEvent) {
        self.events.push(event);
    }

    /// Get all recorded events.
    pub fn events(&self) -> &[BootEvent] {
        &self.events
    }

    /// Format all events for console display.
    pub fn format_all(&self) -> String {
        let mut output = String::new();
        for event in &self.events {
            if event.level == LogLevel::Debug {
                continue;
            }
            output.push_str(&event.format_line());
            output.push('\n');
        }
        output
    }

    /// Get the number of recorded events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Format the boot banner header.
pub fn boot_banner() -> String {
    let mut output = String::new();
    output.push_str("\n  WeftOS v0.1.0\n");
    output.push_str("  ");
    output.push_str(&"-".repeat(45));
    output.push('\n');
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_phase_tags() {
        assert_eq!(BootPhase::Init.tag(), "INIT");
        assert_eq!(BootPhase::Config.tag(), "CONFIG");
        assert_eq!(BootPhase::Services.tag(), "SERVICES");
        assert_eq!(BootPhase::ResourceTree.tag(), "TREE");
        assert_eq!(BootPhase::Agents.tag(), "AGENTS");
        assert_eq!(BootPhase::Network.tag(), "NETWORK");
        assert_eq!(BootPhase::Ready.tag(), "READY");
    }

    #[test]
    fn boot_event_info() {
        let event = BootEvent::info(BootPhase::Init, "WeftOS v0.1.0 booting...");
        assert_eq!(event.phase, BootPhase::Init);
        assert_eq!(event.level, LogLevel::Info);
        assert_eq!(event.message, "WeftOS v0.1.0 booting...");
    }

    #[test]
    fn boot_event_format_line() {
        let event = BootEvent::info(BootPhase::Init, "PID 0 (kernel)");
        let line = event.format_line();
        assert!(line.contains("[INIT"));
        assert!(line.contains("PID 0 (kernel)"));
    }

    #[test]
    fn boot_log_push_and_format() {
        let mut log = BootLog::new();
        log.push(BootEvent::info(BootPhase::Init, "booting..."));
        log.push(BootEvent::info(BootPhase::Config, "config loaded"));
        log.push(BootEvent::info(BootPhase::Ready, "ready"));

        assert_eq!(log.len(), 3);
        let formatted = log.format_all();
        assert!(formatted.contains("booting..."));
        assert!(formatted.contains("config loaded"));
        assert!(formatted.contains("ready"));
    }

    #[test]
    fn boot_log_skips_debug() {
        let mut log = BootLog::new();
        log.push(BootEvent {
            timestamp: Utc::now(),
            phase: BootPhase::Init,
            message: "debug msg".into(),
            level: LogLevel::Debug,
        });
        log.push(BootEvent::info(BootPhase::Init, "info msg"));

        let formatted = log.format_all();
        assert!(!formatted.contains("debug msg"));
        assert!(formatted.contains("info msg"));
    }

    #[test]
    fn boot_log_empty() {
        let log = BootLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert!(log.format_all().is_empty());
    }

    #[test]
    fn boot_banner_format() {
        let banner = boot_banner();
        assert!(banner.contains("WeftOS v0.1.0"));
        assert!(banner.contains("---"));
    }

    #[test]
    fn boot_event_serde() {
        let event = BootEvent::info(BootPhase::Services, "[OK] message-bus");
        let json = serde_json::to_string(&event).unwrap();
        let restored: BootEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.phase, BootPhase::Services);
        assert_eq!(restored.message, "[OK] message-bus");
    }

    #[test]
    fn boot_event_warn_and_error() {
        let warn_event = BootEvent::warn(BootPhase::Services, "slow start");
        assert_eq!(warn_event.level, LogLevel::Warn);

        let err_event = BootEvent::error(BootPhase::Services, "failed");
        assert_eq!(err_event.level, LogLevel::Error);
    }
}
