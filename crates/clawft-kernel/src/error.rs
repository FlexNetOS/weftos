//! Kernel error types.
//!
//! All kernel operations return [`KernelError`] for typed error
//! handling. The error variants cover process table operations,
//! service lifecycle, IPC, and boot sequence failures.

use crate::process::ProcessState;

/// Kernel-level errors.
#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    /// Process not found in the process table.
    #[error("process not found: PID {pid}")]
    ProcessNotFound {
        /// The PID that was looked up.
        pid: u64,
    },

    /// Invalid process state transition.
    #[error("invalid state transition for PID {pid}: {from} -> {to}")]
    InvalidStateTransition {
        /// The affected PID.
        pid: u64,
        /// Current state.
        from: ProcessState,
        /// Requested state.
        to: ProcessState,
    },

    /// Process table has reached maximum capacity.
    #[error("process table full (max: {max})")]
    ProcessTableFull {
        /// Maximum number of processes allowed.
        max: u32,
    },

    /// Service-related error.
    #[error("service error: {0}")]
    Service(String),

    /// Boot sequence error.
    #[error("boot error: {0}")]
    Boot(String),

    /// IPC / messaging error.
    #[error("ipc error: {0}")]
    Ipc(String),

    /// Kernel is in wrong state for requested operation.
    #[error("kernel state error: expected {expected}, got {actual}")]
    WrongState {
        /// Expected state.
        expected: String,
        /// Actual state.
        actual: String,
    },

    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),

    /// Wraps a generic error from downstream crates.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Convenience alias for kernel results.
pub type KernelResult<T> = Result<T, KernelError>;
