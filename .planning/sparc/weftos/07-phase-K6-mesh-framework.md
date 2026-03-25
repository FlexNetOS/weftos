# Phase K6: Transport-Agnostic Encrypted Mesh Network

**Phase ID**: K6
**Workstream**: W-KERNEL
**Duration**: Weeks 11-16 (6 sub-phases)
**Goal**: Extend WeftOS from a single-node kernel into a multi-node cluster with encrypted peer-to-peer networking, distributed IPC, chain replication, tree synchronization, and cluster-wide service discovery
**Gate from**: K2 C10, K5 Symposium (2026-03-25)
**Symposium Decisions**: D1-D10, Commitments C1-C5

---

## S -- Specification

### What Changes

This phase adds a complete mesh networking stack to WeftOS. The mesh is
transport-agnostic: Cloud and Edge nodes use QUIC (quinn), Browser and WASI
nodes use WebSocket, and the protocol treats the transport as a pluggable
implementation detail. All inter-node traffic is encrypted using the Noise
Protocol (snow). Peer discovery uses Kademlia DHT and mDNS. The existing
single-node kernel continues to work unchanged when the `mesh` feature gate
is disabled.

The architecture follows the 5-layer model approved by the K5 Symposium
(Panel 1, Decision D1):

```
APPLICATION    WeftOS IPC (A2ARouter), Chain Sync, Tree Sync
DISCOVERY      Kademlia DHT, mDNS, Bootstrap Peers
ENCRYPTION     Noise Protocol (snow) -- XX/IK handshakes, Ed25519 keys
TRANSPORT      quinn (QUIC) | tokio-tungstenite (WS) | webrtc-rs
IDENTITY       Ed25519 keypair = node identity, governance.genesis = trust root
```

### Files to Create

| File | Phase | Purpose |
|------|-------|---------|
| `crates/clawft-kernel/src/mesh.rs` | K6.1 | `MeshTransport` trait, `MeshStream`, `TransportListener` |
| `crates/clawft-kernel/src/mesh_quic.rs` | K6.1 | QUIC transport implementation via quinn |
| `crates/clawft-kernel/src/mesh_noise.rs` | K6.1 | Noise Protocol wrapper via snow (XX + IK handshakes) |
| `crates/clawft-kernel/src/mesh_framing.rs` | K6.1 | Length-prefix framing + message type dispatch |
| `crates/clawft-kernel/src/mesh_listener.rs` | K6.1 | Accept loop, handshake orchestration, peer registration |
| `crates/clawft-kernel/src/mesh_discovery.rs` | K6.2 | Discovery trait + coordinator |
| `crates/clawft-kernel/src/mesh_kad.rs` | K6.2 | Kademlia DHT wrapper (libp2p-kad) |
| `crates/clawft-kernel/src/mesh_mdns.rs` | K6.2 | mDNS local discovery (libp2p-mdns) |
| `crates/clawft-kernel/src/mesh_bootstrap.rs` | K6.2 | Static seed peer bootstrap |
| `crates/clawft-kernel/src/mesh_ipc.rs` | K6.3 | Serialize/deserialize KernelMessage over mesh streams |
| `crates/clawft-kernel/src/mesh_service.rs` | K6.3 | Cross-node service registry query protocol |
| `crates/clawft-kernel/src/mesh_dedup.rs` | K6.3 | Message deduplication (bloom filter on message IDs) |
| `crates/clawft-kernel/src/mesh_chain.rs` | K6.4 | Chain replication: delta sync, bridge events, subscription forwarding |
| `crates/clawft-kernel/src/mesh_tree.rs` | K6.4 | Tree sync: snapshot transfer, Merkle proof exchange |
| `crates/clawft-kernel/src/mesh_process.rs` | K6.5 | Distributed process table (CRDT gossip) |
| `crates/clawft-kernel/src/mesh_service_adv.rs` | K6.5 | Service advertisement and cluster-wide resolution |
| `crates/clawft-kernel/src/mesh_heartbeat.rs` | K6.5 | SWIM-style heartbeat + failure detection |

### Files to Modify

| File | Change | Phase |
|------|--------|-------|
| `crates/clawft-kernel/Cargo.toml` | Add quinn, snow, x25519-dalek (optional); feature gates mesh/mesh-discovery/mesh-full | K6.0 |
| `crates/clawft-kernel/src/ipc.rs` | Add `MessageTarget::RemoteNode` variant; add `GlobalPid` struct | K6.0 |
| `crates/clawft-kernel/src/cluster.rs` | Add `bind_address`, `seed_peers`, `identity_key_path` to `ClusterConfig`; add `NodeIdentity` | K6.0 |
| `crates/clawft-kernel/src/chain.rs` | Add `tail_from(seq)` for incremental replication; add chain event subscription | K6.0, K6.4 |
| `crates/clawft-kernel/src/tree_manager.rs` | Add `snapshot()`, `apply_remote_mutation()` with signature verification; sign `MutationEvent.signature` | K6.0, K6.4 |
| `crates/clawft-kernel/src/lib.rs` | Re-export mesh modules behind `#[cfg(feature = "mesh")]` | K6.1 |
| `crates/clawft-kernel/src/a2a.rs` | Add cluster-aware service resolution; remote inbox delivery bridge | K6.3 |
| `crates/clawft-kernel/src/boot.rs` | Add mesh listener startup + peer discovery to boot sequence | K6.1 |
| `crates/clawft-kernel/src/service.rs` | Add cross-node service advertisement | K6.5 |
| `Cargo.toml` (workspace) | Add quinn, snow, x25519-dalek, optionally libp2p-kad/libp2p-mdns | K6.0 |

### Key Types

**MeshTransport trait** (`mesh.rs`) -- Symposium C3:
```rust
#[async_trait]
pub trait MeshTransport: Send + Sync + 'static {
    fn name(&self) -> &str;
    async fn listen(&self, addr: &str) -> Result<TransportListener>;
    async fn connect(&self, addr: &str) -> Result<MeshStream>;
    fn supports(&self, addr: &str) -> bool;
}
```

**NoiseChannel** (`mesh_noise.rs`) -- Symposium D3:
```rust
pub struct NoiseChannel {
    session: snow::TransportState,
    stream: Box<dyn MeshStream>,
}
```

**PeerId / NodeIdentity** (`cluster.rs`) -- Symposium D2:
```rust
pub struct NodeIdentity {
    keypair: ed25519_dalek::SigningKey,
    node_id: String,  // hex(SHAKE-256(pubkey)[0..16])
}
```

**GlobalPid** (`ipc.rs`) -- Symposium C2:
```rust
pub struct GlobalPid {
    pub node_id: String,
    pub pid: Pid,
}
```

**MeshConfig** (`cluster.rs`):
```rust
pub struct MeshConfig {
    pub bind_address: String,
    pub seed_peers: Vec<String>,
    pub identity_key_path: Option<PathBuf>,
    pub max_message_size: usize,  // default 16 MiB (D8)
}
```

**MessageTarget::RemoteNode** (`ipc.rs`) -- Symposium C1:
```rust
pub enum MessageTarget {
    // ... existing variants ...
    RemoteNode {
        node_id: String,
        target: Box<MessageTarget>,
    },
}
```

### New Rust Dependencies

| Crate | Version | Feature Gate | Purpose | Symposium Ref |
|-------|---------|-------------|---------|---------------|
| `quinn` | 0.11 | `mesh` | QUIC transport with multiplexing | D1, D6 |
| `snow` | 0.9 | `mesh` | Noise Protocol Framework | D1, D3 |
| `x25519-dalek` | 2.0 | `mesh` | X25519 Diffie-Hellman for Noise | D1 |
| `libp2p-kad` | 0.46 | `mesh-discovery` | Kademlia DHT peer discovery | D1 |
| `libp2p-mdns` | 0.46 | `mesh-discovery` | mDNS LAN peer discovery | D1 |

### Feature Gate Structure -- Symposium D5, C4

```toml
[features]
mesh = ["quinn", "snow", "x25519-dalek"]
mesh-discovery = ["mesh", "libp2p-kad", "libp2p-mdns"]
mesh-full = ["mesh", "mesh-discovery"]
```

| Build Config | Features | Use Case |
|-------------|----------|----------|
| Single-node | (none) | Development, testing, embedded, WASI |
| Static cluster | `mesh` | Known peers, seed-peer bootstrap |
| Dynamic cluster | `mesh-full` | DHT + mDNS auto-discovery |

---

## P -- Pseudocode

### Connection Establishment (K6.1) -- Symposium D1, D3, D6

```
fn dial(addr: &str, identity: &NodeIdentity) -> Result<EncryptedPeer>:
    // 1. Select transport based on address scheme
    transport = match addr:
        "quic://*" => QuicTransport
        "ws://*"   => WebSocketTransport
        "tcp://*"  => TcpTransport

    // 2. Establish raw byte stream
    stream = transport.connect(addr).await?

    // 3. Noise XX handshake (first contact) or IK (known peer) [D3]
    noise_builder = snow::Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2b")
    initiator = noise_builder
        .local_private_key(identity.x25519_secret())
        .build_initiator()?

    // XX pattern: -> e, s  |  <- e, ee, se, s, es  |  -> payload
    buf = [0u8; 65535]
    len = initiator.write_message(&[], &mut buf)?
    stream.send(&buf[..len]).await?

    resp = stream.recv().await?
    initiator.read_message(&resp, &mut buf)?

    // Final message: send our capabilities + chain_head
    payload = serialize(WeftHandshake {
        node_id: identity.node_id(),
        capabilities: local_capabilities(),
        chain_head: chain.head_hash(),
        genesis_hash: governance.genesis_hash(),  // D4: cluster trust root
    })
    len = initiator.write_message(&payload, &mut buf)?
    stream.send(&buf[..len]).await?

    transport_state = initiator.into_transport_mode()?

    // 4. Wrap in NoiseChannel for encrypted I/O
    channel = NoiseChannel::new(transport_state, stream)

    // 5. Verify genesis_hash matches (D4: reject foreign clusters)
    remote_handshake = channel.recv_handshake().await?
    if remote_handshake.genesis_hash != governance.genesis_hash():
        return Err(MeshError::GenesisMismatch)

    // 6. Register peer in ClusterMembership
    cluster.add_peer(PeerNode::from(remote_handshake))?

    Ok(EncryptedPeer { channel, remote: remote_handshake })
```

### Peer Discovery (K6.2) -- Symposium D1

```
fn discover_peers(config: &MeshConfig, cluster: &ClusterMembership):
    // Phase 1: Static seed peers (always available, no extra deps)
    for seed in config.seed_peers:
        spawn(async {
            peer = dial(seed, identity).await?
            exchange_peer_lists(peer).await?
        })

    // Phase 2: mDNS for LAN (if mesh-discovery enabled)
    #[cfg(feature = "mesh-discovery")]
    mdns_task = spawn(async {
        mdns = MdnsDiscovery::new(WEFTOS_SERVICE_NAME)
        loop:
            peer_info = mdns.next().await
            if !cluster.has_peer(peer_info.node_id):
                peer = dial(peer_info.addr, identity).await?
                cluster.add_peer(peer)?
    })

    // Phase 3: Kademlia DHT (if mesh-discovery enabled)
    #[cfg(feature = "mesh-discovery")]
    kad_task = spawn(async {
        kad = KademliaDht::new(identity.node_id_bytes())
        for peer in cluster.active_peers():
            kad.add_address(peer.id, peer.address)
        loop:
            sleep(30s)
            closest = kad.find_node(identity.node_id_bytes()).await
            for found in closest:
                if !cluster.has_peer(found.id):
                    peer = dial(found.addr, identity).await?
                    cluster.add_peer(peer)?
    })
```

### Cross-Node Message Routing (K6.3) -- Symposium C1, C2

```
fn send_message(msg: KernelMessage, target: MessageTarget):
    match target:
        MessageTarget::RemoteNode { node_id, inner_target }:
            // 1. Look up mesh connection for node_id
            peer = mesh_connections.get(node_id)
                .ok_or(MeshError::PeerNotConnected)?

            // 2. Apply governance gate on outbound message
            gate_decision = governance.evaluate(
                GovernanceRequest::new("ipc.cross_node")
                    .with_agent(msg.from)
                    .with_target(node_id)
            )?
            if gate_decision != Permit: return Err(Denied)

            // 3. Frame message with length prefix [D8: 16 MiB max]
            frame = Frame {
                len: serialized_size(&msg),
                msg_type: 0x02,  // KernelMessage (RVF segment)
                payload: serialize_rvf(&msg),  // D8: rvf-wire format
            }
            if frame.len > MAX_MESSAGE_SIZE: return Err(MessageTooLarge)

            // 4. Send over encrypted mesh stream
            peer.channel.send(frame).await?

            // 5. Log to chain
            chain.append("mesh", "ipc.forward", json!({
                "target_node": node_id,
                "msg_id": msg.id,
            }))

        // All other targets: existing local dispatch
        _ => existing_local_send(msg, target)
```

### Chain Sync Protocol (K6.4) -- Symposium D9

```
fn sync_chain_with_peer(peer: &EncryptedPeer):
    // 1. Compare chain heads
    local_head = chain.head_sequence()
    remote_head = peer.handshake.chain_head_seq

    if local_head == remote_head: return  // already in sync

    if local_head < remote_head:
        // Pull missing events from peer
        request = ChainSyncRequest {
            chain_id: 0,  // local chain
            after_sequence: local_head,
        }
        response = peer.channel.request(request).await?

        for event in response.events:
            // Verify dual signature (D9: Ed25519 + ML-DSA-65)
            verify_ed25519(event.signature, event.hash)?
            verify_ml_dsa(event.pq_signature, event.hash)?
            chain.append_verified(event)?

    else:
        // Push our newer events to peer
        events = chain.tail_from(remote_head)
        peer.channel.send(ChainSyncResponse { events }).await?

    // 2. Create bridge event anchoring remote chain head
    chain.append("mesh", "chain.bridge", json!({
        "remote_node": peer.node_id,
        "remote_head_hash": peer.handshake.chain_head_hash,
        "remote_head_seq": remote_head,
    }))
```

### Cluster Join with Governance Verification (K6.0/K6.1) -- Symposium C5, D4

```
fn handle_join_request(request: JoinRequest, peer: &EncryptedPeer):
    // 1. Verify genesis hash matches cluster [D4]
    if request.genesis_hash != governance.genesis_hash():
        peer.send(JoinResponse::Rejected("genesis mismatch")).await?
        return

    // 2. Verify Ed25519 signature on join request [D2]
    verify_ed25519(request.pubkey, request.signature, request.payload)?

    // 3. Evaluate via GovernanceGate [C5]
    decision = governance.evaluate(
        GovernanceRequest::new("cluster.join")
            .with_agent(request.node_id)
            .with_capabilities(request.capabilities)
            .with_platform(request.platform)
    )?

    match decision:
        Permit:
            cluster.add_peer(peer_from_request(request))?
            peer.send(JoinResponse::Accepted {
                peer_list: cluster.active_peers(),
                governance_rules: governance.rules(),
                chain_head: chain.head(),
            }).await?
            // Start chain + tree sync
            spawn(sync_chain_with_peer(peer))
            spawn(sync_tree_with_peer(peer))

        Deny(reason):
            peer.send(JoinResponse::Rejected(reason)).await?
```

---

## A -- Architecture

### 5-Layer Diagram -- Symposium Panel 1

```
+------------------------------------------------------------------+
|                    APPLICATION LAYER                               |
|  A2ARouter (cross-node IPC), ChainSync, TreeSync, ServiceDiscovery|
|  Integration: ipc.rs, a2a.rs, chain.rs, tree_manager.rs           |
+------------------------------------------------------------------+
|                    DISCOVERY LAYER                                 |
|  Kademlia DHT (libp2p-kad), mDNS (libp2p-mdns), Bootstrap Peers  |
|  Files: mesh_discovery.rs, mesh_kad.rs, mesh_mdns.rs              |
|  Feature gate: mesh-discovery                                     |
+------------------------------------------------------------------+
|                    ENCRYPTION LAYER                                |
|  Noise Protocol (snow) -- XX for first contact, IK for known      |
|  Ed25519 static keys, X25519 ephemeral DH                         |
|  File: mesh_noise.rs                                              |
+------------------------------------------------------------------+
|                    TRANSPORT LAYER                                 |
|  quinn 0.11 (QUIC) | tokio-tungstenite (WS) | raw TCP            |
|  Files: mesh.rs (trait), mesh_quic.rs                             |
|  Feature gate: mesh                                               |
+------------------------------------------------------------------+
|                    IDENTITY LAYER                                  |
|  Ed25519 keypair = node identity [D2]                             |
|  governance.genesis = cluster trust root [D4]                     |
|  NodeIdentity in cluster.rs                                       |
+------------------------------------------------------------------+
```

### Component Relationships

```
                    ┌──────────────────────────────┐
                    │         boot.rs               │
                    │   (start mesh listener)       │
                    └──────┬───────────────────────┘
                           │
              ┌────────────┼────────────────┐
              │            │                │
    ┌─────────▼───┐  ┌─────▼──────┐  ┌─────▼─────────────┐
    │ mesh_       │  │ mesh_      │  │ mesh_discovery.rs  │
    │ listener.rs │  │ quic.rs    │  │ mesh_kad.rs        │
    │ (accept)    │  │ (transport)│  │ mesh_mdns.rs       │
    └──────┬──────┘  └─────┬──────┘  └──────┬─────────────┘
           │               │                │
           └───────┬───────┘                │
                   │                        │
           ┌───────▼───────┐       ┌────────▼──────────┐
           │ mesh_noise.rs │       │ cluster.rs         │
           │ (encryption)  │       │ (ClusterMembership)│
           └───────┬───────┘       └────────┬──────────┘
                   │                        │
           ┌───────▼───────┐                │
           │ mesh_         │                │
           │ framing.rs    │                │
           │ (wire format) │                │
           └───────┬───────┘                │
                   │                        │
    ┌──────────────┼──────────────┬─────────┘
    │              │              │
┌───▼─────┐  ┌────▼─────┐  ┌────▼──────────┐
│ mesh_   │  │ mesh_    │  │ mesh_         │
│ ipc.rs  │  │ chain.rs │  │ process.rs    │
│         │  │ (sync)   │  │ (distributed) │
└───┬─────┘  └────┬─────┘  └────┬──────────┘
    │              │              │
┌───▼─────┐  ┌────▼─────┐  ┌────▼──────────┐
│ a2a.rs  │  │ chain.rs │  │ service.rs    │
│ ipc.rs  │  │ tree_    │  │ (cross-node   │
│ (exist.)│  │ manager  │  │  adverts)     │
└─────────┘  └──────────┘  └───────────────┘
```

### Integration with Existing Kernel Modules

| Existing Module | Integration Point | Symposium Ref |
|----------------|-------------------|---------------|
| `cluster.rs` | `ClusterMembership` receives peer updates from mesh discovery | D4 |
| `ipc.rs` | `KernelIpc::send()` forks to mesh transport for `RemoteNode` targets | C1 |
| `a2a.rs` | `A2ARouter` gains cluster-aware service resolution | C1 |
| `chain.rs` | `LocalChain` gains `tail_from(seq)` for incremental replication | D9 |
| `tree_manager.rs` | `TreeManager` gains `snapshot()` / `apply_remote_mutation()` | -- |
| `governance.rs` | `GovernanceEngine` distributes rules via mesh; gates remote ops | C5 |
| `boot.rs` | Boot sequence adds mesh listener + peer discovery when feature enabled | D5 |
| `service.rs` | `ServiceRegistry` gains cross-node service advertisement | D10 |

### Ruvector Reuse -- Symposium D7

Ruvector algorithms are pure computation, producing messages to send and
consuming messages received. The mesh layer provides the I/O bridge:

| Ruvector Crate | Algorithm | Mesh Integration |
|---------------|-----------|-----------------|
| ruvector-cluster | SWIM membership | Drive with mesh heartbeats |
| ruvector-raft | Raft consensus | Use mesh transport for AppendEntries/RequestVote |
| ruvector-replication | Log replication | Replicate chain events over mesh streams |
| ruvector-delta-consensus | CRDT gossip | Gossip CRDT deltas over mesh pub/sub (K6.5) |
| rvf-wire | Zero-copy segments | Wire format for mesh messages (D8) |

### Platform-Transport Matrix -- Symposium D6

| Platform | Primary Transport | Fallback | Discovery |
|----------|------------------|----------|-----------|
| CloudNative | QUIC (quinn) | TCP | Kademlia DHT + bootstrap |
| Edge | QUIC (quinn) | TCP, BLE | Kademlia DHT + mDNS |
| Browser | WebSocket | WebRTC | Bootstrap peers via WS |
| Wasi | WebSocket | -- | Bootstrap peers via WS |
| Embedded | BLE, LoRa | TCP | mDNS, static config |

---

## R -- Refinement

### Edge Cases

**NAT Traversal**:
- QUIC (quinn) handles connection migration and NAT rebinding natively.
- For nodes behind symmetric NATs, designate relay nodes with the `Relay`
  capability (see doc 12 `NodeCapability::Relay`). Browser nodes always
  connect outbound via WebSocket to a relay.
- Future: WebRTC ICE for browser-to-browser direct connections.

**Split Brain / Network Partition**:
- Open question Q5 from symposium. For K6, the approach is:
  - Chain replication uses eventual consistency (not consensus).
  - Each partition continues to extend its local chain independently.
  - On reconnection, chains are reconciled via bridge events anchoring
    each side's head hash. Events are ordered by HLC (Hybrid Logical Clock).
  - Governance rules require judicial branch quorum for irreversible
    operations; partitioned nodes that lack quorum defer such decisions.
- Full consensus (ruvector-raft) for shared metadata is deferred to K6+.

**Network Partition Recovery**:
- When two partitions rejoin, the discovery layer detects new peers.
- Chain sync identifies divergence point (common ancestor by sequence + hash).
- Events from both sides are merged into a DAG structure (bridge events
  record the divergence). No events are lost.
- Tree sync uses Merkle root comparison: differing roots trigger a diff
  exchange of only the changed subtrees.

### Security Boundaries -- Symposium D3, D9

- All inter-node traffic encrypted with Noise Protocol (D3). No plaintext.
  This supersedes doc 12's "encryption is opt-in" approach.
- Dual signing (Ed25519 + ML-DSA-65) required for cross-node chain events (D9).
- GovernanceGate evaluates all remote operations identically to local ones (C5).
- Maximum message size: 16 MiB (prevents memory exhaustion attacks).
- Message deduplication via bloom filter on message IDs.
- Remote capability claims verified against source node's signed advertisement.
- Rate limiting on cluster join requests and governance evaluation requests.

### Browser Node Support

- Browser nodes connect via WebSocket to a cloud/edge relay node.
- The relay terminates WebSocket and bridges to the QUIC mesh.
- Browser nodes participate in the same mesh protocol, same governance,
  same chain verification -- but with limited capabilities:
  - Cannot listen for incoming connections (no server sockets in browsers).
  - Storage is IndexedDB/OPFS (limited, ephemeral).
  - Identity persisted via browser storage (Q3 from symposium).
- Browser transport implementation deferred to K6.3+ but the `MeshTransport`
  trait design accommodates it from K6.1.

### Backward Compatibility

- The `mesh` feature gate (D5) means all mesh code compiles to zero when
  disabled. The default build is unchanged.
- The `RemoteNode` variant in `MessageTarget` (C1) returns
  `Err(IpcError::RemoteNotAvailable)` until K6.1 wires the transport.
  Existing code never constructs this variant.
- All existing single-node tests pass without modification.
- `GlobalPid` (C2) is used only at mesh boundaries. Local code continues
  to use bare `Pid`.

### Doc 12 Deviations

The symposium refined several decisions from doc 12:

| Doc 12 Position | Symposium Decision | Status |
|----------------|-------------------|--------|
| TCP is default transport, encryption opt-in | Noise encryption mandatory for ALL inter-node traffic (D3) | **Superseded** |
| libp2p multiaddr addressing | Direct address strings (quic://host:port, ws://host:port) | **Simplified** |
| DeFi-style bonds and trust levels | Deferred post-K6; governance.genesis as trust root (D4) | **Deferred** |
| 5-step pairing handshake (HELLO/CHALLENGE/PROVE/ACCEPT/BOND) | Noise XX handshake + WeftOS handshake (2 phases) | **Simplified** |
| Post-quantum ML-KEM-768 for bonded channels | Post-quantum via dual signing (D9), not per-channel PQ | **Narrowed** |

---

## C -- Completion

### Exit Criteria

- [ ] Two CloudNative nodes connect via QUIC with Noise XX encryption
- [ ] A Browser node connects via WebSocket with Noise encryption
- [ ] Nodes discover each other via seed peers (static bootstrap)
- [ ] Nodes discover each other via mDNS on LAN (when mesh-discovery enabled)
- [ ] Nodes discover each other via Kademlia DHT (when mesh-discovery enabled)
- [ ] `KernelMessage` routes transparently between nodes via `RemoteNode` target
- [ ] Remote messages pass through GovernanceGate before delivery
- [ ] Chain events replicate incrementally between nodes (`tail_from`)
- [ ] Cross-node chain events carry dual signatures (Ed25519 + ML-DSA-65)
- [ ] Bridge events anchor remote chain head hashes
- [ ] Resource tree state synchronizes between nodes (Merkle root comparison)
- [ ] Remote tree mutations verified against node's Ed25519 signature
- [ ] Services on any node are discoverable from any other node
- [ ] Process advertisements gossip via CRDT-based distributed table
- [ ] Stopped nodes detected as Unreachable via SWIM-style heartbeats
- [ ] All existing single-node tests pass unchanged
- [ ] `mesh` feature gate compiles to zero networking code when disabled
- [ ] Maximum message size (16 MiB) enforced at deserialization
- [ ] Message deduplication prevents double-delivery

### Testing Verification Commands

```bash
# Build with mesh feature
scripts/build.sh native --features mesh

# Build with full mesh + discovery
scripts/build.sh native --features mesh-full

# Run mesh-specific tests
scripts/build.sh test -- --features mesh mesh_

# Verify single-node build unchanged (no mesh deps)
scripts/build.sh check

# Full phase gate
scripts/build.sh gate
```

### 6-Phase Breakdown -- Symposium D10

#### K6.0: Prep Changes (~200 lines, 0 new deps)

Modify existing K0-K5 code to accept K6 extensions. All prep changes
maintain backward compatibility. No new crate dependencies.

| Item | File | Lines | Symposium Ref |
|------|------|:-----:|---------------|
| Add `RemoteNode` to `MessageTarget` | `ipc.rs` | ~10 | C1 |
| Add `GlobalPid` struct | `ipc.rs` | ~20 | C2 |
| Add mesh fields to `ClusterConfig` | `cluster.rs` | ~10 | -- |
| Add `NodeIdentity` struct | `cluster.rs` | ~40 | D2 |
| Add `tail_from()` to `LocalChain` | `chain.rs` | ~10 | -- |
| Add `mesh` feature gate definition | `Cargo.toml` | ~5 | C4, D5 |
| Sign `MutationEvent.signature` with node key | `tree_manager.rs` | ~15 | -- |

**Test**: Existing tests pass. New variant serde roundtrips. GlobalPid equality.

#### K6.1: Transport + Noise Encryption (~420 lines, 3 new deps)

Build the core mesh transport with encrypted connections.

| File | Purpose | Lines |
|------|---------|:-----:|
| `mesh.rs` | MeshTransport trait, MeshStream, TransportListener | ~60 |
| `mesh_quic.rs` | QUIC transport via quinn | ~120 |
| `mesh_noise.rs` | Noise wrapper via snow (XX + IK) | ~100 |
| `mesh_framing.rs` | Length-prefix framing + message type dispatch | ~60 |
| `mesh_listener.rs` | Accept loop, handshake, peer registration | ~80 |

**Test**: Noise handshake roundtrip, QUIC connect+send+recv, framing encode/decode,
max message size enforcement, invalid handshake rejection.

#### K6.2: Discovery (~330 lines, 2 optional deps)

Peer discovery via Kademlia DHT and mDNS.

| File | Purpose | Lines |
|------|---------|:-----:|
| `mesh_discovery.rs` | Discovery trait + coordinator | ~80 |
| `mesh_kad.rs` | Kademlia DHT wrapper | ~120 |
| `mesh_mdns.rs` | mDNS local discovery | ~80 |
| `mesh_bootstrap.rs` | Static seed peer bootstrap | ~50 |

**Test**: Bootstrap from seeds, mDNS announcement+discovery, Kademlia put/get,
peer list exchange, discovery -> ClusterMembership update.

#### K6.3: Cross-Node IPC (~380 lines)

Route KernelMessage across nodes transparently.

| File | Purpose | Lines |
|------|---------|:-----:|
| `mesh_ipc.rs` | KernelMessage serialize/deserialize over mesh | ~80 |
| `mesh_service.rs` | Cross-node service registry query | ~70 |
| `mesh_dedup.rs` | Message deduplication (bloom filter) | ~80 |
| Changes to `ipc.rs` | Transport fork for RemoteNode | ~40 |
| Changes to `a2a.rs` | Cluster-aware service resolution + remote inbox bridge | ~110 |

**Test**: Remote message roundtrip, cross-node service resolution, governance gate
on remote messages, dedup rejection, GlobalPid in responses.

#### K6.4: Chain Replication + Tree Sync (~300 lines)

Synchronize chain events and resource tree state across nodes.

| File | Purpose | Lines |
|------|---------|:-----:|
| `mesh_chain.rs` | ChainSyncRequest/Response, BridgeEvent, push subscription | ~120 |
| `mesh_tree.rs` | TreeSyncRequest/Response, Merkle proof, remote mutation | ~80 |
| Changes to `chain.rs` | `tail_from()`, `subscribe()` | ~50 |
| Changes to `tree_manager.rs` | `snapshot()`, `apply_remote_mutation()` | ~50 |

**Test**: tail_from returns correct slices, chain sync over mesh, bridge events,
tree snapshot roundtrip, remote mutation with valid/invalid signature, root hash
comparison short-circuit.

#### K6.5: Distributed Process Table + Service Discovery (~240 lines)

Cluster-wide process and service visibility.

| File | Purpose | Lines |
|------|---------|:-----:|
| `mesh_process.rs` | ProcessAdvertisement CRDT gossip | ~100 |
| `mesh_service_adv.rs` | ServiceAdvertisement + resolution | ~80 |
| `mesh_heartbeat.rs` | SWIM-style heartbeat + failure detection | ~60 |

**Test**: Process advertisement gossip, cross-node service discovery, failure
detection, CRDT merge convergence, service resolution fallback (local-first).

### Line Count Summary

| Phase | New Lines | Changed Lines | New Deps |
|-------|:---------:|:------------:|----------|
| K6.0 | ~50 | ~150 | None |
| K6.1 | ~400 | ~20 | quinn, snow, x25519-dalek |
| K6.2 | ~300 | ~30 | libp2p-kad, libp2p-mdns (optional) |
| K6.3 | ~300 | ~80 | None |
| K6.4 | ~250 | ~50 | None |
| K6.5 | ~200 | ~40 | None |
| **Total** | **~1,500** | **~370** | **5 (2 optional)** |

---

## Open Questions (Inherited from Symposium)

| # | Question | Impact | Resolve By |
|---|----------|--------|-----------|
| Q1 | Chain merge: leader-based consensus or DAG? | K6.4 architecture | Before K6.4 |
| Q2 | Wire format: JSON or RVF for KernelMessage? | Performance vs debuggability | K6.1 design |
| Q3 | Browser identity persistence across sessions | UX + security | K6.1 browser transport |
| Q4 | Full libp2p-kad or lighter custom DHT? | Dep weight | K6.2 design |
| Q5 | Split-brain handling on network partition | Consistency vs availability | Before K6.4 |
| Q6 | BLAKE3 (ECC D6) or stay with SHAKE-256? | Hash migration | K6.0 design |
| Q7 | Maximum practical cluster size? | Config defaults, test scenarios | K6.2 testing |
| Q8 | Tree sync: full snapshot or Merkle proof exchange? | Bandwidth vs complexity | K6.4 design |

---

## Cross-References

| Document | Relationship |
|----------|-------------|
| `01-phase-K0-kernel-foundation.md` | Process table, service registry (base for mesh extensions) |
| `03-phase-K2-a2a-ipc.md` | IPC architecture extended with RemoteNode routing |
| `07-ruvector-deep-integration.md` | Ruvector algorithms composed with mesh I/O layer (D7) |
| `08-ephemeral-os-architecture.md` | Multi-node fabric vision; mesh makes it real |
| `10-agent-first-single-user.md` | Agent lifecycle unchanged; services work identically over mesh |
| `12-networking-and-pairing.md` | Original networking vision; refined/superseded by symposium |
| `14-exochain-substrate.md` | Chain manager extended with replication + bridge events |
| `docs/weftos/k5-symposium/01-mesh-architecture.md` | Authoritative 5-layer architecture |
| `docs/weftos/k5-symposium/04-k6-implementation-plan.md` | Authoritative phase plan |
| `docs/weftos/k5-symposium/05-symposium-results.md` | Decisions D1-D10, Commitments C1-C5 |
| `docs/weftos/sparc/k6-cluster-networking.md` | Earlier K6 sketch (superseded by this plan) |
| `.planning/development_notes/k6-readiness-audit.md` | Readiness matrix (41 GREEN, 22 YELLOW, 21 RED) |
