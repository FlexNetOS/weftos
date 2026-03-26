# clawft + WeftOS

[![CI](https://github.com/weave-logic-ai/clawft/actions/workflows/ci.yml/badge.svg)](https://github.com/weave-logic-ai/clawft/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/clawft.svg)](LICENSE)

**clawft** is a modular, async Rust framework for building AI assistants.
**WeftOS** is the kernel layer that turns clawft into a distributed operating
system for AI agents — with process management, capability-based security,
mesh networking, and a cognitive substrate.

Together, they run on 5 platforms: native Linux/macOS/Windows binaries, WASI
containers, browser tabs (via WASM), edge devices, and cloud VMs — forming
encrypted peer-to-peer clusters across all of them.

## What This Is

```
+------------------------------------------------------------------+
|                        clawft                                     |
|  AI assistant framework: LLM providers, channels, tools, skills  |
+------------------------------------------------------------------+
|                        WeftOS                                     |
|  Kernel: processes, IPC, governance, ExoChain, mesh networking   |
+------------------------------------------------------------------+
|                     ECC Cognitive Substrate                       |
|  Causal DAG, cognitive tick, HNSW search, impulse queue          |
+------------------------------------------------------------------+
|                     Platform Layer                                |
|  Native (tokio) | WASI | Browser (wasm32) | Edge | Cloud        |
+------------------------------------------------------------------+
```

**clawft** handles the AI layer: connecting to LLM providers (OpenAI, Anthropic,
Ollama, local models), managing conversations across channels (Telegram, Slack,
Discord, CLI, WebSocket), executing tools in sandboxes, and composing agent
skills.

**WeftOS** handles the OS layer: booting a kernel with a process table, enforcing
per-agent RBAC capabilities, routing messages between agents (locally and across
nodes), maintaining an append-only hash chain (ExoChain) for audit trails,
synchronizing resource trees across clusters, and governing all operations
through a three-branch constitutional model.

**ECC** (Ephemeral Causal Cognition) is the cognitive substrate: a causal DAG
that tracks how ideas relate, a cognitive tick that adaptively processes events,
HNSW approximate nearest neighbor search for semantic memory, cross-references
linking structures across the kernel, and an impulse queue for ephemeral
real-time signals.

## Platform Support

| Platform | Binary | Transport | Features |
|----------|--------|-----------|----------|
| **Linux** (x86_64, aarch64) | `weft` / `weave` | TCP, QUIC, WebSocket | Full kernel + mesh + WASM sandbox |
| **macOS** (x86_64, aarch64) | `weft` / `weave` | TCP, QUIC, WebSocket | Full kernel + mesh + WASM sandbox |
| **Windows** (x86_64) | `weft.exe` / `weave.exe` | TCP, WebSocket | Full kernel + mesh |
| **Browser** (wasm32) | `clawft-wasm` | WebSocket | Agent framework, restricted IPC |
| **WASI** (wasm32-wasi) | `clawft-wasi` | WebSocket | Agent framework, sandboxed tools |
| **Edge** (ARM64) | `weft` | TCP, mDNS | Kernel + local mesh discovery |
| **Cloud VM** | `weft` / `weave` | TCP, QUIC, WebSocket | Full stack + Kademlia DHT |
| **Docker** | `alpine:weft` | Configurable | Sidecar orchestration |

All platforms can join the same mesh cluster. A browser tab running `clawft-wasm`
connects via WebSocket to a cloud VM running full WeftOS, which peers with edge
devices over mDNS — all sharing the same governance rules, chain state, and
service registry.

## Key Features

### clawft (AI Framework)

- **Multi-provider LLM** — OpenAI, Anthropic, Ollama, vLLM, llama.cpp, and any
  OpenAI-compatible API behind a single trait
- **Multi-channel messaging** — Telegram, Slack, Discord, CLI, WebSocket, HTTP
  with a unified PluginHost architecture
- **6-stage pipeline** — Classifier, Router, Assembler, Transport, Scorer,
  Learner for structured message processing
- **Tool system** — 27 built-in tools (filesystem, shell, memory, web, agent
  management) with WASM sandboxing
- **MCP integration** — Dual-mode Model Context Protocol: expose tools as server
  or connect to external MCP servers as client
- **Skills and agents** — Declarative SKILL.md format, multi-agent spawning,
  inter-agent IPC
- **Plugin system** — Git, Cargo, OAuth2, TreeSitter, browser, calendar,
  containers — all hot-reloadable

### WeftOS (Kernel)

- **Process management** — PID allocation, state machine (Running/Suspended/
  Stopping), resource tracking, agent supervisor with spawn/stop/restart
- **IPC** — Typed `KernelMessage` envelopes with 7 target types (Process, Topic,
  Broadcast, Service, ServiceMethod, RemoteNode, Kernel) and 6 payload variants
- **Capability-based security** — Per-agent RBAC with IpcScope (All/ParentOnly/
  Restricted/Topic/None), ToolPermissions, SandboxPolicy, ResourceLimits
- **ExoChain** — Append-only hash chain with SHAKE-256 linking, Ed25519 + ML-DSA-65
  dual signing, RVF segment persistence, witness chains, and lineage records
- **Three-branch governance** — Legislative/Executive/Judicial branches evaluate
  every privileged operation against 5-dimensional effect vectors (risk, fairness,
  privacy, novelty, security). Environment-scoped: dev is lenient, prod is strict.
- **WASM sandbox** — Wasmtime-based tool execution with fuel metering (CPU
  budget), memory limits (allocation bomb prevention), and complete host
  filesystem isolation
- **Container integration** — Docker/Podman sidecar orchestration with health
  check propagation to the kernel health system
- **Application framework** — `weftapp.toml` manifests, install/start/stop
  lifecycle, agent spawning with capability wiring from manifests

### Mesh Networking (K6)

- **Transport-agnostic** — TCP and WebSocket today, QUIC (quinn) ready.
  `MeshTransport` trait makes any byte stream a mesh link.
- **Noise Protocol encryption** — XX pattern for first contact, IK for known
  peers. All inter-node traffic encrypted. Ed25519 static keys.
- **Post-quantum protection** — Hybrid ML-KEM-768 + X25519 key exchange.
  Graceful degradation when one side lacks KEM support. Protects against
  store-now-decrypt-later quantum attacks.
- **Peer discovery** — Static seed peers, peer exchange during handshake, mDNS
  for LAN discovery (UDP multicast), Kademlia DHT for WAN discovery (XOR
  distance, k-buckets, governance-namespaced keys).
- **Cross-node IPC** — `MeshIpcEnvelope` transports `KernelMessage` across nodes
  transparently. Hop counting prevents routing loops. Bloom filter deduplication
  prevents double delivery.
- **Distributed process table** — CRDT-based last-writer-wins process
  advertisements. ConsistentHashRing for PID-to-node assignment.
- **Service discovery** — `ClusterServiceRegistry` with round-robin and
  affinity-based resolution. TTL-cached with negative cache. Circuit breaker
  prevents cascade failures.
- **Chain replication** — Incremental `tail_from()` sync, fork detection,
  bridge events anchoring remote chain heads, backpressure with checkpoint
  catch-up when >1000 events behind.
- **Tree synchronization** — Merkle root comparison, incremental diff transfer
  of only changed subtrees. Signed mutations verified against node Ed25519 keys.
- **SWIM heartbeat** — Alive/Suspect/Dead state machine with configurable
  timeouts. Direct ping + indirect probe via random witnesses.
- **Metadata consensus** — Raft-style log for service registry and process table.
  CRDT gossip for eventually-consistent state convergence.

### ECC (Ephemeral Causal Cognition)

The cognitive substrate that gives WeftOS agents the ability to reason about
causality, search semantic memory, and process ephemeral signals in real-time.

- **Causal DAG** — Typed, weighted edges tracking how ideas, events, and
  decisions relate. BFS traversal and path finding. Every edge links to the
  chain event that created it.
- **Cognitive tick** — An adaptive processing interval that fires regularly,
  processing accumulated causal edges, updating the HNSW index, and detecting
  drift. Auto-calibrated at boot time based on hardware capability.
- **HNSW search** — Approximate nearest neighbor search over high-dimensional
  embeddings. Thread-safe wrapper around `instant-distance`. Enables semantic
  queries like "find the 10 most similar past decisions."
- **Cross-references** — BLAKE3-based `UniversalNodeId` links structures across
  the kernel: chain events to tree nodes to causal edges to HNSW entries.
  Bidirectional traversal.
- **Impulse queue** — HLC-sorted ephemeral events with short TTL. For real-time
  cognitive coordination: "this agent just completed a task" signals that decay
  rather than persist.
- **Boot-time calibration** — Benchmarks compute time per tick, measures p50/p95
  latency, auto-adjusts tick interval for the hardware. Prevents cognitive
  overload on constrained devices.
- **Three operating modes** — The same engine runs in three modes:
  - **Act**: Real-time conversation processing within the cognitive tick
  - **Analyze**: Post-hoc understanding of existing corpora (transcripts, PRs,
    research papers)
  - **Generate**: Goal-directed content generation using causal planning
- **Cluster advertisement** — `NodeEccCapability` advertises each node's
  cognitive capacity (tick interval, compute headroom, HNSW vector count, causal
  edge count, spectral analysis support) so the mesh can route ECC queries to
  capable nodes.

## Quick Start

### Install

```sh
cargo install clawft-cli
```

### Configure

```sh
export OPENAI_API_KEY="sk-..."
```

### Run an agent

```sh
weft agent
weft agent -m "Summarize this project"
```

### Run WeftOS kernel

```sh
weave boot                # Start the kernel
weave ps                  # List running processes
weave service list        # List registered services
weave chain status        # ExoChain status
weave app install myapp   # Install an application
weave app list            # List installed apps
weave cluster peers       # Show mesh peers
weave ecc status          # ECC cognitive substrate status
```

## Architecture

clawft is organized as a Cargo workspace with 22 crates:

```
clawft-cli / clawft-weave     CLI binaries (weft / weave)
  |
clawft-kernel                  WeftOS kernel (K0-K6)
  |  |- boot, process, supervisor, capability
  |  |- ipc, a2a, topic, cron, health, service
  |  |- chain, tree_manager, gate, governance
  |  |- wasm_runner, container, app, agency
  |  |- mesh_*, mesh_tcp, mesh_ws (17 networking modules)
  |  |- causal, cognitive_tick, hnsw_service (ECC)
  |
clawft-core                    Agent engine, MessageBus, pipeline
clawft-llm                     LLM provider abstraction
clawft-tools                   Tool implementations
clawft-channels                Channel plugins
clawft-services                Background services
clawft-plugin-*                Plugin crates (git, cargo, oauth2, ...)
clawft-security                Security policies
clawft-wasm                    Browser WASM target
exo-resource-tree              Merkle resource tree with mutation log
```

### Feature Flags

| Feature | Crate | Description |
|---------|-------|-------------|
| `native` | `clawft-kernel` | Tokio runtime, native file I/O (default) |
| `exochain` | `clawft-kernel` | ExoChain hash chain + resource tree + gate backends |
| `cluster` | `clawft-kernel` | Multi-node clustering via ruvector-cluster/raft |
| `mesh` | `clawft-kernel` | Mesh networking (TCP, WebSocket, discovery, IPC) |
| `ecc` | `clawft-kernel` | ECC cognitive substrate (causal DAG, HNSW, impulse) |
| `wasm-sandbox` | `clawft-kernel` | Wasmtime WASM tool execution |
| `containers` | `clawft-kernel` | Docker/Podman container integration |
| `vector-memory` | `clawft-core` | IntelligentRouter, VectorStore, SessionIndexer |

Build configurations:

```sh
# Minimal (single-node, no networking)
cargo build --release

# Full kernel with mesh networking
cargo build --release --features "native,exochain,cluster,mesh,ecc"

# Everything
cargo build --release --features "native,exochain,cluster,mesh,ecc,wasm-sandbox,containers"

# Browser target
cargo build --release --target wasm32-unknown-unknown -p clawft-wasm
```

## Testing

843 tests across all kernel features. Zero regressions across feature combinations.

```sh
# All tests (full features)
cargo test -p clawft-kernel --features "native,exochain,cluster,mesh,wasm-sandbox"

# Phase gate verification (K3-K6)
scripts/k6-gate.sh

# Build check
scripts/build.sh check

# Lint
scripts/build.sh clippy
```

## Documentation

The documentation site is built with [Fumadocs](https://fumadocs.vercel.app/)
and lives in `docs/src/`:

```sh
cd docs/src
npm install
npm run dev     # http://localhost:3000
```

32 pages across two sections:
- **clawft** — Getting started, architecture, CLI reference, configuration,
  plugins, providers, channels, tools, skills, browser mode, deployment, security
- **WeftOS** — Architecture, kernel phases (K0-K6), boot sequence, process table,
  IPC, capabilities, ExoChain, governance, WASM sandbox, containers, app framework,
  mesh networking, discovery, clustering, ECC, security, decisions, kernel guide

## Governance Model

WeftOS uses a three-branch constitutional governance model inspired by
separation of powers:

| Branch | Role | Example Rules |
|--------|------|---------------|
| **Legislative** | Define what agents can do | "Agents may not access /etc" |
| **Executive** | Approve runtime operations | "Auto-approve low-risk tool calls" |
| **Judicial** | Review and audit after the fact | "Flag operations with risk > 0.8" |

Every privileged operation is evaluated against a 5-dimensional **EffectVector**:

| Dimension | Measures |
|-----------|----------|
| Risk | Potential for harm or data loss |
| Fairness | Equitable resource distribution |
| Privacy | Data exposure and access scope |
| Novelty | Deviation from established patterns |
| Security | Attack surface and vulnerability |

Environments apply different thresholds:
- **Development**: Lenient (threshold 0.9) — rapid iteration
- **Staging**: Normal (threshold 0.6) — pre-production validation
- **Production**: Strict (threshold 0.3) — maximum safety

## Security

- **Capability-based access control** — Every agent has explicit capabilities
  (IPC scope, tool permissions, spawn rights, resource limits)
- **Dual-layer governance gate** — Routing-time check before message delivery +
  handler-time check before tool execution
- **Noise Protocol encryption** — All inter-node traffic encrypted with forward
  secrecy and mutual authentication
- **Post-quantum cryptography** — Hybrid ML-KEM-768 + X25519 key exchange,
  Ed25519 + ML-DSA-65 dual chain signing
- **WASM sandbox isolation** — Fuel metering, memory limits, no host filesystem
  access by default
- **Message size limits** — 16 MiB maximum at all deserialization boundaries
- **Rate limiting** — Configurable peer addition rate limiting
- **Browser sandboxing** — Browser agents start with `IpcScope::Restricted`,
  capability elevation requires governance gate approval
- **Tamper-evident audit trail** — ExoChain with SHAKE-256 hash linking,
  integrity verification, witness chains

## Building from Source

### Prerequisites

- Rust 1.93+ (edition 2024)
- Node.js 18+ (for documentation site)
- Docker (optional, for container features)

### Build

```sh
git clone https://github.com/weave-logic-ai/clawft.git
cd clawft
scripts/build.sh native          # Release binary
scripts/build.sh native-debug    # Debug binary (fast iteration)
scripts/build.sh test            # Run tests
scripts/build.sh gate            # Full phase gate (11 checks)
scripts/build.sh all             # Everything (native + wasi + browser + ui)
```

## Contributing

1. Fork the repository and create a feature branch
2. Write tests for new functionality
3. Run `scripts/build.sh gate` before submitting
4. Follow the [kernel developer guide](docs/src/content/docs/weftos/kernel-guide.mdx)
5. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

## Attribution

clawft + WeftOS builds on ideas and patterns from:

- [claude-code](https://github.com/anthropics/claude-code) — Anthropic's agentic coding tool
- [claude-flow](https://github.com/ruvnet/claude-flow) — Multi-agent orchestration framework
- [ruvector](https://github.com/ruvnet/ruvector) — Rust vector operations and distributed consensus
- The WeftOS kernel architecture draws from microkernel OS design (L4, seL4),
  capability-based security (Capsicum), and distributed systems research
  (SWIM, Raft, CRDTs, Kademlia)
