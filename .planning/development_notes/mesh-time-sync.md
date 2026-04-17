# Mesh Time Synchronization

**Date**: 2026-04-17
**Status**: Design
**Priority**: High (required for distributed ECC, temporal topology)

## Problem

Distributed WeftOS nodes need consistent timestamps for:
- ECC causal graph ordering (which event happened first?)
- Topology browser temporal dimension (timeline layout)
- ExoChain event sequencing (chain_seq assumes monotonic time)
- Sensor fusion across multiple machines (robotics use case)

Without time sync, events on different nodes can't be accurately
ordered, and the causal graph becomes unreliable.

## Architecture

```
         ┌─────────────────────┐
         │   Time Authority    │
         │  (NTP-synced node)  │
         │  source: NTP/GPS    │
         │  precision: ~1ms    │
         └─────────┬───────────┘
                   │ mesh heartbeat
                   │ carries authority_time
          ┌────────┼────────┐
          ▼        ▼        ▼
       ┌──────┐ ┌──────┐ ┌──────┐
       │Node A│ │Node B│ │Node C│
       │offset│ │offset│ │offset│
       │=+3ms │ │=-1ms │ │=+7ms │
       └──────┘ └──────┘ └──────┘
```

### Clock hierarchy

1. **Authority election**: node with the best clock source wins
   - Priority: GPS > NTP > local monotonic
   - Tie-break: lowest node_id
   - Re-election on authority failure (heartbeat timeout)

2. **Sync protocol**: piggyback on mesh heartbeats
   - Heartbeat already has `timestamp: u64` field
   - Add `authority_time: u64` (microseconds since epoch)
   - Add `authority_id: String` (which node is authority)
   - Receiving node computes: offset = authority_time - local_time

3. **Offset smoothing**: exponential moving average
   - Don't jump on every heartbeat — smooth the offset
   - `smoothed_offset = 0.9 * smoothed_offset + 0.1 * new_offset`
   - Reject outliers (>100ms jump = network issue, not clock drift)

4. **Clock API**:
   ```rust
   impl MeshRuntime {
       /// Get the current mesh-synchronized time (microseconds).
       fn mesh_time(&self) -> u64 {
           let local = monotonic_micros();
           (local as i64 + self.clock_offset) as u64
       }

       /// Get the clock quality estimate (microseconds of uncertainty).
       fn clock_uncertainty(&self) -> u64 { ... }
   }
   ```

## WiFi TSF Integration (ESP32/embedded)

On ESP32, the WiFi TSF counter provides microsecond-precision
synchronization between all stations on the same AP:

```rust
unsafe {
    let mut tsf: u64 = 0;
    esp_idf_sys::esp_wifi_get_tsf_time(
        esp_idf_sys::wifi_interface_t_WIFI_IF_STA,
        &mut tsf,
    );
    // tsf is in microseconds, synchronized across all WiFi stations
}
```

For WeftOS nodes on the same WiFi network, TSF gives sub-microsecond
sync for free. The mesh time sync protocol is for nodes on different
networks where TSF isn't available.

**Hybrid approach**: use TSF when available (same WiFi), fall back to
mesh heartbeat sync when not (cross-network, wired, etc).

## Precision Targets

| Scenario | Method | Expected Precision |
|----------|--------|-------------------|
| Same WiFi AP | TSF counter | < 1 µs |
| Same LAN, mesh TCP | Heartbeat sync | ~100 µs |
| Cross-network, mesh TCP | Heartbeat sync + NTP authority | ~1-5 ms |
| WAN, mesh TCP | NTP authority + offset smoothing | ~10-50 ms |

## Implementation Plan

### Phase 1: Authority timestamp in heartbeat
- Add `authority_time` and `authority_id` to heartbeat message
- Authority node includes its NTP-synced time
- Non-authority nodes compute and store offset

### Phase 2: Authority election
- Nodes advertise their clock source quality in heartbeat
- Highest-quality source becomes authority
- Automatic re-election on authority failure

### Phase 3: mesh_time() API
- MeshRuntime exposes `mesh_time()` returning synced microseconds
- ECC causal graph uses mesh_time() for edge timestamps
- ExoChain uses mesh_time() for event sequencing

### Phase 4: TSF integration (embedded)
- On ESP32/WiFi platforms, prefer TSF when nodes share an AP
- Fall back to mesh sync for cross-AP communication
- Report clock source in heartbeat (TSF/NTP/mesh/local)

## Impact on Existing Systems

- **ECC**: `CausalEdge.timestamp` becomes mesh-synced
- **ExoChain**: `chain_seq` ordering reliable across nodes
- **Topology browser**: timeline mode works for distributed systems
- **Robotics**: sensor fusion with consistent timestamps
- **Assessment**: declared-vs-observed timing for latency analysis
