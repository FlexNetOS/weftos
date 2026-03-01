//! `weaver` — WeftOS operator CLI.
//!
//! The human-facing CLI for kernel management, agent orchestration,
//! and system administration. Complement to `weft` (the agent CLI).
//!
//! # Commands
//!
//! - `weaver kernel` — Boot, status, process table, services.
//! - `weaver agent` — Spawn, stop, restart, inspect agents (planned).
//! - `weaver app` — Install, start, stop applications (planned).
//! - `weaver ipc` — Send messages, manage topics (planned).

use clap::{Parser, Subcommand};

mod client;
mod commands;
mod daemon;
mod protocol;

/// WeftOS operator CLI.
#[derive(Parser)]
#[command(
    name = "weaver",
    about = "WeftOS operator CLI — kernel, agents, and system management",
    version,
    disable_help_subcommand = true
)]
struct Cli {
    /// Enable verbose (debug-level) logging.
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

/// Top-level subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Kernel management (boot, status, services, processes).
    Kernel(commands::kernel_cmd::KernelArgs),

    /// Cluster management (nodes, shards, health).
    Cluster(commands::cluster_cmd::ClusterArgs),

    /// Show version and build info.
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let default_filter = if cli.verbose { "debug" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_filter.into()),
        )
        .init();

    match cli.command {
        Commands::Kernel(args) => commands::kernel_cmd::run(args).await?,
        Commands::Cluster(args) => commands::cluster_cmd::run(args).await?,
        Commands::Version => {
            println!("weaver {} (WeftOS)", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parses_without_error() {
        Cli::command().debug_assert();
    }

    #[test]
    fn cli_help_contains_binary_name() {
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("weaver"));
    }
}
