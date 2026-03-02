//! Built-in kernel agent work loop.
//!
//! Every daemon-spawned agent runs this loop. It receives messages
//! from the A2ARouter inbox and processes built-in commands.

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use crate::a2a::A2ARouter;
use crate::cron::CronService;
use crate::ipc::{KernelMessage, MessagePayload, MessageTarget};
use crate::process::Pid;

/// Run the built-in kernel agent work loop.
///
/// The agent:
/// 1. Receives messages from its A2ARouter inbox
/// 2. Processes built-in commands dispatched as JSON `{"cmd": "..."}` payloads
/// 3. Sends responses back via A2ARouter
/// 4. Exits when the cancellation token is triggered
///
/// Returns an exit code (0 = normal shutdown).
pub async fn kernel_agent_loop(
    pid: Pid,
    cancel: CancellationToken,
    mut inbox: mpsc::Receiver<KernelMessage>,
    a2a: Arc<A2ARouter>,
    cron: Arc<CronService>,
    #[cfg(feature = "exochain")] chain: Option<Arc<crate::chain::ChainManager>>,
) -> i32 {
    let started = Instant::now();
    debug!(pid, "agent loop started");

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                debug!(pid, "agent loop cancelled");
                return 0;
            }
            msg = inbox.recv() => {
                match msg {
                    Some(message) => {
                        handle_message(
                            pid,
                            &message,
                            &a2a,
                            &cron,
                            #[cfg(feature = "exochain")]
                            chain.as_deref(),
                            &started,
                        ).await;
                    }
                    None => {
                        // Inbox closed — shutdown
                        debug!(pid, "inbox closed, exiting");
                        return 0;
                    }
                }
            }
        }
    }
}

/// Handle a single inbound message.
async fn handle_message(
    pid: Pid,
    msg: &KernelMessage,
    a2a: &A2ARouter,
    cron: &CronService,
    #[cfg(feature = "exochain")] chain: Option<&crate::chain::ChainManager>,
    started: &Instant,
) {
    // Extract command from payload — supports JSON, Text, and RVF envelopes.
    let cmd_value = match &msg.payload {
        MessagePayload::Json(v) => v.clone(),
        MessagePayload::Text(text) => {
            // Try parsing text as JSON, otherwise treat as plain text
            match serde_json::from_str::<serde_json::Value>(text) {
                Ok(v) => v,
                Err(_) => serde_json::json!({"cmd": "echo", "text": text}),
            }
        }
        MessagePayload::Rvf { segment_type, data } => {
            // Decode RVF-typed payloads:
            //   0x40 (ExochainEvent) — treat inner CBOR/JSON as command
            //   Other — wrap as a typed envelope for the agent
            debug!(pid, segment_type, data_len = data.len(), "received RVF payload");

            // With exochain: try CBOR decode first (rvf-wire format), then JSON
            #[cfg(feature = "exochain")]
            {
                if let Ok(val) = ciborium::from_reader::<ciborium::Value, _>(&data[..]) {
                    let json_str = serde_json::to_string(&val).unwrap_or_default();
                    match serde_json::from_str::<serde_json::Value>(&json_str) {
                        Ok(v) => v,
                        Err(_) => serde_json::json!({
                            "cmd": "rvf.recv",
                            "segment_type": segment_type,
                            "data_len": data.len(),
                        }),
                    }
                } else if let Ok(v) = serde_json::from_slice::<serde_json::Value>(data) {
                    v
                } else {
                    serde_json::json!({
                        "cmd": "rvf.recv",
                        "segment_type": segment_type,
                        "data_len": data.len(),
                    })
                }
            }
            // Without exochain: try JSON decode, fall back to rvf.recv
            #[cfg(not(feature = "exochain"))]
            {
                if let Ok(v) = serde_json::from_slice::<serde_json::Value>(data) {
                    v
                } else {
                    serde_json::json!({
                        "cmd": "rvf.recv",
                        "segment_type": segment_type,
                        "data_len": data.len(),
                    })
                }
            }
        }
        _ => {
            debug!(pid, "ignoring signal message");
            return;
        }
    };

    let cmd = cmd_value
        .get("cmd")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let response = match cmd {
        "ping" => {
            let uptime_ms = started.elapsed().as_millis() as u64;
            serde_json::json!({
                "status": "ok",
                "pid": pid,
                "uptime_ms": uptime_ms,
            })
        }
        "cron.add" => {
            let name = cmd_value
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unnamed")
                .to_string();
            let interval_secs = cmd_value
                .get("interval_secs")
                .and_then(|v| v.as_u64())
                .unwrap_or(60);
            let command = cmd_value
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("ping")
                .to_string();
            let target_pid = cmd_value
                .get("target_pid")
                .and_then(|v| v.as_u64());

            let job = cron.add_job(name, interval_secs, command, target_pid);

            #[cfg(feature = "exochain")]
            if let Some(cm) = chain {
                cm.append(
                    "cron",
                    "cron.add",
                    Some(serde_json::json!({
                        "job_id": job.id,
                        "name": job.name,
                        "interval_secs": job.interval_secs,
                        "via_agent": pid,
                    })),
                );
            }

            serde_json::to_value(&job).unwrap_or_default()
        }
        "cron.list" => {
            let jobs = cron.list_jobs();
            serde_json::to_value(&jobs).unwrap_or_default()
        }
        "cron.remove" => {
            let id = cmd_value
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match cron.remove_job(id) {
                Some(job) => {
                    #[cfg(feature = "exochain")]
                    if let Some(cm) = chain {
                        cm.append(
                            "cron",
                            "cron.remove",
                            Some(serde_json::json!({
                                "job_id": job.id,
                                "name": job.name,
                                "via_agent": pid,
                            })),
                        );
                    }
                    serde_json::json!({"removed": true, "job_id": job.id})
                }
                None => serde_json::json!({"removed": false, "error": "job not found"}),
            }
        }
        "exec" => {
            // Placeholder for K2 tool execution
            let text = cmd_value
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("(no input)");
            serde_json::json!({
                "status": "ok",
                "echo": text,
                "pid": pid,
            })
        }
        "echo" => {
            let text = cmd_value
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            serde_json::json!({"echo": text, "pid": pid})
        }
        "rvf.recv" => {
            // Acknowledge receipt of an RVF-typed payload
            let seg_type = cmd_value
                .get("segment_type")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let data_len = cmd_value
                .get("data_len")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            serde_json::json!({
                "status": "ok",
                "cmd": "rvf.recv",
                "segment_type": seg_type,
                "data_len": data_len,
                "pid": pid,
            })
        }
        unknown => {
            serde_json::json!({
                "error": format!("unknown command: {unknown}"),
                "pid": pid,
            })
        }
    };

    // Send response back to sender via chain-logged path
    let reply = KernelMessage::with_correlation(
        pid,
        MessageTarget::Process(msg.from),
        MessagePayload::Json(response),
        msg.id.clone(),
    );

    #[cfg(feature = "exochain")]
    {
        if let Err(e) = a2a.send_checked(reply, chain).await {
            warn!(pid, error = %e, "failed to send reply");
        }
    }
    #[cfg(not(feature = "exochain"))]
    {
        if let Err(e) = a2a.send(reply).await {
            warn!(pid, error = %e, "failed to send reply");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{AgentCapabilities, CapabilityChecker};
    use crate::process::{ProcessEntry, ProcessState, ProcessTable, ResourceUsage};
    use crate::topic::TopicRouter;

    fn setup() -> (Arc<A2ARouter>, Arc<CronService>, Arc<ProcessTable>) {
        let pt = Arc::new(ProcessTable::new(64));

        // Insert a "kernel" process at PID 0 for message routing
        let kernel_entry = ProcessEntry {
            pid: 0,
            agent_id: "kernel".into(),
            state: ProcessState::Running,
            capabilities: AgentCapabilities::default(),
            resource_usage: ResourceUsage::default(),
            cancel_token: CancellationToken::new(),
            parent_pid: None,
        };
        pt.insert_with_pid(kernel_entry).unwrap();

        let checker = Arc::new(CapabilityChecker::new(pt.clone()));
        let topic_router = Arc::new(TopicRouter::new(pt.clone()));
        let a2a = Arc::new(A2ARouter::new(pt.clone(), checker, topic_router));
        let cron = Arc::new(CronService::new());
        (a2a, cron, pt)
    }

    fn spawn_agent(
        pt: &ProcessTable,
        a2a: &A2ARouter,
        agent_id: &str,
    ) -> (Pid, mpsc::Receiver<KernelMessage>) {
        let entry = ProcessEntry {
            pid: 0,
            agent_id: agent_id.into(),
            state: ProcessState::Running,
            capabilities: AgentCapabilities::default(),
            resource_usage: ResourceUsage::default(),
            cancel_token: CancellationToken::new(),
            parent_pid: None,
        };
        let pid = pt.insert(entry).unwrap();
        let rx = a2a.create_inbox(pid);
        (pid, rx)
    }

    #[tokio::test]
    async fn ping_command() {
        let (a2a, cron, pt) = setup();
        let (agent_pid, inbox) = spawn_agent(&pt, &a2a, "test-agent");
        let mut kernel_inbox = a2a.create_inbox(0);

        let cancel = CancellationToken::new();
        let cancel2 = cancel.clone();

        let handle = tokio::spawn({
            let a2a = a2a.clone();
            let cron = cron.clone();
            async move {
                kernel_agent_loop(
                    agent_pid,
                    cancel2,
                    inbox,
                    a2a,
                    cron,
                    #[cfg(feature = "exochain")]
                    None,
                )
                .await
            }
        });

        // Send ping from kernel (PID 0)
        let msg = KernelMessage::new(
            0,
            MessageTarget::Process(agent_pid),
            MessagePayload::Json(serde_json::json!({"cmd": "ping"})),
        );
        a2a.send(msg).await.unwrap();

        // Wait for reply
        let reply = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            kernel_inbox.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        if let MessagePayload::Json(v) = &reply.payload {
            assert_eq!(v["status"], "ok");
            assert_eq!(v["pid"], agent_pid);
        } else {
            panic!("expected JSON reply");
        }

        cancel.cancel();
        let code = handle.await.unwrap();
        assert_eq!(code, 0);
    }

    #[tokio::test]
    async fn unknown_command() {
        let (a2a, cron, pt) = setup();
        let (agent_pid, inbox) = spawn_agent(&pt, &a2a, "test-agent");
        let mut kernel_inbox = a2a.create_inbox(0);

        let cancel = CancellationToken::new();
        let cancel2 = cancel.clone();

        let handle = tokio::spawn({
            let a2a = a2a.clone();
            let cron = cron.clone();
            async move {
                kernel_agent_loop(
                    agent_pid,
                    cancel2,
                    inbox,
                    a2a,
                    cron,
                    #[cfg(feature = "exochain")]
                    None,
                )
                .await
            }
        });

        let msg = KernelMessage::new(
            0,
            MessageTarget::Process(agent_pid),
            MessagePayload::Json(serde_json::json!({"cmd": "nosuch"})),
        );
        a2a.send(msg).await.unwrap();

        let reply = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            kernel_inbox.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        if let MessagePayload::Json(v) = &reply.payload {
            assert!(v["error"].as_str().unwrap().contains("unknown command"));
        } else {
            panic!("expected JSON reply");
        }

        cancel.cancel();
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn cron_add_via_agent() {
        let (a2a, cron, pt) = setup();
        let (agent_pid, inbox) = spawn_agent(&pt, &a2a, "test-agent");
        let mut kernel_inbox = a2a.create_inbox(0);

        let cancel = CancellationToken::new();
        let cancel2 = cancel.clone();

        let handle = tokio::spawn({
            let a2a = a2a.clone();
            let cron = cron.clone();
            async move {
                kernel_agent_loop(
                    agent_pid,
                    cancel2,
                    inbox,
                    a2a,
                    cron,
                    #[cfg(feature = "exochain")]
                    None,
                )
                .await
            }
        });

        let msg = KernelMessage::new(
            0,
            MessageTarget::Process(agent_pid),
            MessagePayload::Json(serde_json::json!({
                "cmd": "cron.add",
                "name": "test-job",
                "interval_secs": 30,
                "command": "health",
            })),
        );
        a2a.send(msg).await.unwrap();

        let reply = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            kernel_inbox.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        if let MessagePayload::Json(v) = &reply.payload {
            assert_eq!(v["name"], "test-job");
            assert!(v["id"].as_str().is_some());
        } else {
            panic!("expected JSON reply");
        }

        // Verify job was actually added
        assert_eq!(cron.job_count(), 1);

        cancel.cancel();
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn cancellation_exits_cleanly() {
        let (a2a, cron, pt) = setup();
        let (agent_pid, inbox) = spawn_agent(&pt, &a2a, "test-agent");

        let cancel = CancellationToken::new();
        let cancel2 = cancel.clone();

        let handle = tokio::spawn({
            let a2a = a2a.clone();
            let cron = cron.clone();
            async move {
                kernel_agent_loop(
                    agent_pid,
                    cancel2,
                    inbox,
                    a2a,
                    cron,
                    #[cfg(feature = "exochain")]
                    None,
                )
                .await
            }
        });

        cancel.cancel();
        let code = handle.await.unwrap();
        assert_eq!(code, 0);
    }

    #[tokio::test]
    async fn rvf_json_payload_processed() {
        let (a2a, cron, pt) = setup();
        let (agent_pid, inbox) = spawn_agent(&pt, &a2a, "test-agent");
        let mut kernel_inbox = a2a.create_inbox(0);

        let cancel = CancellationToken::new();
        let cancel2 = cancel.clone();

        let handle = tokio::spawn({
            let a2a = a2a.clone();
            let cron = cron.clone();
            async move {
                kernel_agent_loop(
                    agent_pid,
                    cancel2,
                    inbox,
                    a2a,
                    cron,
                    #[cfg(feature = "exochain")]
                    None,
                )
                .await
            }
        });

        // Send an RVF payload containing JSON bytes (e.g. `{"cmd":"ping"}`)
        let json_bytes = serde_json::to_vec(&serde_json::json!({"cmd": "ping"})).unwrap();
        let msg = KernelMessage::new(
            0,
            MessageTarget::Process(agent_pid),
            MessagePayload::Rvf {
                segment_type: 0x40,
                data: json_bytes,
            },
        );
        a2a.send(msg).await.unwrap();

        let reply = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            kernel_inbox.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        if let MessagePayload::Json(v) = &reply.payload {
            assert_eq!(v["status"], "ok");
            assert_eq!(v["pid"], agent_pid);
        } else {
            panic!("expected JSON reply to RVF-wrapped ping");
        }

        cancel.cancel();
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn rvf_opaque_binary_acknowledged() {
        let (a2a, cron, pt) = setup();
        let (agent_pid, inbox) = spawn_agent(&pt, &a2a, "test-agent");
        let mut kernel_inbox = a2a.create_inbox(0);

        let cancel = CancellationToken::new();
        let cancel2 = cancel.clone();

        let handle = tokio::spawn({
            let a2a = a2a.clone();
            let cron = cron.clone();
            async move {
                kernel_agent_loop(
                    agent_pid,
                    cancel2,
                    inbox,
                    a2a,
                    cron,
                    #[cfg(feature = "exochain")]
                    None,
                )
                .await
            }
        });

        // Send raw binary that isn't valid JSON or CBOR
        let msg = KernelMessage::new(
            0,
            MessageTarget::Process(agent_pid),
            MessagePayload::Rvf {
                segment_type: 0x42,
                data: vec![0xDE, 0xAD, 0xBE, 0xEF],
            },
        );
        a2a.send(msg).await.unwrap();

        let reply = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            kernel_inbox.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        if let MessagePayload::Json(v) = &reply.payload {
            assert_eq!(v["cmd"], "rvf.recv");
            assert_eq!(v["segment_type"], 0x42);
            assert_eq!(v["data_len"], 4);
        } else {
            panic!("expected JSON reply acknowledging RVF binary");
        }

        cancel.cancel();
        handle.await.unwrap();
    }
}
