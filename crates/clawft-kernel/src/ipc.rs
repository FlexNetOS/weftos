//! Kernel IPC subsystem.
//!
//! [`KernelIpc`] wraps the existing [`MessageBus`] from `clawft-core`,
//! adding typed [`KernelMessage`] envelopes and PID-based routing.
//! The underlying message bus channels are reused (no new channels).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;

use clawft_core::bus::MessageBus;

use crate::error::KernelError;
use crate::process::Pid;

/// Target for a kernel message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTarget {
    /// Send to a specific process by PID.
    Process(Pid),
    /// Broadcast to all processes.
    Broadcast,
    /// Send to a named service.
    Service(String),
    /// Send to the kernel itself.
    Kernel,
}

/// Payload types for kernel messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Plain text message.
    Text(String),
    /// Structured JSON data.
    Json(serde_json::Value),
    /// System control signal.
    Signal(KernelSignal),
}

/// Kernel control signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelSignal {
    /// Request a process to shut down gracefully.
    Shutdown,
    /// Request a process to suspend.
    Suspend,
    /// Request a process to resume from suspension.
    Resume,
    /// Heartbeat / keep-alive ping.
    Ping,
    /// Response to a heartbeat ping.
    Pong,
}

/// A typed message envelope for kernel IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelMessage {
    /// Unique message identifier.
    pub id: String,
    /// Sender PID (0 = kernel).
    pub from: Pid,
    /// Target for delivery.
    pub target: MessageTarget,
    /// Message payload.
    pub payload: MessagePayload,
    /// Creation timestamp.
    pub timestamp: DateTime<Utc>,
}

impl KernelMessage {
    /// Create a new kernel message.
    pub fn new(from: Pid, target: MessageTarget, payload: MessagePayload) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            target,
            payload,
            timestamp: Utc::now(),
        }
    }

    /// Create a text message.
    pub fn text(from: Pid, target: MessageTarget, text: impl Into<String>) -> Self {
        Self::new(from, target, MessagePayload::Text(text.into()))
    }

    /// Create a signal message.
    pub fn signal(from: Pid, target: MessageTarget, signal: KernelSignal) -> Self {
        Self::new(from, target, MessagePayload::Signal(signal))
    }
}

/// Kernel IPC subsystem wrapping the core MessageBus.
///
/// Adds kernel-level message envelope (type, routing, timestamps)
/// on top of the existing broadcast channel infrastructure.
pub struct KernelIpc {
    bus: Arc<MessageBus>,
}

impl KernelIpc {
    /// Create a new KernelIpc wrapping the given MessageBus.
    pub fn new(bus: Arc<MessageBus>) -> Self {
        Self { bus }
    }

    /// Get a reference to the underlying MessageBus.
    pub fn bus(&self) -> &Arc<MessageBus> {
        &self.bus
    }

    /// Send a kernel message.
    ///
    /// Currently serializes the message to JSON and publishes it
    /// as an inbound message on the bus. Future versions (K2) will
    /// implement PID-based routing and topic subscriptions.
    pub fn send(&self, msg: &KernelMessage) -> Result<(), KernelError> {
        debug!(
            id = %msg.id,
            from = msg.from,
            "sending kernel message"
        );

        let json = serde_json::to_string(msg)
            .map_err(|e| KernelError::Ipc(format!("failed to serialize message: {e}")))?;

        // For now, publish as an inbound message. The A2A routing (K2)
        // will replace this with proper PID-based delivery.
        let inbound = clawft_types::event::InboundMessage {
            channel: "kernel-ipc".to_owned(),
            sender_id: format!("pid-{}", msg.from),
            chat_id: msg.id.clone(),
            content: json,
            timestamp: msg.timestamp,
            media: vec![],
            metadata: std::collections::HashMap::new(),
        };

        self.bus
            .publish_inbound(inbound)
            .map_err(|e| KernelError::Ipc(format!("bus publish failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_message_text() {
        let msg = KernelMessage::text(0, MessageTarget::Process(1), "hello");
        assert_eq!(msg.from, 0);
        assert!(matches!(msg.target, MessageTarget::Process(1)));
        assert!(matches!(msg.payload, MessagePayload::Text(ref t) if t == "hello"));
    }

    #[test]
    fn kernel_message_signal() {
        let msg = KernelMessage::signal(0, MessageTarget::Broadcast, KernelSignal::Shutdown);
        assert!(matches!(msg.target, MessageTarget::Broadcast));
        assert!(matches!(
            msg.payload,
            MessagePayload::Signal(KernelSignal::Shutdown)
        ));
    }

    #[test]
    fn kernel_message_json_payload() {
        let payload = MessagePayload::Json(serde_json::json!({"key": "value"}));
        let msg = KernelMessage::new(1, MessageTarget::Kernel, payload);
        assert!(matches!(msg.payload, MessagePayload::Json(_)));
    }

    #[test]
    fn message_serde_roundtrip() {
        let msg = KernelMessage::text(5, MessageTarget::Service("health".into()), "check");
        let json = serde_json::to_string(&msg).unwrap();
        let restored: KernelMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, msg.id);
        assert_eq!(restored.from, 5);
    }

    #[tokio::test]
    async fn ipc_send() {
        let bus = Arc::new(MessageBus::new());
        let ipc = KernelIpc::new(bus.clone());

        let msg = KernelMessage::text(0, MessageTarget::Process(1), "test");
        ipc.send(&msg).unwrap();

        // Should be consumable from the bus
        let received = bus.consume_inbound().await.unwrap();
        assert_eq!(received.channel, "kernel-ipc");
        assert_eq!(received.sender_id, "pid-0");
    }

    #[test]
    fn ipc_bus_ref() {
        let bus = Arc::new(MessageBus::new());
        let ipc = KernelIpc::new(bus.clone());
        assert!(Arc::ptr_eq(ipc.bus(), &bus));
    }

    #[test]
    fn message_target_variants() {
        let targets = vec![
            MessageTarget::Process(1),
            MessageTarget::Broadcast,
            MessageTarget::Service("test".into()),
            MessageTarget::Kernel,
        ];
        for target in targets {
            let json = serde_json::to_string(&target).unwrap();
            let _: MessageTarget = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn kernel_signal_variants() {
        let signals = vec![
            KernelSignal::Shutdown,
            KernelSignal::Suspend,
            KernelSignal::Resume,
            KernelSignal::Ping,
            KernelSignal::Pong,
        ];
        for signal in signals {
            let json = serde_json::to_string(&signal).unwrap();
            let _: KernelSignal = serde_json::from_str(&json).unwrap();
        }
    }
}
