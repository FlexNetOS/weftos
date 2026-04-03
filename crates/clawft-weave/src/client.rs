//! Daemon client — re-exported from `clawft-rpc`.
//!
//! The canonical client implementation lives in the `clawft-rpc` crate.
//! This module re-exports it for backward compatibility within clawft-weave.

pub use clawft_rpc::{connect_or_bail, is_daemon_running, DaemonClient};
