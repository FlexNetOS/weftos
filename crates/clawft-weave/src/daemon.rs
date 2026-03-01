//! Kernel daemon — persistent kernel process with Unix socket RPC.
//!
//! The daemon boots a [`Kernel`], then listens on a Unix domain socket
//! for JSON-RPC requests. This is the native transport layer; the
//! kernel itself is platform-agnostic and could be wrapped in
//! WebSocket, TCP, or `postMessage` for other environments.

use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use clawft_kernel::{Kernel, KernelState};
use clawft_platform::NativePlatform;
use clawft_types::config::{Config, KernelConfig};

use crate::protocol::{
    self, ClusterJoinParams, ClusterLeaveParams, ClusterNodeInfo, ClusterStatusResult,
    KernelStatusResult, LogEntry, LogsParams, ProcessInfo, Request, Response, ServiceInfo,
};

/// Fork the daemon into the background.
///
/// Spawns `weaver kernel start --foreground` as a detached child process,
/// redirecting stdout/stderr to the kernel log file. Writes the child PID
/// to the PID file. The parent process exits immediately after confirming
/// the daemon started.
pub fn daemonize(config_override: Option<&str>) -> anyhow::Result<()> {
    use std::process::Command;

    let runtime_dir = protocol::runtime_dir();
    std::fs::create_dir_all(&runtime_dir)?;

    let log_path = protocol::log_path();
    let pid_path = protocol::pid_path();

    // Check if already running
    if pid_path.exists()
        && let Ok(pid_str) = std::fs::read_to_string(&pid_path)
    {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            // Check if process is alive
            let check = Command::new("kill").args(["-0", &pid.to_string()]).output();
            if check.map(|o| o.status.success()).unwrap_or(false) {
                anyhow::bail!("kernel already running (pid {pid})");
            }
        }
        // Stale PID file
        let _ = std::fs::remove_file(&pid_path);
    }

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let log_err = log_file.try_clone()?;

    let mut cmd = Command::new(std::env::current_exe()?);
    cmd.args(["kernel", "start", "--foreground"]);
    if let Some(cfg) = config_override {
        cmd.args(["--config", cfg]);
    }

    let child = cmd
        .stdout(log_file)
        .stderr(log_err)
        .stdin(std::process::Stdio::null())
        .spawn()?;

    let pid = child.id();
    std::fs::write(&pid_path, pid.to_string())?;

    println!("WeftOS kernel started (pid {pid})");
    println!("  Socket: {}", protocol::socket_path().display());
    println!("  Log:    {}", log_path.display());
    println!("  PID:    {}", pid_path.display());
    println!();
    println!("Use 'weaver kernel status' to check, 'weaver kernel attach' to view logs.");
    println!("Use 'weaver kernel stop' to shut down.");

    Ok(())
}

/// Run the kernel daemon in the foreground.
///
/// Boots the kernel, binds to a Unix socket, and serves requests
/// until shutdown is requested (via `kernel.shutdown` RPC or signal).
pub async fn run(config: Config, kernel_config: KernelConfig) -> anyhow::Result<()> {
    let socket_path = protocol::socket_path();

    // Clean up stale socket file
    if socket_path.exists() {
        // Try connecting to see if a daemon is already running
        if tokio::net::UnixStream::connect(&socket_path)
            .await
            .is_ok()
        {
            anyhow::bail!(
                "daemon already running (socket exists and is accepting connections: {})",
                socket_path.display()
            );
        }
        // Stale socket — remove it
        std::fs::remove_file(&socket_path)?;
        debug!("removed stale socket file");
    }

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Boot kernel
    let platform = NativePlatform::new();
    let kernel = Kernel::boot(config, kernel_config, Arc::new(platform)).await?;
    let kernel = Arc::new(tokio::sync::RwLock::new(kernel));

    // Print boot banner
    {
        let k = kernel.read().await;
        print!("{}", clawft_kernel::console::boot_banner());
        print!("{}", k.boot_log().format_all());
    }

    // Bind socket
    let listener = UnixListener::bind(&socket_path)?;
    info!(path = %socket_path.display(), "daemon listening");
    println!("Daemon listening on {}", socket_path.display());

    // Log daemon start to kernel event log
    {
        let k = kernel.read().await;
        k.event_log()
            .info("daemon", format!("listening on {}", socket_path.display()));
    }

    // Shutdown signal
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Accept loop — clone shutdown_tx so the outer scope can still use it for Ctrl+C
    let accept_kernel = Arc::clone(&kernel);
    let rpc_shutdown_tx = shutdown_tx.clone();
    let mut accept_handle = tokio::spawn(async move {
        let mut shutdown_rx = shutdown_rx;
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            let k = Arc::clone(&accept_kernel);
                            let tx = rpc_shutdown_tx.clone();
                            tokio::spawn(handle_connection(stream, k, tx));
                        }
                        Err(e) => {
                            error!("accept error: {e}");
                        }
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("shutdown signal received, stopping accept loop");
                        break;
                    }
                }
            }
        }
    });

    // Wait for either Ctrl+C or the accept loop to finish (RPC shutdown).
    let ctrl_c_triggered = tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received, shutting down daemon");
            let _ = shutdown_tx.send(true);
            true
        }
        _ = &mut accept_handle => {
            // Accept loop finished (shutdown requested via RPC)
            info!("accept loop finished (RPC shutdown)");
            false
        }
    };

    // If Ctrl+C triggered, wait for the accept loop to finish.
    // If it finished on its own (RPC shutdown), the handle is already consumed.
    if ctrl_c_triggered {
        let _ = accept_handle.await;
    }

    // Shut down kernel
    {
        let mut k = kernel.write().await;
        if let Err(e) = k.shutdown().await {
            warn!("kernel shutdown error: {e}");
        }
    }

    // Clean up socket and PID file
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }
    let pid_path = protocol::pid_path();
    if pid_path.exists() {
        let _ = std::fs::remove_file(&pid_path);
    }

    println!("Daemon stopped.");
    Ok(())
}

/// Handle a single client connection.
async fn handle_connection(
    stream: tokio::net::UnixStream,
    kernel: Arc<tokio::sync::RwLock<Kernel<NativePlatform>>>,
    shutdown_tx: watch::Sender<bool>,
) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => {
                let id = req.id.clone();
                dispatch(
                    req.method,
                    req.params,
                    Arc::clone(&kernel),
                    shutdown_tx.clone(),
                )
                .await
                .with_id(id)
            }
            Err(e) => Response::error(format!("invalid request: {e}")),
        };

        let mut json = serde_json::to_string(&response).unwrap_or_else(|e| {
            serde_json::to_string(&Response::error(format!("serialize error: {e}"))).unwrap()
        });
        json.push('\n');

        if let Err(e) = writer.write_all(json.as_bytes()).await {
            debug!("write error (client disconnected?): {e}");
            break;
        }
    }
}

/// Dispatch a request to the appropriate handler.
///
/// Takes owned values to ensure the future is `Send + 'static`
/// for use inside `tokio::spawn`.
async fn dispatch(
    method: String,
    params: serde_json::Value,
    kernel: Arc<tokio::sync::RwLock<Kernel<NativePlatform>>>,
    shutdown_tx: watch::Sender<bool>,
) -> Response {
    match method.as_str() {
        "kernel.status" => {
            let k = kernel.read().await;
            let state_str = match k.state() {
                KernelState::Booting => "booting",
                KernelState::Running => "running",
                KernelState::ShuttingDown => "shutting_down",
                KernelState::Halted => "halted",
            };
            let result = KernelStatusResult {
                state: state_str.to_owned(),
                uptime_secs: k.uptime().as_secs_f64(),
                process_count: k.process_table().len(),
                service_count: k.services().len(),
                max_processes: k.kernel_config().max_processes,
                health_check_interval_secs: k.kernel_config().health_check_interval_secs,
            };
            Response::success(serde_json::to_value(result).unwrap())
        }
        "kernel.ps" => {
            let k = kernel.read().await;
            let mut entries: Vec<ProcessInfo> = k
                .process_table()
                .list()
                .iter()
                .map(|e| ProcessInfo {
                    pid: e.pid,
                    agent_id: e.agent_id.clone(),
                    state: e.state.to_string(),
                    memory_bytes: e.resource_usage.memory_bytes,
                    cpu_time_ms: e.resource_usage.cpu_time_ms,
                    parent_pid: e.parent_pid,
                })
                .collect();
            entries.sort_by_key(|e| e.pid);
            Response::success(serde_json::to_value(entries).unwrap())
        }
        "kernel.services" => {
            let k = kernel.read().await;
            let services = k.services().list();
            let infos: Vec<ServiceInfo> = services
                .iter()
                .map(|(name, stype)| ServiceInfo {
                    name: name.clone(),
                    service_type: stype.to_string(),
                    health: "registered".into(),
                })
                .collect();
            Response::success(serde_json::to_value(infos).unwrap())
        }
        "kernel.logs" => {
            let log_params: LogsParams = serde_json::from_value(params).unwrap_or(LogsParams {
                count: 50,
                level: None,
            });

            let k = kernel.read().await;
            let event_log = k.event_log();

            let events = if let Some(ref level_str) = log_params.level {
                let level = match level_str.as_str() {
                    "debug" => clawft_kernel::LogLevel::Debug,
                    "warn" | "warning" => clawft_kernel::LogLevel::Warn,
                    "error" => clawft_kernel::LogLevel::Error,
                    _ => clawft_kernel::LogLevel::Info,
                };
                event_log.filter_level(&level, log_params.count)
            } else {
                event_log.tail(log_params.count)
            };

            let entries: Vec<LogEntry> = events
                .iter()
                .map(|e| LogEntry {
                    timestamp: e.timestamp.to_rfc3339(),
                    phase: e.phase.tag().to_owned(),
                    level: format!("{:?}", e.level).to_lowercase(),
                    message: e.message.clone(),
                })
                .collect();

            Response::success(serde_json::to_value(entries).unwrap())
        }
        "kernel.shutdown" => {
            // Log the shutdown event before signaling
            {
                let k = kernel.read().await;
                k.event_log().info("daemon", "shutdown requested via RPC");
            }
            info!("shutdown requested via RPC");
            let _ = shutdown_tx.send(true);
            Response::success(serde_json::json!("shutting down"))
        }
        "cluster.status" => {
            let k = kernel.read().await;
            let membership = k.cluster_membership();
            let active = membership.count_by_state(&clawft_kernel::NodeState::Active);
            let total = membership.len();

            let result = ClusterStatusResult {
                total_nodes: total,
                healthy_nodes: active,
                total_shards: 0,
                active_shards: 0,
                consensus_enabled: false,
            };
            Response::success(serde_json::to_value(result).unwrap())
        }
        "cluster.nodes" => {
            let k = kernel.read().await;
            let membership = k.cluster_membership();
            let peers = membership.list_peers();
            let nodes: Vec<ClusterNodeInfo> = peers
                .iter()
                .map(|(id, state, platform)| {
                    let peer = membership.get_peer(id);
                    ClusterNodeInfo {
                        node_id: id.clone(),
                        name: peer
                            .as_ref()
                            .map(|p| p.name.clone())
                            .unwrap_or_else(|| id.clone()),
                        platform: platform.to_string(),
                        state: state.to_string(),
                        address: peer.and_then(|p| p.address),
                        last_seen: String::new(),
                    }
                })
                .collect();
            Response::success(serde_json::to_value(nodes).unwrap())
        }
        "cluster.join" => {
            let join_params: ClusterJoinParams =
                match serde_json::from_value(params) {
                    Ok(p) => p,
                    Err(e) => return Response::error(format!("invalid params: {e}")),
                };

            let k = kernel.read().await;
            let membership = k.cluster_membership();
            let node_id = uuid::Uuid::new_v4().to_string();
            let platform = match join_params.platform.as_str() {
                "browser" => clawft_kernel::NodePlatform::Browser,
                "edge" => clawft_kernel::NodePlatform::Edge,
                "wasi" => clawft_kernel::NodePlatform::Wasi,
                _ => clawft_kernel::NodePlatform::CloudNative,
            };
            let peer = clawft_kernel::PeerNode {
                id: node_id.clone(),
                name: join_params.name.unwrap_or_else(|| node_id.clone()),
                platform,
                state: clawft_kernel::NodeState::Active,
                address: join_params.address,
                first_seen: chrono::Utc::now(),
                last_heartbeat: chrono::Utc::now(),
                capabilities: Vec::new(),
                labels: std::collections::HashMap::new(),
            };
            match membership.add_peer(peer) {
                Ok(()) => Response::success(serde_json::json!({ "node_id": node_id })),
                Err(e) => Response::error(format!("join failed: {e}")),
            }
        }
        "cluster.leave" => {
            let leave_params: ClusterLeaveParams =
                match serde_json::from_value(params) {
                    Ok(p) => p,
                    Err(e) => return Response::error(format!("invalid params: {e}")),
                };

            let k = kernel.read().await;
            let membership = k.cluster_membership();
            match membership.remove_peer(&leave_params.node_id) {
                Ok(_) => Response::success(serde_json::json!("ok")),
                Err(e) => Response::error(format!("leave failed: {e}")),
            }
        }
        "cluster.health" => {
            let k = kernel.read().await;
            let membership = k.cluster_membership();
            let peers = membership.list_peers();
            let health: Vec<serde_json::Value> = peers
                .iter()
                .map(|(id, state, _)| {
                    serde_json::json!({
                        "node_id": id,
                        "healthy": matches!(state, clawft_kernel::NodeState::Active),
                        "state": state.to_string(),
                    })
                })
                .collect();
            Response::success(serde_json::to_value(health).unwrap())
        }
        "cluster.shards" => {
            // Shards are only available with the cluster feature
            Response::success(serde_json::json!([]))
        }
        "ping" => Response::success(serde_json::json!("pong")),
        other => Response::error(format!("unknown method: {other}")),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn socket_path_resolves() {
        let path = crate::protocol::socket_path();
        assert!(path.to_string_lossy().ends_with("kernel.sock"));
    }
}
