//! Daemon client — connects to a running kernel daemon over Unix socket.
//!
//! Used by `weaver kernel status`, `weaver kernel ps`, etc. to talk
//! to a persistent daemon instead of booting an ephemeral kernel.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::protocol::{self, Request, Response};

/// A client connected to the kernel daemon.
pub struct DaemonClient {
    stream: UnixStream,
}

impl DaemonClient {
    /// Try to connect to the daemon. Returns `None` if no daemon is running.
    pub async fn connect() -> Option<Self> {
        let path = protocol::socket_path();
        let stream = UnixStream::connect(&path).await.ok()?;
        Some(Self { stream })
    }

    /// Send a request and wait for the response.
    pub async fn call(&mut self, request: Request) -> anyhow::Result<Response> {
        let mut json = serde_json::to_string(&request)?;
        json.push('\n');

        self.stream.write_all(json.as_bytes()).await?;

        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        if line.trim().is_empty() {
            anyhow::bail!("daemon closed connection without response");
        }

        let response: Response = serde_json::from_str(line.trim())?;
        Ok(response)
    }

    /// Convenience: send a no-params request.
    pub async fn simple_call(&mut self, method: &str) -> anyhow::Result<Response> {
        self.call(Request::new(method)).await
    }
}

/// Check if a daemon is running (socket exists and accepts connections).
#[allow(dead_code)]
pub async fn is_daemon_running() -> bool {
    DaemonClient::connect().await.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_returns_none_when_no_daemon() {
        // No daemon running in test environment
        let client = DaemonClient::connect().await;
        assert!(client.is_none());
    }

    #[tokio::test]
    async fn is_daemon_running_false_when_no_daemon() {
        assert!(!is_daemon_running().await);
    }
}
