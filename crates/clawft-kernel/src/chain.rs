//! Local exochain manager for kernel event logging.
//!
//! Provides an append-only event chain with SHAKE-256 hash linking
//! (via [`rvf_crypto`]). Each event references the hash of the
//! previous event *and* a content hash of its payload, forming
//! an immutable, tamper-evident audit trail suitable for cross-service
//! and cross-node verification.
//!
//! ## Hash scheme
//!
//! Every event carries three hashes:
//! - **`prev_hash`** — SHAKE-256 of the preceding event (chain link)
//! - **`payload_hash`** — SHAKE-256 of the canonical JSON payload bytes
//!   (content commitment; zeroed when payload is `None`)
//! - **`hash`** — SHAKE-256 of `(sequence ‖ chain_id ‖ prev_hash ‖
//!   source ‖ 0x00 ‖ kind ‖ 0x00 ‖ timestamp ‖ payload_hash)`
//!
//! Together these enable *two-way verification*: given an event you
//! can verify the chain link backward *and* the payload content
//! independently.
//!
//! # K0 Scope
//! Local chain only: genesis, append, checkpoint.
//!
//! # K1+ Scope (not implemented)
//! Global root chain, BridgeEvent anchoring, ruvector-raft consensus.

use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rvf_crypto::hash::shake256_256;
use rvf_types::{ExoChainHeader, EXOCHAIN_MAGIC, SEGMENT_HEADER_SIZE};
use rvf_wire::writer::{calculate_padded_size, write_exochain_event};
use rvf_wire::{decode_exochain_payload, read_segment, validate_segment};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// A chain event -- one entry in the append-only log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainEvent {
    /// Sequence number (0 = genesis).
    pub sequence: u64,
    /// Chain ID (0 = local).
    pub chain_id: u32,
    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
    /// SHAKE-256 hash of the previous event (zeroed for genesis).
    pub prev_hash: [u8; 32],
    /// SHAKE-256 hash of this event (covers all fields incl. payload).
    pub hash: [u8; 32],
    /// SHAKE-256 hash of the canonical payload bytes (zeroed when
    /// payload is `None`). Enables independent content verification.
    #[serde(default)]
    pub payload_hash: [u8; 32],
    /// Event source (e.g. "kernel", "service.cron", "cluster").
    pub source: String,
    /// Event kind (e.g. "boot", "service.start", "peer.join").
    pub kind: String,
    /// Optional payload (JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// A checkpoint snapshot of the chain state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainCheckpoint {
    /// Chain ID.
    pub chain_id: u32,
    /// Sequence number at checkpoint.
    pub sequence: u64,
    /// Hash of the last event at checkpoint.
    pub last_hash: [u8; 32],
    /// Timestamp of the checkpoint.
    pub timestamp: DateTime<Utc>,
    /// Number of events since last checkpoint.
    pub events_since_last: u64,
}

/// Result of chain integrity verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerifyResult {
    /// Whether the entire chain is valid.
    pub valid: bool,
    /// Number of events verified.
    pub event_count: usize,
    /// List of errors found (empty if valid).
    pub errors: Vec<String>,
}

/// Compute the SHAKE-256 content hash of a payload.
///
/// Returns the 32-byte SHAKE-256 hash of the canonical JSON bytes,
/// or all zeros if the payload is `None`.
pub(crate) fn compute_payload_hash(payload: &Option<serde_json::Value>) -> [u8; 32] {
    match payload {
        Some(val) => {
            let bytes = serde_json::to_vec(val).unwrap_or_default();
            shake256_256(&bytes)
        }
        None => [0u8; 32],
    }
}

/// Compute the SHAKE-256 hash for a chain event.
///
/// This is the canonical hash function used for both event creation
/// and integrity verification. The hash commits to **all** fields:
///
/// ```text
/// SHAKE-256(
///     sequence(8)  ‖  chain_id(4)  ‖  prev_hash(32)  ‖
///     source  ‖  0x00  ‖  kind  ‖  0x00  ‖
///     timestamp(8)  ‖  payload_hash(32)
/// )
/// ```
///
/// The null-byte separators between `source` and `kind` prevent
/// domain collisions (e.g. "foo" + "bar.baz" vs "foo.bar" + "baz").
pub(crate) fn compute_event_hash(
    sequence: u64,
    chain_id: u32,
    prev_hash: &[u8; 32],
    source: &str,
    kind: &str,
    timestamp: &DateTime<Utc>,
    payload_hash: &[u8; 32],
) -> [u8; 32] {
    let mut buf = Vec::with_capacity(128);
    buf.extend_from_slice(&sequence.to_le_bytes());
    buf.extend_from_slice(&chain_id.to_le_bytes());
    buf.extend_from_slice(prev_hash);
    buf.extend_from_slice(source.as_bytes());
    buf.push(0x00); // separator
    buf.extend_from_slice(kind.as_bytes());
    buf.push(0x00); // separator
    buf.extend_from_slice(&timestamp.timestamp().to_le_bytes());
    buf.extend_from_slice(payload_hash);
    shake256_256(&buf)
}

/// Local chain state.
struct LocalChain {
    chain_id: u32,
    events: Vec<ChainEvent>,
    last_hash: [u8; 32],
    sequence: u64,
    checkpoint_interval: u64,
    events_since_checkpoint: u64,
    checkpoints: Vec<ChainCheckpoint>,
}

impl LocalChain {
    fn new(chain_id: u32, checkpoint_interval: u64) -> Self {
        Self {
            chain_id,
            events: Vec::new(),
            last_hash: [0u8; 32],
            sequence: 0,
            checkpoint_interval,
            events_since_checkpoint: 0,
            checkpoints: Vec::new(),
        }
    }

    /// Restore from a saved set of events.
    fn from_events(
        chain_id: u32,
        checkpoint_interval: u64,
        events: Vec<ChainEvent>,
    ) -> Self {
        let (last_hash, sequence) = if let Some(last) = events.last() {
            (last.hash, last.sequence + 1)
        } else {
            ([0u8; 32], 0)
        };
        Self {
            chain_id,
            events,
            last_hash,
            sequence,
            checkpoint_interval,
            events_since_checkpoint: 0,
            checkpoints: Vec::new(),
        }
    }

    fn append(
        &mut self,
        source: String,
        kind: String,
        payload: Option<serde_json::Value>,
    ) -> &ChainEvent {
        let timestamp = Utc::now();
        let payload_hash = compute_payload_hash(&payload);
        let hash = compute_event_hash(
            self.sequence,
            self.chain_id,
            &self.last_hash,
            &source,
            &kind,
            &timestamp,
            &payload_hash,
        );

        let event = ChainEvent {
            sequence: self.sequence,
            chain_id: self.chain_id,
            timestamp,
            prev_hash: self.last_hash,
            hash,
            payload_hash,
            source,
            kind,
            payload,
        };

        self.last_hash = hash;
        self.sequence += 1;
        self.events_since_checkpoint += 1;
        self.events.push(event);

        // Auto-checkpoint
        if self.checkpoint_interval > 0
            && self.events_since_checkpoint >= self.checkpoint_interval
        {
            self.create_checkpoint();
        }

        self.events.last().unwrap()
    }

    fn create_checkpoint(&mut self) -> ChainCheckpoint {
        let cp = ChainCheckpoint {
            chain_id: self.chain_id,
            sequence: self.sequence.saturating_sub(1),
            last_hash: self.last_hash,
            timestamp: Utc::now(),
            events_since_last: self.events_since_checkpoint,
        };
        self.events_since_checkpoint = 0;
        self.checkpoints.push(cp.clone());
        cp
    }
}

/// CBOR payload structure for RVF segment persistence.
///
/// Contains the per-event fields that are not already covered by the
/// ExoChainHeader (which stores sequence, chain_id, timestamp, prev_hash).
#[derive(Serialize, Deserialize)]
struct RvfChainPayload {
    source: String,
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    payload: Option<serde_json::Value>,
    /// Hex-encoded 32-byte payload hash.
    payload_hash: String,
    /// Hex-encoded 32-byte event hash.
    hash: String,
}

/// Encode a 32-byte hash as a lowercase hex string.
fn hex_hash(h: &[u8; 32]) -> String {
    h.iter().map(|b| format!("{b:02x}")).collect()
}

/// Parse a 64-char hex string back into a 32-byte array.
fn parse_hex_hash(s: &str) -> Result<[u8; 32], Box<dyn std::error::Error + Send + Sync>> {
    if s.len() != 64 {
        return Err(format!("expected 64 hex chars, got {}", s.len()).into());
    }
    let mut out = [0u8; 32];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        let hi = hex_nibble(chunk[0])?;
        let lo = hex_nibble(chunk[1])?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

/// Convert a single ASCII hex character to its nibble value.
fn hex_nibble(c: u8) -> Result<u8, Box<dyn std::error::Error + Send + Sync>> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("invalid hex char: {}", c as char).into()),
    }
}

/// Thread-safe chain manager.
///
/// Wraps a local chain with mutex protection for concurrent access
/// from multiple kernel subsystems.
pub struct ChainManager {
    inner: Mutex<LocalChain>,
}

impl ChainManager {
    /// Create a new chain manager with genesis event.
    pub fn new(chain_id: u32, checkpoint_interval: u64) -> Self {
        let mut chain = LocalChain::new(chain_id, checkpoint_interval);
        // Genesis event
        chain.append(
            "chain".into(),
            "genesis".into(),
            Some(serde_json::json!({ "chain_id": chain_id })),
        );
        debug!(chain_id, "local chain initialized with genesis event");

        Self {
            inner: Mutex::new(chain),
        }
    }

    /// Create with default settings.
    pub fn default_local() -> Self {
        Self::new(0, 1000)
    }

    /// Append an event to the chain.
    pub fn append(
        &self,
        source: &str,
        kind: &str,
        payload: Option<serde_json::Value>,
    ) -> ChainEvent {
        let mut chain = self.inner.lock().unwrap();
        chain.append(source.into(), kind.into(), payload).clone()
    }

    /// Create a checkpoint.
    pub fn checkpoint(&self) -> ChainCheckpoint {
        let mut chain = self.inner.lock().unwrap();
        chain.create_checkpoint()
    }

    /// Get the current chain length.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().events.len()
    }

    /// Check if the chain is empty (should never be after genesis).
    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().events.is_empty()
    }

    /// Get the current sequence number.
    pub fn sequence(&self) -> u64 {
        self.inner.lock().unwrap().sequence
    }

    /// Get the last hash.
    pub fn last_hash(&self) -> [u8; 32] {
        self.inner.lock().unwrap().last_hash
    }

    /// Get the chain ID.
    pub fn chain_id(&self) -> u32 {
        self.inner.lock().unwrap().chain_id
    }

    /// Get recent events (last n, or all if n=0).
    pub fn tail(&self, n: usize) -> Vec<ChainEvent> {
        let chain = self.inner.lock().unwrap();
        if n == 0 || n >= chain.events.len() {
            chain.events.clone()
        } else {
            chain.events[chain.events.len() - n..].to_vec()
        }
    }

    /// Get all checkpoints.
    pub fn checkpoints(&self) -> Vec<ChainCheckpoint> {
        self.inner.lock().unwrap().checkpoints.clone()
    }

    /// Verify the integrity of the entire chain.
    ///
    /// Walks all events and verifies:
    /// 1. Each event's `prev_hash` matches the prior event's `hash`
    /// 2. Each event's `payload_hash` matches the recomputed payload hash
    /// 3. Each event's `hash` matches the recomputed event hash
    pub fn verify_integrity(&self) -> ChainVerifyResult {
        let chain = self.inner.lock().unwrap();
        let mut errors = Vec::new();

        for (i, event) in chain.events.iter().enumerate() {
            // 1. Verify prev_hash linkage
            let expected_prev = if i == 0 {
                [0u8; 32]
            } else {
                chain.events[i - 1].hash
            };
            if event.prev_hash != expected_prev {
                errors.push(format!(
                    "seq {}: prev_hash mismatch (expected {:02x}{:02x}..., got {:02x}{:02x}...)",
                    event.sequence,
                    expected_prev[0], expected_prev[1],
                    event.prev_hash[0], event.prev_hash[1],
                ));
            }

            // 2. Verify payload_hash
            let recomputed_payload = compute_payload_hash(&event.payload);
            if event.payload_hash != recomputed_payload {
                errors.push(format!(
                    "seq {}: payload_hash mismatch (recomputed {:02x}{:02x}..., stored {:02x}{:02x}...)",
                    event.sequence,
                    recomputed_payload[0], recomputed_payload[1],
                    event.payload_hash[0], event.payload_hash[1],
                ));
            }

            // 3. Recompute and verify event hash
            let recomputed = compute_event_hash(
                event.sequence,
                event.chain_id,
                &event.prev_hash,
                &event.source,
                &event.kind,
                &event.timestamp,
                &event.payload_hash,
            );
            if event.hash != recomputed {
                errors.push(format!(
                    "seq {}: hash mismatch (recomputed {:02x}{:02x}..., stored {:02x}{:02x}...)",
                    event.sequence,
                    recomputed[0], recomputed[1],
                    event.hash[0], event.hash[1],
                ));
            }
        }

        ChainVerifyResult {
            valid: errors.is_empty(),
            event_count: chain.events.len(),
            errors,
        }
    }

    /// Save the chain to a file (line-delimited JSON).
    ///
    /// Writes all events as newline-delimited JSON to the given path.
    /// Creates parent directories if they don't exist.
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let chain = self.inner.lock().map_err(|e| format!("lock: {e}"))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut output = String::new();
        for event in &chain.events {
            let line = serde_json::to_string(event)?;
            output.push_str(&line);
            output.push('\n');
        }

        std::fs::write(path, output)?;
        info!(
            path = %path.display(),
            events = chain.events.len(),
            sequence = chain.sequence,
            "chain saved to file"
        );
        Ok(())
    }

    /// Load a chain from a file (line-delimited JSON).
    ///
    /// Reads events, verifies integrity, and restores state so that
    /// new events continue from the last sequence number.
    pub fn load_from_file(
        path: &Path,
        checkpoint_interval: u64,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let contents = std::fs::read_to_string(path)?;
        let mut events = Vec::new();

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let event: ChainEvent = serde_json::from_str(trimmed)?;
            events.push(event);
        }

        if events.is_empty() {
            return Err("chain file is empty (no events)".into());
        }

        let chain_id = events[0].chain_id;
        let chain = LocalChain::from_events(chain_id, checkpoint_interval, events);

        let mgr = Self {
            inner: Mutex::new(chain),
        };

        // Verify integrity of the loaded chain
        let result = mgr.verify_integrity();
        if !result.valid {
            warn!(
                errors = result.errors.len(),
                "loaded chain has integrity errors"
            );
            return Err(format!(
                "chain integrity check failed: {} errors",
                result.errors.len()
            )
            .into());
        }

        info!(
            path = %path.display(),
            events = result.event_count,
            chain_id,
            "chain restored from file"
        );
        Ok(mgr)
    }

    /// Save the chain as a concatenation of RVF segments.
    ///
    /// Each event is serialized as an ExochainEvent segment (subtype 0x40)
    /// containing a 64-byte ExoChainHeader + CBOR payload. A trailing
    /// ExochainCheckpoint segment (subtype 0x41) records the final chain
    /// state for external verification.
    pub fn save_to_rvf(
        &self,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let chain = self.inner.lock().map_err(|e| format!("lock: {e}"))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut output = Vec::new();

        for event in &chain.events {
            // Build the ExoChainHeader from event fields.
            let exo_header = ExoChainHeader {
                magic: EXOCHAIN_MAGIC,
                version: 1,
                subtype: 0x40, // ExochainEvent
                flags: 0,
                chain_id: event.chain_id,
                _reserved: 0,
                sequence: event.sequence,
                timestamp_secs: event.timestamp.timestamp() as u64,
                prev_hash: event.prev_hash,
            };

            // Serialize the remaining fields as CBOR.
            let rvf_payload = RvfChainPayload {
                source: event.source.clone(),
                kind: event.kind.clone(),
                payload: event.payload.clone(),
                payload_hash: hex_hash(&event.payload_hash),
                hash: hex_hash(&event.hash),
            };

            let mut cbor_bytes = Vec::new();
            ciborium::into_writer(&rvf_payload, &mut cbor_bytes)
                .map_err(|e| format!("cbor encode: {e}"))?;

            // Write the full RVF segment (header + exo header + cbor + padding).
            let segment = write_exochain_event(&exo_header, &cbor_bytes, event.sequence);
            output.extend_from_slice(&segment);
        }

        // Write a trailing checkpoint segment (subtype 0x41).
        let checkpoint_header = ExoChainHeader {
            magic: EXOCHAIN_MAGIC,
            version: 1,
            subtype: 0x41, // ExochainCheckpoint
            flags: 0,
            chain_id: chain.chain_id,
            _reserved: 0,
            sequence: chain.sequence.saturating_sub(1),
            timestamp_secs: Utc::now().timestamp() as u64,
            prev_hash: chain.last_hash,
        };

        let cp_payload = serde_json::json!({
            "event_count": chain.events.len(),
            "last_hash": hex_hash(&chain.last_hash),
        });
        let mut cp_cbor = Vec::new();
        ciborium::into_writer(&cp_payload, &mut cp_cbor)
            .map_err(|e| format!("cbor encode checkpoint: {e}"))?;

        let cp_segment = write_exochain_event(
            &checkpoint_header,
            &cp_cbor,
            chain.sequence, // use next sequence as segment_id
        );
        output.extend_from_slice(&cp_segment);

        std::fs::write(path, &output)?;
        info!(
            path = %path.display(),
            events = chain.events.len(),
            bytes = output.len(),
            "chain saved to RVF file"
        );
        Ok(())
    }

    /// Load a chain from an RVF segment file.
    ///
    /// Reads concatenated RVF segments, validates each segment's content
    /// hash, decodes ExoChainHeader + CBOR payload, and reconstructs the
    /// chain events. Checkpoint segments (subtype 0x41) are skipped.
    pub fn load_from_rvf(
        path: &Path,
        checkpoint_interval: u64,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let data = std::fs::read(path)?;
        let mut offset = 0;
        let mut events = Vec::new();

        while offset < data.len() {
            // Need at least a segment header.
            if data.len() - offset < SEGMENT_HEADER_SIZE {
                break;
            }

            let (seg_header, seg_payload) = read_segment(&data[offset..])
                .map_err(|e| format!("read segment at offset {offset}: {e}"))?;

            // Validate the content hash.
            validate_segment(&seg_header, seg_payload)
                .map_err(|e| format!("validate segment at offset {offset}: {e}"))?;

            // Decode the ExoChainHeader + CBOR from the segment payload.
            let (exo_header, cbor_bytes) = decode_exochain_payload(seg_payload)
                .ok_or_else(|| {
                    format!("decode exochain payload at offset {offset}")
                })?;

            if exo_header.subtype == 0x40 {
                // ExochainEvent -- deserialize the CBOR payload.
                let rvf_payload: RvfChainPayload =
                    ciborium::from_reader(cbor_bytes)
                        .map_err(|e| format!("cbor decode at offset {offset}: {e}"))?;

                let payload_hash = parse_hex_hash(&rvf_payload.payload_hash)?;
                let hash = parse_hex_hash(&rvf_payload.hash)?;

                let timestamp = DateTime::from_timestamp(
                    exo_header.timestamp_secs as i64,
                    0,
                )
                .ok_or_else(|| {
                    format!(
                        "invalid timestamp {} at offset {offset}",
                        exo_header.timestamp_secs
                    )
                })?;

                events.push(ChainEvent {
                    sequence: exo_header.sequence,
                    chain_id: exo_header.chain_id,
                    timestamp,
                    prev_hash: exo_header.prev_hash,
                    hash,
                    payload_hash,
                    source: rvf_payload.source,
                    kind: rvf_payload.kind,
                    payload: rvf_payload.payload,
                });
            }
            // subtype 0x41 (Checkpoint) and 0x42 (Proof) are skipped.

            // Advance past the segment: header + payload padded to 64 bytes.
            let padded = calculate_padded_size(
                SEGMENT_HEADER_SIZE,
                seg_header.payload_length as usize,
            );
            offset += padded;
        }

        if events.is_empty() {
            return Err("RVF file contains no chain events".into());
        }

        let chain_id = events[0].chain_id;
        let chain = LocalChain::from_events(chain_id, checkpoint_interval, events);

        let mgr = Self {
            inner: Mutex::new(chain),
        };

        // Verify integrity of the loaded chain.
        let result = mgr.verify_integrity();
        if !result.valid {
            warn!(
                errors = result.errors.len(),
                "loaded RVF chain has integrity errors"
            );
            return Err(format!(
                "RVF chain integrity check failed: {} errors",
                result.errors.len()
            )
            .into());
        }

        info!(
            path = %path.display(),
            events = result.event_count,
            chain_id,
            "chain restored from RVF file"
        );
        Ok(mgr)
    }

    /// Get a status summary.
    pub fn status(&self) -> ChainStatus {
        let chain = self.inner.lock().unwrap();
        ChainStatus {
            chain_id: chain.chain_id,
            sequence: chain.sequence,
            last_hash: chain.last_hash,
            event_count: chain.events.len(),
            checkpoint_count: chain.checkpoints.len(),
            events_since_checkpoint: chain.events_since_checkpoint,
        }
    }
}

/// Chain status summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStatus {
    pub chain_id: u32,
    pub sequence: u64,
    pub last_hash: [u8; 32],
    pub event_count: usize,
    pub checkpoint_count: usize,
    pub events_since_checkpoint: u64,
}

impl std::fmt::Debug for ChainManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = self.status();
        f.debug_struct("ChainManager")
            .field("chain_id", &status.chain_id)
            .field("sequence", &status.sequence)
            .field("event_count", &status.event_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_event() {
        let cm = ChainManager::new(0, 1000);
        assert_eq!(cm.len(), 1);
        assert_eq!(cm.sequence(), 1); // genesis consumed seq 0
        let events = cm.tail(0);
        assert_eq!(events[0].kind, "genesis");
        assert_eq!(events[0].sequence, 0);
        assert_eq!(events[0].prev_hash, [0u8; 32]);
    }

    #[test]
    fn append_links_hashes() {
        let cm = ChainManager::new(0, 1000);
        let genesis_hash = cm.last_hash();

        let e1 = cm.append("test", "event.one", None);
        assert_eq!(e1.prev_hash, genesis_hash);
        assert_ne!(e1.hash, [0u8; 32]);

        let e2 = cm.append("test", "event.two", Some(serde_json::json!({"key": "value"})));
        assert_eq!(e2.prev_hash, e1.hash);
    }

    #[test]
    fn checkpoint() {
        let cm = ChainManager::new(0, 1000);
        cm.append("test", "event", None);

        let cp = cm.checkpoint();
        assert_eq!(cp.chain_id, 0);
        assert_eq!(cp.sequence, 1);
        assert_eq!(cm.checkpoints().len(), 1);
    }

    #[test]
    fn auto_checkpoint() {
        let cm = ChainManager::new(0, 5); // checkpoint every 5 events
        // Genesis is event 0 (1 event since checkpoint)
        for i in 0..4 {
            cm.append("test", &format!("event.{i}"), None);
        }
        // 5 total events (genesis + 4) -> should auto-checkpoint
        assert_eq!(cm.checkpoints().len(), 1);
    }

    #[test]
    fn status() {
        let cm = ChainManager::new(0, 1000);
        cm.append("test", "event", None);
        let status = cm.status();
        assert_eq!(status.chain_id, 0);
        assert_eq!(status.sequence, 2);
        assert_eq!(status.event_count, 2);
    }

    #[test]
    fn verify_integrity_valid() {
        let cm = ChainManager::new(0, 1000);
        cm.append("kernel", "boot.init", None);
        cm.append("tree", "bootstrap", Some(serde_json::json!({"nodes": 8})));
        cm.append("kernel", "boot.ready", None);

        let result = cm.verify_integrity();
        assert!(result.valid);
        assert_eq!(result.event_count, 4); // genesis + 3
        assert!(result.errors.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let cm = ChainManager::new(0, 1000);
        cm.append("kernel", "boot.init", None);
        cm.append("tree", "bootstrap", Some(serde_json::json!({"nodes": 8})));
        cm.append("kernel", "boot.ready", None);

        let original_seq = cm.sequence();
        let original_hash = cm.last_hash();
        let original_len = cm.len();

        let dir = std::env::temp_dir().join("clawft-chain-test");
        let path = dir.join("test-chain.json");
        cm.save_to_file(&path).unwrap();

        let restored = ChainManager::load_from_file(&path, 1000).unwrap();
        assert_eq!(restored.sequence(), original_seq);
        assert_eq!(restored.last_hash(), original_hash);
        assert_eq!(restored.len(), original_len);
        assert_eq!(restored.chain_id(), 0);

        // Verify restored chain integrity
        let result = restored.verify_integrity();
        assert!(result.valid);

        // New events continue from restored state
        let new_event = restored.append("test", "after.restore", None);
        assert_eq!(new_event.sequence, original_seq);
        assert_eq!(new_event.prev_hash, original_hash);

        // Clean up
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_from_nonexistent_file_fails() {
        let result = ChainManager::load_from_file(
            &std::path::PathBuf::from("/tmp/nonexistent-chain-file.json"),
            1000,
        );
        assert!(result.is_err());
    }

    #[test]
    fn tail() {
        let cm = ChainManager::new(0, 1000);
        cm.append("a", "1", None);
        cm.append("b", "2", None);
        cm.append("c", "3", None);

        let last2 = cm.tail(2);
        assert_eq!(last2.len(), 2);
        assert_eq!(last2[0].kind, "2");
        assert_eq!(last2[1].kind, "3");

        let all = cm.tail(0);
        assert_eq!(all.len(), 4); // genesis + 3
    }

    #[test]
    fn save_and_load_rvf_roundtrip() {
        let cm = ChainManager::new(0, 1000);
        cm.append("kernel", "boot.init", None);
        cm.append(
            "tree",
            "bootstrap",
            Some(serde_json::json!({"nodes": 8})),
        );
        cm.append("kernel", "boot.ready", None);

        let original_seq = cm.sequence();
        let original_hash = cm.last_hash();
        let original_len = cm.len();

        let dir = std::env::temp_dir().join("clawft-chain-rvf-test");
        let path = dir.join("test-chain.rvf");
        cm.save_to_rvf(&path).unwrap();

        let restored = ChainManager::load_from_rvf(&path, 1000).unwrap();
        assert_eq!(restored.sequence(), original_seq);
        assert_eq!(restored.last_hash(), original_hash);
        assert_eq!(restored.len(), original_len);
        assert_eq!(restored.chain_id(), 0);

        // Verify restored chain integrity.
        let result = restored.verify_integrity();
        assert!(result.valid, "integrity errors: {:?}", result.errors);
        assert_eq!(result.event_count, original_len);

        // New events continue from restored state.
        let new_event = restored.append("test", "after.rvf.restore", None);
        assert_eq!(new_event.sequence, original_seq);
        assert_eq!(new_event.prev_hash, original_hash);

        // Clean up.
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rvf_validates_on_load() {
        let cm = ChainManager::new(0, 1000);
        cm.append("kernel", "boot", None);

        let dir = std::env::temp_dir().join("clawft-chain-rvf-validate");
        let path = dir.join("corrupt.rvf");
        cm.save_to_rvf(&path).unwrap();

        // Corrupt a byte in the first segment's payload area.
        let mut data = std::fs::read(&path).unwrap();
        // The payload starts at SEGMENT_HEADER_SIZE (64). Flip a byte
        // inside the ExoChainHeader portion of the payload.
        if data.len() > SEGMENT_HEADER_SIZE + 10 {
            data[SEGMENT_HEADER_SIZE + 10] ^= 0xFF;
        }
        std::fs::write(&path, &data).unwrap();

        let result = ChainManager::load_from_rvf(&path, 1000);
        assert!(result.is_err(), "expected validation error on corrupted RVF");

        // Clean up.
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rvf_migration_from_json() {
        // Create a chain via the normal API.
        let cm = ChainManager::new(0, 1000);
        cm.append("kernel", "boot.init", None);
        cm.append(
            "tree",
            "bootstrap",
            Some(serde_json::json!({"nodes": 4, "name": "test"})),
        );
        cm.append("kernel", "boot.ready", None);

        let dir = std::env::temp_dir().join("clawft-chain-migrate-test");
        let json_path = dir.join("chain.json");
        let rvf_path = dir.join("chain.rvf");

        // Save as JSON, load as JSON.
        cm.save_to_file(&json_path).unwrap();
        let from_json = ChainManager::load_from_file(&json_path, 1000).unwrap();

        // Save the JSON-loaded chain as RVF.
        from_json.save_to_rvf(&rvf_path).unwrap();

        // Load from RVF and compare.
        let from_rvf = ChainManager::load_from_rvf(&rvf_path, 1000).unwrap();

        assert_eq!(from_rvf.sequence(), cm.sequence());
        assert_eq!(from_rvf.last_hash(), cm.last_hash());
        assert_eq!(from_rvf.len(), cm.len());
        assert_eq!(from_rvf.chain_id(), cm.chain_id());

        // Compare event-by-event.
        let original_events = cm.tail(0);
        let rvf_events = from_rvf.tail(0);
        assert_eq!(original_events.len(), rvf_events.len());
        for (orig, loaded) in original_events.iter().zip(rvf_events.iter()) {
            assert_eq!(orig.sequence, loaded.sequence);
            assert_eq!(orig.chain_id, loaded.chain_id);
            assert_eq!(orig.hash, loaded.hash);
            assert_eq!(orig.prev_hash, loaded.prev_hash);
            assert_eq!(orig.payload_hash, loaded.payload_hash);
            assert_eq!(orig.source, loaded.source);
            assert_eq!(orig.kind, loaded.kind);
            assert_eq!(orig.payload, loaded.payload);
        }

        // Verify integrity of the RVF-loaded chain.
        let result = from_rvf.verify_integrity();
        assert!(result.valid, "integrity errors: {:?}", result.errors);

        // Clean up.
        let _ = std::fs::remove_dir_all(&dir);
    }
}
