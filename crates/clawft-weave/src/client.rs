//! Daemon client — connects to a running kernel daemon.
//!
//! On Unix, connects over a Unix domain socket. On other platforms,
//! `connect()` always returns `None` (daemon transport not yet available).

use crate::protocol::{Request, Response};

// ── Unix implementation ──────────────────────────────────────────

#[cfg(unix)]
mod imp {
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
}

// ── Non-Unix stub ────────────────────────────────────────────────

#[cfg(not(unix))]
mod imp {
    use crate::protocol::{Request, Response};

    /// Stub daemon client for non-Unix platforms.
    ///
    /// `connect()` always returns `None`. Windows named-pipe transport
    /// is planned for v0.2.
    pub struct DaemonClient;

    impl DaemonClient {
        /// Always returns `None` on non-Unix platforms.
        pub async fn connect() -> Option<Self> {
            None
        }

        pub async fn call(&mut self, _request: Request) -> anyhow::Result<Response> {
            anyhow::bail!("daemon not available on this platform")
        }

        pub async fn simple_call(&mut self, _method: &str) -> anyhow::Result<Response> {
            anyhow::bail!("daemon not available on this platform")
        }
    }
}

pub use imp::DaemonClient;

/// Check if a daemon is running (socket exists and accepts connections).
#[allow(dead_code)]
pub async fn is_daemon_running() -> bool {
    DaemonClient::connect().await.is_some()
}

// ── RVF-framed client (Unix only) ────────────────────────────────

#[cfg(all(unix, feature = "rvf-rpc"))]
#[allow(dead_code)]
pub struct RvfDaemonClient {
    reader: crate::rvf_codec::RvfFrameReader<tokio::net::unix::OwnedReadHalf>,
    writer: crate::rvf_codec::RvfFrameWriter<tokio::net::unix::OwnedWriteHalf>,
    next_segment_id: u64,
}

#[cfg(all(unix, feature = "rvf-rpc"))]
#[allow(dead_code)]
impl RvfDaemonClient {
    pub async fn connect() -> Option<Self> {
        use tokio::io::AsyncWriteExt;
        use crate::protocol;

        let path = protocol::socket_path();
        let mut stream = tokio::net::UnixStream::connect(&path).await.ok()?;
        stream.write_all(b"RVFS").await.ok()?;

        let (reader, writer) = stream.into_split();
        Some(Self {
            reader: crate::rvf_codec::RvfFrameReader::new(reader),
            writer: crate::rvf_codec::RvfFrameWriter::new(writer),
            next_segment_id: 1,
        })
    }

    pub async fn call(&mut self, request: Request) -> anyhow::Result<Response> {
        let (seg_type, payload, flags, segment_id) =
            crate::rvf_rpc::encode_request(&request, self.next_segment_id);
        self.next_segment_id += 1;

        self.writer
            .write_frame(seg_type, &payload, flags, segment_id)
            .await?;

        let frame = self
            .reader
            .read_frame()
            .await?
            .ok_or_else(|| anyhow::anyhow!("daemon closed connection without response"))?;

        crate::rvf_rpc::decode_response(&frame)
    }

    pub async fn simple_call(&mut self, method: &str) -> anyhow::Result<Response> {
        self.call(Request::new(method)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_returns_none_when_no_daemon() {
        let client = DaemonClient::connect().await;
        assert!(client.is_none());
    }

    #[tokio::test]
    async fn is_daemon_running_false_when_no_daemon() {
        assert!(!is_daemon_running().await);
    }
}
