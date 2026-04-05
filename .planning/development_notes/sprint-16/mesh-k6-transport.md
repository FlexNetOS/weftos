# Mesh K6 Transport: Real-time Cross-Project Assessment Coordination

**Date**: 2026-04-04
**SOP**: 3 (Cross-Project Coordination)
**Sprint**: 16
**Status**: Implemented

## Summary

Implemented real-time mesh coordination for cross-project assessment exchange,
replacing the artifact-based (disk file) comparison model from Sprint 15 with
a wire protocol that pushes assessment summaries over the K6 mesh network.

## What Changed

### New Files

- `crates/clawft-kernel/src/mesh_assess.rs` -- The assessment mesh transport
  layer. Contains `AssessmentEnvelope` (wire format) and `AssessmentTransport`
  (the bridge between `MeshCoordinator` and the mesh networking stack).

### Modified Files

- `crates/clawft-kernel/src/mesh_framing.rs` -- Added `FrameType::AssessmentSync`
  (0x0E) discriminant for assessment-specific mesh frames.
- `crates/clawft-kernel/src/lib.rs` -- Registered `mesh_assess` module and
  added re-exports (`AssessmentEnvelope`, `AssessmentTransport`).

## Architecture

```
AssessmentService
  |
  v
MeshCoordinator (assessment/mesh.rs) -- existing, unchanged
  |
  v
AssessmentTransport (mesh_assess.rs) -- NEW: serializes/deserializes
  |                                      assessment messages to/from
  |                                      MeshFrame payloads
  v
MeshFrame (mesh_framing.rs) -- FrameType::AssessmentSync (0x0E)
  |
  v
TcpTransport / WsTransport (mesh_tcp.rs, mesh_ws.rs) -- wire
```

### Wire Format

Assessment messages are JSON-serialized `AssessmentEnvelope` structs inside
`FrameType::AssessmentSync` frames. The envelope adds routing metadata:

```json
{
  "source_node": "node-a",
  "sequence": 42,
  "message": { /* AssessmentMessage variant */ }
}
```

The `sequence` field is a monotonically increasing counter for deduplication.

### Protocol Messages (unchanged from Sprint 15)

| Message | Direction | Purpose |
|---------|-----------|---------|
| `Gossip` | Periodic broadcast | Lightweight status exchange |
| `ReportAvailable` | Event-driven broadcast | Notify peers of new assessment |
| `RequestReport` | Point-to-point | Ask a peer for its full report |
| `FullReport` | Point-to-point response | Send the full assessment report |

### Key APIs

- `AssessmentTransport::build_gossip_frame()` -- Serialize gossip for all peers
- `AssessmentTransport::build_broadcast_frame()` -- Serialize ReportAvailable
- `AssessmentTransport::handle_incoming()` -- Decode and dispatch to coordinator
- `AssessmentTransport::gossip_tick()` -- Periodic gossip driver
- `AssessmentTransport::drain_pending()` -- Consume pending broadcast from coordinator

## Feature Gate

All code is behind `#[cfg(feature = "mesh")]` -- same gate as the rest of
the K6 networking stack. No new dependencies added. WASM builds are unaffected.

## Tests

17 new tests covering:

- Envelope serde roundtrip
- Frame type correctness and rejection of wrong types
- Gossip, broadcast, request, and full-report frame roundtrips
- Two-node assessment exchange (full protocol simulation)
- TCP integration test (real TCP transport with assessment frames)
- Sequence number monotonicity
- Pending broadcast drain lifecycle
- Gossip tick with/without reports

## Integration Points

The `AssessmentTransport` is designed to be held by the mesh runtime or
daemon. The typical integration pattern:

1. Daemon creates `AssessmentTransport::new(Arc::clone(&coordinator))`
2. On each heartbeat tick, calls `gossip_tick()` and broadcasts bytes to peers
3. After each `assess run`, calls `drain_pending()` and sends to peers
4. On incoming `FrameType::AssessmentSync` frames, calls `handle_incoming()`

## Next Steps

- Wire `AssessmentTransport` into the daemon's mesh event loop
- Add `weft assess mesh-status` CLI subcommand showing real-time peer states
- Implement assessment diff propagation (push only changed findings)
- Add QUIC transport support (ADR-026) alongside existing TCP
