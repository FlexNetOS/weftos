# Pasqal Cloud Integration with WeftOS Quantum Cognitive Layer

## Research Date: 2026-04-04

---

## 1. Pasqal Cloud Platform Overview

Pasqal builds quantum computers using **neutral atoms** trapped by optical tweezers
(highly focused lasers) in programmable 2D/3D geometries. This is uniquely suited
to WeftOS because the graph topology of a `CausalGraph` can be directly mapped to
atom positions -- the hardware natively encodes graph structure.

### Hardware

| Device | Target ID | Qubits | Status |
|--------|-----------|--------|--------|
| Fresnel | `pasqal.qpu.fresnel` | 100 | Production |
| Orion Gamma | TBD | 140+ | Late 2025 |
| Next-gen | TBD | 250 | Early 2026 (quantum advantage target) |
| Roadmap | TBD | 1,000 | End 2025 lab demo |
| Long-term | TBD | 10,000 | 2028 |

**Atom type**: Rubidium (Rb)
**Qubit encoding**: Ground state |g> vs Rydberg state |r> (high-energy excited state)
**Interaction**: Van der Waals C6/R^6 between Rydberg states (distance-dependent)
**Blockade radius**: Rb = (C6 / hbar*Omega)^(1/6) -- atoms closer than Rb cannot both
be in Rydberg state simultaneously

### Emulators

| Emulator | Target ID | Qubits | Backend |
|----------|-----------|--------|---------|
| EMU-TN | `pasqal.sim.emu-tn` | 100 (1D/2D) | GPU cluster (NVIDIA A100 DGX) |
| EMU-MPS | N/A | 60-100 | GPU-accelerated MPS |
| EMU-SV | N/A | ~25 | State vector (exact) |
| EMU-FREE | N/A | varies | Free tier emulator |

### Availability

- Pasqal Cloud direct (cloud.pasqal.com)
- Microsoft Azure Quantum (`pasqal.qpu.fresnel`, `pasqal.sim.emu-tn`)
- Google Cloud Marketplace

---

## 2. API and SDK Details

### 2.1 Authentication

**Auth provider**: Auth0
**Token endpoint**: `POST https://pasqal.eu.auth0.com/oauth/token`

**User authentication**:
```json
{
  "grant_type": "password",
  "username": "{{email}}",
  "password": "{{password}}",
  "client_id": "eiSaMfiINjiaXr0tnc2Bh1Mr6XPQ1BDK",
  "audience": "https://apis.pasqal.cloud/account/api/v1",
  "scope": "openid profile email"
}
```

**Service account** (machine-to-machine):
```json
{
  "grant_type": "client_credentials",
  "client_id": "{{service_account_client_id}}",
  "client_secret": "{{service_account_client_secret}}",
  "audience": "https://apis.pasqal.cloud/account/api/v1"
}
```

**Token response**:
```json
{
  "access_token": "eyJhbGciOi...",
  "scope": "openid profile email",
  "expires_in": 86400,
  "token_type": "Bearer"
}
```

**Usage**: `Authorization: Bearer {{access_token}}` on all subsequent requests.
**Expiry**: 24 hours.

### 2.2 REST API Endpoints

Base URL: `https://apis.pasqal.cloud` (inferred from audience)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/batches` | GET | List batches (paginated, filterable) |
| `/api/v1/batches` | POST | Create a batch with jobs |
| `/api/v2/batches/{batch_id}` | GET | Get batch details (v2 returns job IDs only) |
| `/api/v1/batches/{batch_id}/cancel` | PATCH | Cancel a batch |
| `/api/v1/batches/{batch_id}/results` | GET | Get batch results |
| `/api/v2/jobs` | GET | List jobs |
| `/api/v2/jobs/{job_id}` | GET | Get job details |
| `/api/v2/jobs/{job_id}/cancel` | PATCH | Cancel a job |
| `/api/v1/devices` | GET | List available devices |
| `/api/v1/devices/specs/{device_type}` | GET | Get device specifications |
| `/api/v1/workloads` | POST | Submit workload |
| `/api/v1/auth/delegated-token` | POST | Get delegated token |
| `/api/v1/auth/info` | GET | Auth info |

**Data format**: `application/json`
**Input format**: `pasqal.pulser.v1`
**Output format**: `pasqal.pulser-results.v1`

### 2.3 Python SDK (pasqal-cloud)

```python
from pasqal_cloud import SDK, DeviceTypeName

sdk = SDK(username="user@email.com", project_id="proj-id")

batch = sdk.create_batch(
    serialized_sequence,                              # JSON from seq.to_abstract_repr()
    [{"runs": 100, "variables": {"omega_max": 9.5}}], # job parameters
    device_type=DeviceTypeName.FRESNEL,                # or FRESNEL_SA1
)

# Open batch for iterative jobs
batch = sdk.create_batch(..., open=True)
batch.add_jobs([...], wait=True)
batch.close()

# Get results
results = sdk.get_batch(batch.id)
```

**Batch constraints**: Max 1,000 jobs per batch. Open batches time out after several
minutes without new jobs.

**Job lifecycle**: PENDING -> RUNNING -> DONE | ERROR | CANCELED

### 2.4 Pulser SDK (Sequence Construction)

```python
import pulser

# 1. Define register (atom positions in micrometers)
qubits = {"q0": [0, 0], "q1": [5, 0], "q2": [0, 5]}
register = pulser.Register(qubits)

# 2. Select device
device = pulser.devices.AnalogDevice  # or DigitalAnalogDevice

# 3. Build sequence
seq = pulser.Sequence(register, device)
seq.declare_channel("rydberg", "rydberg_global")

# 4. Add pulses (Omega in rad/us, detuning in rad/us, duration in ns)
pulse = pulser.Pulse.ConstantPulse(
    duration=1000,   # ns
    amplitude=1.0,   # rad/us (Rabi frequency Omega)
    detuning=0.0,    # rad/us (delta)
    phase=0.0,       # radians
)
seq.add(pulse, "rydberg")
seq.measure("ground-rydberg")

# 5. Serialize for cloud submission
serialized = seq.to_abstract_repr()
```

**Available waveforms**: ConstantWaveform, RampWaveform, BlackmanWaveform,
KaiserWaveform, CustomWaveform, CompositeWaveform, InterpolatedWaveform

### 2.5 No Rust SDK

There is NO Rust SDK for Pasqal. The integration from Rust must use one of:
1. **Direct REST API calls** (reqwest) -- serialize Pulser-format JSON manually
2. **Python sidecar** -- call a Python script that uses pasqal-cloud + Pulser
3. **Azure Quantum SDK** -- if using Azure as intermediary (also Python-first)

**Recommendation**: Direct REST API. The JSON format from `to_abstract_repr()` is
well-defined and can be constructed in Rust without needing Python at runtime.

---

## 3. The Rydberg Hamiltonian

The full Hamiltonian that Pasqal hardware implements:

```
H(t)/hbar = sum_i [ (Omega(t)/2) * e^(-i*phi) |g_i><r_i|
                   + (Omega(t)/2) * e^(i*phi) |r_i><g_i|
                   - delta(t) * |r_i><r_i| ]
           + sum_{i<j} (C6 / R_ij^6) * n_i * n_j
```

Where:
- Omega(t) = Rabi frequency (amplitude of the driving laser)
- delta(t) = detuning (frequency offset from resonance)
- phi = phase of the driving field
- C6 = interaction coefficient (depends on Rydberg state)
- R_ij = distance between atoms i and j
- n_i = |r_i><r_i| = Rydberg state projector for atom i

**Key insight**: The interaction term `(C6/R_ij^6) * n_i * n_j` naturally encodes
a weighted graph where edge weights are determined by inter-atom distances. By
arranging atoms to match a graph topology, the hardware physically implements
the graph's interaction Hamiltonian.

---

## 4. Pricing

| Access Method | Pricing |
|---------------|---------|
| Pasqal Cloud direct | ~$300/QPU-hour (pay-as-you-go) |
| Azure Quantum | Azure Quantum pricing (varies) |
| Google Cloud | Marketplace pricing |

**Cost estimation per WeftOS operation**:
- A single quantum walk evolution (1000 shots, 100 atoms): ~$5-15
  (assuming ~1-3 minutes QPU time including calibration overhead)
- Emulator runs (EMU-TN): significantly cheaper, suitable for development
- EMU-FREE: zero cost for prototyping

---

## 5. Mapping WeftOS Operations to Pasqal

### 5.1 Graph -> Atom Register

The CausalGraph's adjacency structure maps to atom positions:

```
WeftOS CausalGraph              Pasqal Register
-------------------             ---------------
Node i at position p_i    ->    Atom "q_i" at coordinates [x_i, y_i]
Edge (i,j) with weight w  ->    Distance R_ij chosen so C6/R_ij^6 ~ w
No edge between i,j       ->    R_ij >> blockade radius (no interaction)
```

The mapping algorithm:
1. Extract the adjacency matrix A from CausalGraph
2. Use force-directed layout (or spectral embedding) to get 2D positions
3. Scale positions so that edge weights map to interaction strengths:
   R_ij = (C6 / w_ij)^(1/6) for connected nodes
4. Ensure non-adjacent nodes are far enough apart (R >> R_blockade)
5. Verify positions satisfy device constraints (min inter-atom distance)

**Limitation**: C6/R^6 decay means the interaction graph is inherently a
"unit-disk graph" -- atoms interact with all neighbors within range.
Exact arbitrary graph topologies require careful embedding. For graphs
with up to ~100 nodes, this is tractable.

### 5.2 Operation Mapping Table

| WeftOS Operation | Current Implementation | Pasqal Mapping | Benefit | Priority |
|---|---|---|---|---|
| **Coherent evolution** exp(-i*dt*H) | First-order approx in quantum_state.rs (line 169) | REAL quantum evolution via Rydberg Hamiltonian with graph-encoded register | Exact unitary evolution instead of O(dt) approximation | **P0** |
| **Born-rule measurement** P=\|alpha\|^2 | Simulated via norm_sq() (line 136) | REAL projective measurement on QPU; statistical sampling over shots | True quantum randomness and correct correlation structure | **P0** |
| **Evidence ranking** delta_lambda_2 | Classical formula w*(phi[u]-phi[v])^2 (line 187) | Quantum amplitude estimation on the evolved state | Quadratic speedup for large graphs | P2 |
| **Spectral analysis** (Lanczos) | O(k*m) Lanczos or RFF (cognitive_tick.rs line 9) | Quantum phase estimation on graph Laplacian | Exponential speedup for eigenvalue computation | P2 |
| **Graph partitioning** | Fiedler vector sign | Quantum walk naturally separates into communities via amplitude distribution | Inherent in the quantum walk dynamics | **P1** |
| **Subgraph exploration** | MCTS / beam search | Quantum walk explores ALL paths in superposition simultaneously | Quadratic speedup (Grover-like) | P1 |
| **Community detection** | Label propagation | Analog quantum optimization (QUBO/MIS formulation) | Natural fit for neutral atom hardware | P1 |
| **Hypothesis collapse** | HypothesisSuperposition::observe() (line 401) | Map hypotheses to atom groups, measure for Born-rule selection | True quantum measurement statistics | **P0** |

### 5.3 What NOT to Run on Pasqal

- **EML predictions** -- too fast classically (O(1)), quantum overhead not worth it
- **HNSW search** -- not a quantum-native operation
- **Embedding generation** -- better on GPU (neural network inference)
- **Cross-reference lookups** -- pure database operations

---

## 6. Integration Architecture

### 6.1 PasqalBackend Trait

```rust
/// Quantum backend abstraction for WeftOS.
/// Allows swapping between classical simulation and Pasqal QPU.
#[async_trait]
pub trait QuantumBackend: Send + Sync {
    /// Submit a quantum walk evolution job.
    /// Returns a job handle for async result retrieval.
    async fn submit_evolution(
        &self,
        graph: &CausalGraph,
        initial_state: &QuantumCognitiveState,
        evolution_time: f64,
        shots: u32,
    ) -> Result<JobHandle, QuantumError>;

    /// Poll for job completion and retrieve results.
    async fn get_results(&self, handle: &JobHandle) -> Result<Option<QuantumResults>, QuantumError>;

    /// Check if backend is available and healthy.
    async fn health_check(&self) -> Result<BackendStatus, QuantumError>;

    /// Maximum number of qubits/nodes supported.
    fn max_qubits(&self) -> usize;
}
```

### 6.2 Pasqal REST Client (Rust)

```rust
pub struct PasqalClient {
    http: reqwest::Client,
    api_url: String,
    auth_token: String,
    token_expiry: Instant,
    project_id: String,
    // Auth0 credentials for token refresh
    client_id: String,
    client_secret: Option<String>,
}

impl PasqalClient {
    /// Authenticate with Auth0 and get access token.
    async fn authenticate(&mut self) -> Result<(), PasqalError> {
        let resp = self.http.post("https://pasqal.eu.auth0.com/oauth/token")
            .json(&serde_json::json!({
                "grant_type": "client_credentials",
                "client_id": self.client_id,
                "client_secret": self.client_secret,
                "audience": "https://apis.pasqal.cloud/account/api/v1"
            }))
            .send().await?;
        let token: AuthResponse = resp.json().await?;
        self.auth_token = token.access_token;
        self.token_expiry = Instant::now() + Duration::from_secs(token.expires_in - 300);
        Ok(())
    }

    /// Create a batch with a serialized Pulser sequence.
    async fn create_batch(
        &self,
        sequence_json: serde_json::Value,
        jobs: Vec<JobSpec>,
        device_type: &str,
    ) -> Result<BatchResponse, PasqalError> {
        let resp = self.http.post(&format!("{}/api/v1/batches", self.api_url))
            .bearer_auth(&self.auth_token)
            .json(&serde_json::json!({
                "sequence_builder": sequence_json,
                "jobs": jobs,
                "device_type": device_type,
            }))
            .send().await?;
        Ok(resp.json().await?)
    }

    /// Get batch results (measurement bitstrings).
    async fn get_results(&self, batch_id: &str) -> Result<BatchResults, PasqalError> {
        let resp = self.http
            .get(&format!("{}/api/v1/batches/{}/results", self.api_url, batch_id))
            .bearer_auth(&self.auth_token)
            .send().await?;
        Ok(resp.json().await?)
    }
}
```

### 6.3 Graph-to-Pulser Serialization (Rust-native)

Instead of depending on Python/Pulser at runtime, construct the Pulser JSON
format directly in Rust:

```rust
/// Convert a CausalGraph to Pasqal register + sequence JSON.
pub fn graph_to_pulser_json(
    graph: &CausalGraph,
    evolution_time_ns: u64,
    omega: f64,
    detuning: f64,
) -> serde_json::Value {
    // 1. Compute 2D layout from graph (spectral embedding or force-directed)
    let positions = spectral_layout_2d(graph);

    // 2. Build register JSON
    let register: HashMap<String, [f64; 2]> = positions.iter().enumerate()
        .map(|(i, pos)| (format!("q{}", i), [pos.0, pos.1]))
        .collect();

    // 3. Build sequence JSON (Pulser abstract representation)
    serde_json::json!({
        "version": "1",
        "device": "AnalogDevice",
        "register": register.iter().map(|(name, coords)| {
            serde_json::json!({"name": name, "x": coords[0], "y": coords[1]})
        }).collect::<Vec<_>>(),
        "channels": {
            "rydberg": {
                "type": "rydberg_global",
                "operations": [{
                    "op": "pulse",
                    "duration": evolution_time_ns,
                    "amplitude": omega,
                    "detuning": detuning,
                    "phase": 0.0,
                    "waveform": "constant"
                }]
            }
        },
        "measurement": "ground-rydberg"
    })
}
```

**Important caveat**: The exact `to_abstract_repr()` JSON schema should be
reverse-engineered from the Pulser Python library or documented by Pasqal.
The above is an approximation. For production, validate against Pulser's
actual serialization format.

### 6.4 Hybrid DEMOCRITUS Loop

```text
Classical tick (20 Hz, CognitiveTickConfig default = 50ms):
  SENSE -> EMBED -> SEARCH -> UPDATE -> COMMIT
  |
  +-- Every N ticks (configurable, e.g. every 1000 ticks = 50s):
      |
      +-- Check: has the graph changed significantly since last quantum job?
      |   (measure by: number of new edges, entropy delta, drift count)
      |
      +-- If yes: submit quantum walk to Pasqal
      |   - Build register from current CausalGraph subgraph
      |   - Serialize as Pulser JSON
      |   - POST to /api/v1/batches
      |   - Store job handle
      |
      +-- On every tick: check for completed Pasqal jobs
          - GET /api/v2/jobs/{job_id}
          - If DONE: parse measurement bitstrings
          - Convert to probability distribution
          - Update QuantumCognitiveState.psi with real quantum data
          - Log quantum coherence metrics
```

### 6.5 Result Interpretation

Pasqal returns measurement outcomes as bitstrings (one per shot), where
each bit indicates whether atom i was in |g> (0) or |r> (1).

```rust
/// Convert Pasqal measurement bitstrings to WeftOS probability distribution.
pub fn bitstrings_to_probabilities(
    bitstrings: &[Vec<u8>],  // each inner vec: 0 or 1 per atom
    n_atoms: usize,
) -> Vec<f64> {
    let n_shots = bitstrings.len() as f64;
    let mut counts = vec![0u64; n_atoms];
    for bs in bitstrings {
        for (i, &bit) in bs.iter().enumerate() {
            if bit == 1 {
                counts[i] += 1;
            }
        }
    }
    counts.iter().map(|&c| c as f64 / n_shots).collect()
}

/// Update QuantumCognitiveState with real quantum measurement data.
pub fn update_state_from_pasqal(
    state: &mut QuantumCognitiveState,
    pasqal_probs: &[f64],
) {
    // Replace simulated amplitudes with sqrt of measured probabilities.
    // Phase information is lost in measurement -- use current phases.
    for (i, &p) in pasqal_probs.iter().enumerate() {
        let current_phase = state.psi[i].im.atan2(state.psi[i].re);
        let new_amp = p.sqrt();
        state.psi[i] = Complex::new(
            new_amp * current_phase.cos(),
            new_amp * current_phase.sin(),
        );
    }
    state.normalize();
}
```

---

## 7. Configuration

```toml
[kernel.quantum]
# Backend: "simulator" (default, uses existing quantum_state.rs),
#          "pasqal" (real QPU), "pasqal-emulator" (cloud emulator)
backend = "pasqal"

# Pasqal Cloud API
api_url = "https://apis.pasqal.cloud"
project_id = "your-project-id"

# Auth (service account recommended for production)
auth_method = "service_account"  # or "user"
client_id = "your-client-id"
client_secret_env = "PASQAL_CLIENT_SECRET"  # read from env var

# Device selection
device_type = "FRESNEL"  # FRESNEL, FRESNEL_SA1, EMU_TN, EMU_FREE

# Execution parameters
max_qubits = 100
default_shots = 100
job_timeout_s = 300

# Hybrid loop integration
submit_every_n_ticks = 1000      # submit quantum job every N cognitive ticks
min_graph_change_threshold = 10  # minimum new edges before submitting
evolution_time_ns = 1000         # quantum walk duration in nanoseconds
omega = 1.0                     # Rabi frequency (rad/us)
detuning = 0.0                  # detuning (rad/us)

# Subgraph selection (for graphs > 100 nodes)
max_subgraph_nodes = 100         # select most-connected subgraph
subgraph_strategy = "highest_degree"  # or "most_recent", "highest_entropy"
```

---

## 8. Limitations and Constraints

### Hardware Limitations
- **Max 100 qubits** on current Fresnel QPU (140+ on Orion Gamma, late 2025)
- **Unit-disk graph constraint**: Interaction strength is C6/R^6 -- cannot have
  two nodes interact strongly if they are physically far apart. The graph must
  be embeddable as a unit-disk graph (or close approximation).
- **No arbitrary Hamiltonian**: The Hamiltonian is fixed to the Rydberg form.
  You control Omega(t), delta(t), and phi, but the interaction term is always
  C6/R^6. This means the graph Laplacian is not directly implementable -- you
  get the adjacency-like interaction matrix instead.
- **Gate fidelity**: ~99% for single-qubit, ~97-99% for two-qubit (varies)
- **Coherence time**: Microseconds to milliseconds (depends on operation)
- **Repetition rate**: ~few Hz effective rate (including atom loading, calibration)
- **Minimum inter-atom distance**: Device-specific, typically ~4 micrometers

### Graph Laplacian vs Rydberg Hamiltonian

**Critical design consideration**: WeftOS uses the graph Laplacian L = D - A
(degree matrix minus adjacency) as its Hamiltonian. Pasqal's native Hamiltonian
encodes the adjacency matrix A through C6/R^6 interactions, not the Laplacian.

Options to reconcile:
1. **Use adjacency directly**: Redefine QuantumCognitiveState to use H = A
   instead of H = L. The quantum walk dynamics differ but still provide
   useful spectral information.
2. **Encode Laplacian via detuning**: Use the detuning delta_i to add the
   diagonal (degree) terms: H_eff = sum_i d_i |r_i><r_i| + sum_{i<j} w_ij n_i n_j.
   This requires local addressing (DigitalAnalogDevice, not pure AnalogDevice).
3. **Post-process**: Run adjacency-based quantum walk, then classically transform
   results to approximate Laplacian-based evolution.

**Recommendation**: Option 1 for the prototype. The adjacency-based quantum walk
still provides community detection, graph partitioning, and evidence ranking
capabilities. The Laplacian formulation can be added later with local addressing.

### API Constraints
- Max 1,000 jobs per batch
- Open batches time out after ~minutes without activity
- 24-hour token expiry
- Queue priority levels: CRITICAL, HIGH, MEDIUM, LOW, FREE

---

## 9. Rust Integration Approach: Direct REST (No Python Sidecar)

### Rationale

A Python sidecar would add:
- Process management complexity
- Python runtime dependency
- IPC overhead
- Deployment complexity (pip, virtualenv, etc.)

Instead, build the Pulser-format JSON directly in Rust:
1. The JSON schema for `to_abstract_repr()` is stable and documented
2. reqwest handles all HTTP needs
3. serde_json handles serialization
4. Auth0 token refresh is straightforward REST

### Dependencies (Cargo.toml additions)

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
# serde + serde_json already in workspace
```

### Module Structure

```
crates/clawft-kernel/src/
  quantum_state.rs           # existing -- classical simulation (unchanged)
  quantum_backend.rs         # NEW -- QuantumBackend trait
  quantum_pasqal.rs          # NEW -- PasqalBackend implementation
  quantum_pasqal_auth.rs     # NEW -- Auth0 token management
  quantum_pasqal_register.rs # NEW -- Graph -> Pulser register conversion
  quantum_pasqal_results.rs  # NEW -- Result parsing and state update
```

Feature-gated behind `pasqal`:
```toml
[features]
pasqal = ["reqwest"]
```

---

## 10. Prototype Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| P1: REST client | 1 week | Auth0 + batch creation + result retrieval |
| P2: Register mapping | 1 week | CausalGraph -> 2D atom positions |
| P3: Pulser JSON builder | 1 week | Rust-native sequence serialization |
| P4: EMU-FREE integration | 1 week | End-to-end on free emulator |
| P5: DEMOCRITUS integration | 1 week | Hybrid classical-quantum tick loop |
| P6: QPU validation | 1 week | Run on actual Fresnel hardware |

**Total**: ~6 weeks to prototype, assuming Pasqal Cloud account access.

---

## 11. Which Operations Benefit Most

Ranked by impact-to-effort ratio:

1. **Quantum walk for evidence exploration** (P0) -- Pasqal's sweet spot.
   Neutral atoms in arbitrary 2D geometries naturally encode graph topology.
   A single quantum walk explores all paths simultaneously, replacing the
   classical MCTS/beam search in O(sqrt(N)) instead of O(N).

2. **Born-rule measurement for hypothesis selection** (P0) -- True quantum
   measurement replaces simulated collapse. The correlation structure of
   real quantum measurements captures interference effects that classical
   simulation misses.

3. **Community detection via analog optimization** (P1) -- Maximum Independent
   Set (MIS) on neutral atoms is Pasqal's flagship algorithm. Mapping the
   CausalGraph community detection to MIS/QUBO is a natural fit.

4. **Coherent evolution exp(-iHt)** (P0) -- Replace first-order approximation
   (quantum_state.rs line 174: `psi' = psi - i*dt*H*psi`) with exact unitary
   evolution on hardware. This is the highest-fidelity improvement.

5. **Spectral analysis** (P2) -- Quantum phase estimation could provide
   eigenvalues of the graph Hamiltonian, but requires digital gates which
   are less mature on neutral atom hardware. Better for later phases.

---

## 12. Open Questions for Pasqal

1. **Exact JSON schema** for `to_abstract_repr()` -- need to reverse-engineer
   or get documentation for Rust-native serialization.
2. **Local addressing availability** on Fresnel -- required for encoding the
   Laplacian diagonal (degree terms) via per-atom detuning.
3. **Batch API rate limits** -- how many batches/hour are permitted?
4. **Result format specification** -- exact JSON schema for measurement outcomes.
5. **Academic/startup pricing** -- any discounts for research use?
6. **QoolQit availability** -- higher-level SDK that might simplify encoding
   of graph problems (mentioned in docs but details sparse).

---

## Sources

- [Pasqal Cloud Documentation](https://docs.pasqal.com/cloud/)
- [Pasqal Cloud SDK (pasqal-cloud)](https://docs.pasqal.com/cloud/pasqal-cloud/)
- [pasqal-io/pasqal-cloud GitHub](https://github.com/pasqal-io/pasqal-cloud)
- [Pulser SDK](https://github.com/pasqal-io/Pulser)
- [Pulser Register Documentation](https://docs.pasqal.com/pulser/register/)
- [Pulser Sequence Documentation](https://docs.pasqal.com/pulser/sequence/)
- [Pulser Programming Guide](https://docs.pasqal.com/pulser/programming/)
- [Authentication with Auth0](https://docs.pasqal.com/cloud/retrieve-token-from-auth0/)
- [Batches of Jobs](https://docs.pasqal.com/cloud/batches/)
- [PASQAL on Azure Quantum](https://learn.microsoft.com/en-us/azure/quantum/provider-pasqal)
- [Pasqal 2025 Roadmap](https://www.pasqal.com/newsroom/pasqal-releases-2025-roadmap/)
- [Graph Algorithms with Neutral Atom Quantum Processors](https://arxiv.org/html/2403.11931v1)
- [Pasqal 1000 Atoms Achievement](https://www.pasqal.com/newsroom/pasqal-exceeds-1000-atoms-in-quantum-processor/)
- [Device Specs API](https://docs.pasqal.com/cloud/api/core/operations/get_device_specs_by_device_type_api_v1_devices_specs__device_type__get/)
- [Pasqal Cloud Brochure](https://www.pasqal.com/wp-content/uploads/2025/06/Pasqal_Cloud_Brochure.pdf)
