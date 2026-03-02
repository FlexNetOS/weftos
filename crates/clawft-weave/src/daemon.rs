//! Kernel daemon — persistent kernel process with Unix socket RPC.
//!
//! The daemon boots a [`Kernel`], then listens on a Unix domain socket
//! for JSON-RPC requests. This is the native transport layer; the
//! kernel itself is platform-agnostic and could be wrapped in
//! WebSocket, TCP, or `postMessage` for other environments.

use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use clawft_kernel::{Kernel, KernelState};
use clawft_platform::NativePlatform;
use clawft_types::config::{Config, KernelConfig};

use crate::protocol::{
    self, AgentInspectResult, AgentSendParams, AgentSpawnParams, AgentSpawnResult, AgentStopParams,
    AgentRestartParams, ClusterJoinParams, ClusterLeaveParams, ClusterNodeInfo,
    ClusterStatusResult, CronAddParams, CronJobInfo, CronRemoveParams, KernelStatusResult,
    LogEntry, LogsParams, ProcessInfo, Request, Response, ServiceInfo,
};
#[cfg(feature = "exochain")]
use crate::protocol::{
    ChainEventInfo, ChainExportParams, ChainLocalParams, ChainStatusResult, ChainVerifyResult,
    ResourceInspectParams, ResourceNodeInfo, ResourceStatsResult,
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

    // Cron tick loop — fires overdue jobs every second
    let cron_kernel = Arc::clone(&kernel);
    let mut cron_shutdown_rx = shutdown_rx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let k = cron_kernel.read().await;
                    let cron = k.cron_service();
                    let tick_result = cron.tick();

                    // Dispatch fired jobs via chain-logged A2ARouter
                    for job_id in &tick_result.fired {
                        if let Some(job) = cron.job_snapshot(job_id)
                            && let Some(target_pid) = job.target_pid
                        {
                            // Log cron.fire event first (records scheduling intent)
                            #[cfg(feature = "exochain")]
                            if let Some(cm) = k.chain_manager() {
                                cm.append(
                                    "cron",
                                    "cron.fire",
                                    Some(serde_json::json!({
                                        "job_id": job.id,
                                        "name": job.name,
                                        "fire_count": job.fire_count,
                                        "target_pid": job.target_pid,
                                    })),
                                );
                            }

                            // Dispatch via send_checked (logs ipc.send in chain)
                            let msg = clawft_kernel::KernelMessage::new(
                                0,
                                clawft_kernel::MessageTarget::Process(target_pid),
                                clawft_kernel::MessagePayload::Json(serde_json::json!({
                                    "cmd": job.command,
                                    "cron_job_id": job.id,
                                    "cron_job_name": job.name,
                                })),
                            );
                            let a2a = k.a2a_router().clone();

                            #[cfg(feature = "exochain")]
                            {
                                let chain = k.chain_manager();
                                if let Err(e) = a2a.send_checked(msg, chain.map(|c| c.as_ref())).await {
                                    warn!(job_id = %job.id, error = %e, "cron: failed to send to target");
                                }
                            }
                            #[cfg(not(feature = "exochain"))]
                            {
                                if let Err(e) = a2a.send(msg).await {
                                    warn!(job_id = %job.id, error = %e, "cron: failed to send to target");
                                }
                            }
                        }
                    }
                }
                _ = cron_shutdown_rx.changed() => {
                    if *cron_shutdown_rx.borrow() {
                        debug!("cron tick loop shutting down");
                        break;
                    }
                }
            }
        }
    });

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
///
/// Detects the connection mode by reading the first 4 bytes:
/// - `RVFS` → RVF-framed protocol (content-hash verified segments)
/// - Anything else → legacy line-delimited JSON (bytes prepended to first line)
async fn handle_connection(
    mut stream: tokio::net::UnixStream,
    kernel: Arc<tokio::sync::RwLock<Kernel<NativePlatform>>>,
    shutdown_tx: watch::Sender<bool>,
) {
    // Read 4-byte header to detect protocol mode.
    let mut header = [0u8; 4];
    if stream.read_exact(&mut header).await.is_err() {
        return; // connection closed immediately
    }

    #[cfg(feature = "rvf-rpc")]
    if &header == b"RVFS" {
        return handle_rvf_connection(stream, kernel, shutdown_tx).await;
    }

    // JSON mode: the 4 header bytes are the start of the first JSON line.
    handle_json_connection(header, stream, kernel, shutdown_tx).await;
}

/// Dispatch a single JSON line and write the response.
///
/// Returns `true` to continue reading, `false` to stop (write error).
async fn dispatch_json_line(
    line: &str,
    kernel: &Arc<tokio::sync::RwLock<Kernel<NativePlatform>>>,
    shutdown_tx: &watch::Sender<bool>,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> bool {
    let line = line.trim();
    if line.is_empty() {
        return true;
    }

    let response = match serde_json::from_str::<Request>(line) {
        Ok(req) => {
            let id = req.id.clone();
            dispatch(
                req.method,
                req.params,
                Arc::clone(kernel),
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
        return false;
    }
    true
}

/// Handle a legacy line-delimited JSON connection.
///
/// The `prefix` bytes were consumed during protocol detection and form
/// the beginning of the first JSON line on the wire.
async fn handle_json_connection(
    prefix: [u8; 4],
    stream: tokio::net::UnixStream,
    kernel: Arc<tokio::sync::RwLock<Kernel<NativePlatform>>>,
    shutdown_tx: watch::Sender<bool>,
) {
    let (reader, mut writer) = stream.into_split();
    let mut buf = BufReader::new(reader);

    // Reconstruct the first line: the 4-byte prefix + the rest until '\n'.
    let mut rest_of_first = String::new();
    if buf.read_line(&mut rest_of_first).await.is_err() {
        return;
    }
    let first_line = format!("{}{}", String::from_utf8_lossy(&prefix), rest_of_first);
    if !dispatch_json_line(&first_line, &kernel, &shutdown_tx, &mut writer).await {
        return;
    }

    // Process remaining lines normally.
    loop {
        let mut line = String::new();
        match buf.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }
        if !dispatch_json_line(&line, &kernel, &shutdown_tx, &mut writer).await {
            break;
        }
    }
}

/// Handle an RVF-framed connection.
///
/// Each request/response is an RVF Meta segment with content-hash
/// integrity. Responses carry the SEALED flag; requests do not.
/// Uses the same `dispatch()` function as JSON mode.
#[cfg(feature = "rvf-rpc")]
async fn handle_rvf_connection(
    stream: tokio::net::UnixStream,
    kernel: Arc<tokio::sync::RwLock<Kernel<NativePlatform>>>,
    shutdown_tx: watch::Sender<bool>,
) {
    use crate::rvf_codec::{RvfFrameReader, RvfFrameWriter};
    use crate::rvf_rpc;

    let (reader, writer) = stream.into_split();
    let mut frame_reader = RvfFrameReader::new(reader);
    let mut frame_writer = RvfFrameWriter::new(writer);
    let mut next_id: u64 = 1;

    loop {
        let frame = match frame_reader.read_frame().await {
            Ok(Some(f)) => f,
            Ok(None) => break, // clean EOF
            Err(e) => {
                debug!("RVF read error: {e}");
                break;
            }
        };

        let response = match rvf_rpc::decode_request(&frame) {
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
            Err(e) => Response::error(format!("invalid RVF request: {e}")),
        };

        let (seg_type, payload, flags, segment_id) =
            rvf_rpc::encode_response(&response, next_id);
        next_id += 1;

        if let Err(e) = frame_writer
            .write_frame(seg_type, &payload, flags, segment_id)
            .await
        {
            debug!("RVF write error: {e}");
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
        "chain.status" => {
            #[cfg(feature = "exochain")]
            {
                let k = kernel.read().await;
                if let Some(cm) = k.chain_manager() {
                    let status = cm.status();
                    let hash_hex: String =
                        status.last_hash.iter().map(|b| format!("{b:02x}")).collect();
                    let result = ChainStatusResult {
                        chain_id: status.chain_id,
                        sequence: status.sequence,
                        event_count: status.event_count,
                        checkpoint_count: status.checkpoint_count,
                        events_since_checkpoint: status.events_since_checkpoint,
                        last_hash: hash_hex,
                    };
                    Response::success(serde_json::to_value(result).unwrap())
                } else {
                    Response::error("chain not enabled")
                }
            }
            #[cfg(not(feature = "exochain"))]
            Response::error("exochain feature not enabled")
        }
        "chain.local" => {
            #[cfg(feature = "exochain")]
            {
                let local_params: ChainLocalParams = serde_json::from_value(params)
                    .unwrap_or(ChainLocalParams { count: 20 });
                let k = kernel.read().await;
                if let Some(cm) = k.chain_manager() {
                    let events = cm.tail(local_params.count);
                    let infos: Vec<ChainEventInfo> = events
                        .iter()
                        .map(|e| {
                            let hash_hex: String =
                                e.hash.iter().map(|b| format!("{b:02x}")).collect();
                            ChainEventInfo {
                                sequence: e.sequence,
                                chain_id: e.chain_id,
                                timestamp: e.timestamp.to_rfc3339(),
                                source: e.source.clone(),
                                kind: e.kind.clone(),
                                hash: hash_hex,
                            }
                        })
                        .collect();
                    Response::success(serde_json::to_value(infos).unwrap())
                } else {
                    Response::error("chain not enabled")
                }
            }
            #[cfg(not(feature = "exochain"))]
            Response::error("exochain feature not enabled")
        }
        "chain.checkpoint" => {
            #[cfg(feature = "exochain")]
            {
                let k = kernel.read().await;
                if let Some(cm) = k.chain_manager() {
                    let cp = cm.checkpoint();
                    Response::success(serde_json::to_value(cp).unwrap())
                } else {
                    Response::error("chain not enabled")
                }
            }
            #[cfg(not(feature = "exochain"))]
            Response::error("exochain feature not enabled")
        }
        "chain.verify" => {
            #[cfg(feature = "exochain")]
            {
                let k = kernel.read().await;
                if let Some(cm) = k.chain_manager() {
                    let result = cm.verify_integrity();
                    let proto_result = ChainVerifyResult {
                        valid: result.valid,
                        event_count: result.event_count,
                        errors: result.errors,
                    };
                    Response::success(serde_json::to_value(proto_result).unwrap())
                } else {
                    Response::error("chain not enabled")
                }
            }
            #[cfg(not(feature = "exochain"))]
            Response::error("exochain feature not enabled")
        }
        "chain.export" => {
            #[cfg(feature = "exochain")]
            {
                let export_params: ChainExportParams = serde_json::from_value(params)
                    .unwrap_or(ChainExportParams {
                        format: "json".into(),
                        output: None,
                    });
                let k = kernel.read().await;
                if let Some(cm) = k.chain_manager() {
                    match export_params.format.as_str() {
                        "rvf" => {
                            let default_path =
                                protocol::runtime_dir().join("chain").join("export.rvf");
                            let output_path = export_params
                                .output
                                .map(std::path::PathBuf::from)
                                .unwrap_or(default_path);
                            match cm.save_to_rvf(&output_path) {
                                Ok(()) => Response::success(serde_json::json!({
                                    "format": "rvf",
                                    "path": output_path.display().to_string(),
                                })),
                                Err(e) => Response::error(format!("RVF export failed: {e}")),
                            }
                        }
                        _ => {
                            let events = cm.tail(0);
                            let infos: Vec<ChainEventInfo> = events
                                .iter()
                                .map(|e| {
                                    let hash_hex: String =
                                        e.hash.iter().map(|b| format!("{b:02x}")).collect();
                                    ChainEventInfo {
                                        sequence: e.sequence,
                                        chain_id: e.chain_id,
                                        timestamp: e.timestamp.to_rfc3339(),
                                        source: e.source.clone(),
                                        kind: e.kind.clone(),
                                        hash: hash_hex,
                                    }
                                })
                                .collect();
                            Response::success(serde_json::to_value(infos).unwrap())
                        }
                    }
                } else {
                    Response::error("chain not enabled")
                }
            }
            #[cfg(not(feature = "exochain"))]
            Response::error("exochain feature not enabled")
        }
        "resource.tree" => {
            #[cfg(feature = "exochain")]
            {
                let k = kernel.read().await;
                if let Some(tm) = k.tree_manager() {
                    let tree = tm.tree().lock().unwrap();
                    let mut nodes: Vec<ResourceNodeInfo> = tree
                        .iter()
                        .map(|(id, node)| {
                            let hash_hex: String =
                                node.merkle_hash.iter().map(|b| format!("{b:02x}")).collect();
                            ResourceNodeInfo {
                                id: id.to_string(),
                                kind: format!("{:?}", node.kind),
                                parent: node.parent.as_ref().map(|p| p.to_string()),
                                children: node.children.iter().map(|c| c.to_string()).collect(),
                                metadata: serde_json::to_value(&node.metadata).unwrap_or_default(),
                                merkle_hash: hash_hex,
                            }
                        })
                        .collect();
                    nodes.sort_by(|a, b| a.id.cmp(&b.id));
                    Response::success(serde_json::to_value(nodes).unwrap())
                } else {
                    Response::error("resource tree not enabled")
                }
            }
            #[cfg(not(feature = "exochain"))]
            Response::error("exochain feature not enabled")
        }
        "resource.inspect" => {
            #[cfg(feature = "exochain")]
            {
                let inspect_params: ResourceInspectParams = match serde_json::from_value(params) {
                    Ok(p) => p,
                    Err(e) => return Response::error(format!("invalid params: {e}")),
                };
                let k = kernel.read().await;
                if let Some(tm) = k.tree_manager() {
                    let tree = tm.tree().lock().unwrap();
                    let rid = exo_resource_tree::ResourceId::new(&inspect_params.path);
                    if let Some(node) = tree.get(&rid) {
                        let hash_hex: String =
                            node.merkle_hash.iter().map(|b| format!("{b:02x}")).collect();
                        let info = ResourceNodeInfo {
                            id: node.id.to_string(),
                            kind: format!("{:?}", node.kind),
                            parent: node.parent.as_ref().map(|p| p.to_string()),
                            children: node.children.iter().map(|c| c.to_string()).collect(),
                            metadata: serde_json::to_value(&node.metadata).unwrap_or_default(),
                            merkle_hash: hash_hex,
                        };
                        Response::success(serde_json::to_value(info).unwrap())
                    } else {
                        Response::error(format!("resource not found: {}", inspect_params.path))
                    }
                } else {
                    Response::error("resource tree not enabled")
                }
            }
            #[cfg(not(feature = "exochain"))]
            Response::error("exochain feature not enabled")
        }
        "resource.stats" => {
            #[cfg(feature = "exochain")]
            {
                let k = kernel.read().await;
                if let Some(tm) = k.tree_manager() {
                    let tree = tm.tree().lock().unwrap();
                    let hash_hex: String =
                        tree.root_hash().iter().map(|b| format!("{b:02x}")).collect();
                    let mut namespaces = 0usize;
                    let mut services = 0usize;
                    let mut agents = 0usize;
                    let mut devices = 0usize;
                    for (_, node) in tree.iter() {
                        match node.kind {
                            exo_resource_tree::ResourceKind::Namespace => namespaces += 1,
                            exo_resource_tree::ResourceKind::Service => services += 1,
                            exo_resource_tree::ResourceKind::Agent => agents += 1,
                            exo_resource_tree::ResourceKind::Device => devices += 1,
                            _ => {}
                        }
                    }
                    let result = ResourceStatsResult {
                        total_nodes: tree.len(),
                        root_hash: hash_hex,
                        namespaces,
                        services,
                        agents,
                        devices,
                    };
                    Response::success(serde_json::to_value(result).unwrap())
                } else {
                    Response::error("resource tree not enabled")
                }
            }
            #[cfg(not(feature = "exochain"))]
            Response::error("exochain feature not enabled")
        }
        "agent.spawn" => {
            let spawn_params: AgentSpawnParams = match serde_json::from_value(params) {
                Ok(p) => p,
                Err(e) => return Response::error(format!("invalid params: {e}")),
            };
            let k = kernel.read().await;
            let request = clawft_kernel::SpawnRequest {
                agent_id: spawn_params.agent_id,
                capabilities: None,
                parent_pid: spawn_params.parent_pid,
                env: std::collections::HashMap::new(),
            };

            // Create inbox via A2ARouter before spawning
            let a2a = k.a2a_router().clone();
            let cron = k.cron_service().clone();
            #[cfg(feature = "exochain")]
            let chain = k.chain_manager().cloned();

            // Use spawn_and_run to actually execute the agent work loop
            match k.supervisor().spawn_and_run(request, {
                let a2a_clone = a2a.clone();
                let cron_clone = cron.clone();
                #[cfg(feature = "exochain")]
                let chain_clone = chain.clone();
                move |pid, cancel| {
                    let inbox = a2a_clone.create_inbox(pid);
                    async move {
                        clawft_kernel::agent_loop::kernel_agent_loop(
                            pid,
                            cancel,
                            inbox,
                            a2a_clone,
                            cron_clone,
                            #[cfg(feature = "exochain")]
                            chain_clone,
                        )
                        .await
                    }
                }
            }) {
                Ok(result) => {
                    k.event_log().info("agent", format!("spawned {} (PID {})", result.agent_id, result.pid));
                    let spawn_result = AgentSpawnResult {
                        pid: result.pid,
                        agent_id: result.agent_id,
                    };
                    Response::success(serde_json::to_value(spawn_result).unwrap())
                }
                Err(e) => Response::error(format!("spawn failed: {e}")),
            }
        }
        "agent.stop" => {
            let stop_params: AgentStopParams = match serde_json::from_value(params) {
                Ok(p) => p,
                Err(e) => return Response::error(format!("invalid params: {e}")),
            };
            let k = kernel.read().await;
            match k.supervisor().stop(stop_params.pid, stop_params.graceful) {
                Ok(()) => {
                    k.event_log().info("agent", format!("stopped PID {}", stop_params.pid));
                    Response::success(serde_json::json!("ok"))
                }
                Err(e) => Response::error(format!("stop failed: {e}")),
            }
        }
        "agent.restart" => {
            let restart_params: AgentRestartParams = match serde_json::from_value(params) {
                Ok(p) => p,
                Err(e) => return Response::error(format!("invalid params: {e}")),
            };
            let k = kernel.read().await;
            match k.supervisor().restart(restart_params.pid) {
                Ok(result) => {
                    let _ = k.process_table().update_state(result.pid, clawft_kernel::ProcessState::Running);
                    k.event_log().info("agent", format!("restarted {} (PID {} -> {})", result.agent_id, restart_params.pid, result.pid));
                    let spawn_result = AgentSpawnResult {
                        pid: result.pid,
                        agent_id: result.agent_id,
                    };
                    Response::success(serde_json::to_value(spawn_result).unwrap())
                }
                Err(e) => Response::error(format!("restart failed: {e}")),
            }
        }
        "agent.inspect" => {
            let pid = params
                .get("pid")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let k = kernel.read().await;
            match k.supervisor().inspect(pid) {
                Ok(entry) => {
                    let result = AgentInspectResult {
                        pid: entry.pid,
                        agent_id: entry.agent_id,
                        state: entry.state.to_string(),
                        memory_bytes: entry.resource_usage.memory_bytes,
                        cpu_time_ms: entry.resource_usage.cpu_time_ms,
                        parent_pid: entry.parent_pid,
                        can_spawn: entry.capabilities.can_spawn,
                        can_ipc: entry.capabilities.can_ipc,
                        can_exec_tools: entry.capabilities.can_exec_tools,
                        can_network: entry.capabilities.can_network,
                    };
                    Response::success(serde_json::to_value(result).unwrap())
                }
                Err(e) => Response::error(format!("inspect failed: {e}")),
            }
        }
        "agent.list" => {
            let k = kernel.read().await;
            let agents = k.supervisor().list_agents();
            let mut infos: Vec<ProcessInfo> = agents
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
            infos.sort_by_key(|e| e.pid);
            Response::success(serde_json::to_value(infos).unwrap())
        }
        "agent.send" => {
            let send_params: AgentSendParams = match serde_json::from_value(params) {
                Ok(p) => p,
                Err(e) => return Response::error(format!("invalid params: {e}")),
            };
            let k = kernel.read().await;

            // Try to parse message as JSON; fall back to text payload
            let payload = match serde_json::from_str::<serde_json::Value>(&send_params.message) {
                Ok(v) => clawft_kernel::MessagePayload::Json(v),
                Err(_) => clawft_kernel::MessagePayload::Text(send_params.message.clone()),
            };
            let msg = clawft_kernel::KernelMessage::new(
                0, // from kernel (PID 0)
                clawft_kernel::MessageTarget::Process(send_params.pid),
                payload,
            );

            // Route through A2ARouter with chain-logged delivery
            let a2a = k.a2a_router().clone();

            #[cfg(feature = "exochain")]
            let send_result = {
                let chain = k.chain_manager();
                a2a.send_checked(msg, chain.map(|c| c.as_ref())).await
            };
            #[cfg(not(feature = "exochain"))]
            let send_result = a2a.send(msg).await;

            match send_result {
                Ok(()) => {
                    k.event_log().info("ipc", format!("message sent to PID {}", send_params.pid));

                    // Wait briefly for a response from the agent
                    // Create a temporary inbox for kernel PID 0 if not already present
                    // (it may already exist from a previous call — A2ARouter replaces it)
                    let mut reply_rx = a2a.create_inbox(0);
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(2),
                        reply_rx.recv(),
                    )
                    .await
                    {
                        Ok(Some(reply)) => {
                            let reply_value = match &reply.payload {
                                clawft_kernel::MessagePayload::Json(v) => v.clone(),
                                clawft_kernel::MessagePayload::Text(t) => {
                                    serde_json::json!({"text": t})
                                }
                                _ => serde_json::json!({"payload": "non-json"}),
                            };
                            Response::success(reply_value)
                        }
                        Ok(None) | Err(_) => {
                            // No reply within timeout — still report send success
                            Response::success(serde_json::json!("sent"))
                        }
                    }
                }
                Err(e) => Response::error(format!("send failed: {e}")),
            }
        }
        "cron.add" => {
            let cron_params: CronAddParams = match serde_json::from_value(params) {
                Ok(p) => p,
                Err(e) => return Response::error(format!("invalid params: {e}")),
            };
            let k = kernel.read().await;
            let job = k.cron_service().add_job(
                cron_params.name,
                cron_params.interval_secs,
                cron_params.command,
                cron_params.target_pid,
            );

            #[cfg(feature = "exochain")]
            if let Some(cm) = k.chain_manager() {
                cm.append(
                    "cron",
                    "cron.add",
                    Some(serde_json::json!({
                        "job_id": job.id,
                        "name": job.name,
                        "interval_secs": job.interval_secs,
                    })),
                );
            }

            k.event_log().info("cron", format!("job added: {} ({}s)", job.name, job.interval_secs));
            let info = CronJobInfo {
                id: job.id,
                name: job.name,
                interval_secs: job.interval_secs,
                command: job.command,
                target_pid: job.target_pid,
                enabled: job.enabled,
                fire_count: job.fire_count,
                last_fired: job.last_fired.map(|t| t.to_rfc3339()),
                created_at: job.created_at.to_rfc3339(),
            };
            Response::success(serde_json::to_value(info).unwrap())
        }
        "cron.list" => {
            let k = kernel.read().await;
            let jobs = k.cron_service().list_jobs();
            let infos: Vec<CronJobInfo> = jobs
                .iter()
                .map(|j| CronJobInfo {
                    id: j.id.clone(),
                    name: j.name.clone(),
                    interval_secs: j.interval_secs,
                    command: j.command.clone(),
                    target_pid: j.target_pid,
                    enabled: j.enabled,
                    fire_count: j.fire_count,
                    last_fired: j.last_fired.map(|t| t.to_rfc3339()),
                    created_at: j.created_at.to_rfc3339(),
                })
                .collect();
            Response::success(serde_json::to_value(infos).unwrap())
        }
        "cron.remove" => {
            let remove_params: CronRemoveParams = match serde_json::from_value(params) {
                Ok(p) => p,
                Err(e) => return Response::error(format!("invalid params: {e}")),
            };
            let k = kernel.read().await;
            match k.cron_service().remove_job(&remove_params.id) {
                Some(job) => {
                    #[cfg(feature = "exochain")]
                    if let Some(cm) = k.chain_manager() {
                        cm.append(
                            "cron",
                            "cron.remove",
                            Some(serde_json::json!({
                                "job_id": job.id,
                                "name": job.name,
                            })),
                        );
                    }
                    k.event_log().info("cron", format!("job removed: {}", job.name));
                    Response::success(serde_json::json!({"removed": true, "job_id": job.id}))
                }
                None => Response::error(format!("cron job not found: {}", remove_params.id)),
            }
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
