# NVIDIA Quantum / "Ising" Integration Assessment for WeftOS

Research note — 2026-04-14
Author: research sweep for the v0.6.x quantum cognitive layer
Status: DEFER (with one conditional PILOT)

Sources verified during this pass:

- NVIDIA Ising landing page: `https://www.nvidia.com/en-us/solutions/quantum-computing/ising/`
- NVIDIA press release, 2026-04-14: `https://nvidianews.nvidia.com/news/nvidia-launches-ising-the-worlds-first-open-ai-models-to-accelerate-the-path-to-useful-quantum-computers`
- `https://github.com/NVIDIA/Ising-Decoding` — verified Apache-2.0
- `https://huggingface.co/nvidia/Ising-Calibration-1-35B-A3B` — verified `nvidia-open-model-license`
- `https://github.com/NVIDIA/cuda-quantum` — verified Apache-2.0
- `https://github.com/NVIDIA/cuQuantum` (Python bindings + samples) — verified BSD-3-Clause
- NVIDIA cuQuantum SDK docs: `https://docs.nvidia.com/cuda/cuquantum/latest/`
- `https://github.com/NVIDIA/cuopt` — verified Apache-2.0
- NVIDIA NVQLink page: `https://www.nvidia.com/en-us/solutions/quantum-computing/nvqlink/`
- `https://developer.nvidia.com/cuquantum-sdk`, `https://developer.nvidia.com/cuda-q`

---

## Executive Summary

**NVIDIA's "Ising" brand is misleading.** Despite the URL
`/quantum-computing/ising/`, the product announced 2026-04-14 is **not** an
Ising / QUBO solver. It is a **family of open AI models** named after the
Ising physics model. The family has exactly two members today:

1. **Ising-Calibration** — a 35B-parameter vision-language model (Qwen3.5
   derivative) fine-tuned to infer QPU calibration actions from lab
   measurement data.
2. **Ising-Decoding** — 0.9M / 1.8M-parameter 3D CNNs that pre-decode
   surface-code syndromes before handing off to PyMatching; reported 2.5×
   faster and 3× more accurate than PyMatching alone.

Neither one helps WeftOS directly. Both solve problems (physical QPU
calibration, surface-code QEC decoding) that exist one abstraction layer
**below** the Pasqal Cloud API we already consume. Pasqal handles its own
calibration; we never see syndromes. Ising is a hardware-vendor tool, not a
client-workload accelerator.

The underlying NVIDIA stack around Ising — **CUDA-Q, cuQuantum
(cuStateVec/cuTensorNet/cuDensityMat), cuOpt, NVQLink** — is more
substantive. CUDA-Q and cuQuantum could provide a legitimate **third
`QuantumBackend`** implementation running GPU-accelerated analog Rydberg
simulation locally (useful for dev / CI / pre-flight validation before
Pasqal submissions). Both are Apache-2.0 or BSD-3-Clause at the source
level, though they depend on proprietary NVIDIA CUDA runtime and the
cuQuantum SDK binary, which is an NVIDIA redistributable (not OSI-approved
but compatible with closed linking for Apache-2.0 software).

None of the NVIDIA tooling is **built for Rust**. Integration is either
(a) subprocess to a Python/CUDA-Q runner, or (b) FFI against the CUDA-Q C++
API or cuQuantum C API. There is no `cuquantum` or `cuda-quantum` crate on
crates.io (verified 2026-04-14).

**Recommendation: DEFER.** The one concrete pilot worth queueing is a
**cuDensityMat-backed `SimulatorBackend`** behind a `quantum-nvidia` feature
flag, Python-sidecar first, FFI later. That can wait until v0.7.x after GUI
ships. Everything else (Ising models, cuOpt, NVQLink) is out of scope for
WeftOS.

---

## 1. What NVIDIA Actually Ships

NVIDIA's quantum offering is a **stack**, not a single library. The public
list as of 2026-04-14:

| Component | Purpose | Repo / Home | License | Status |
|-----------|---------|-------------|---------|--------|
| **Ising-Calibration** | VLM for QPU calibration automation | huggingface.co/nvidia/Ising-Calibration-1-35B-A3B | nvidia-open-model-license (Qwen3.5 base: Apache-2.0) | v1, April 2026 |
| **Ising-Decoding** | 3D-CNN pre-decoder for surface codes | github.com/NVIDIA/Ising-Decoding | **Apache-2.0** | v0.1.0, April 2026 |
| **CUDA-Q (cuda-quantum)** | Hybrid quantum-classical programming model (C++/Python) with `nvq++` compiler | github.com/NVIDIA/cuda-quantum | **Apache-2.0** | Production |
| **CUDA-Q QEC** | Quantum error correction library on top of CUDA-Q | Shipped inside cuda-quantum | Apache-2.0 (parent) | Integrated |
| **cuQuantum SDK** | C libraries: cuStateVec, cuTensorNet, cuDensityMat, cuPauliProp, cuStabilizer | docs.nvidia.com/cuda/cuquantum/ | NVIDIA SLA (binary redistributable) | Production |
| **cuQuantum Python** | Python bindings + samples | github.com/NVIDIA/cuQuantum | **BSD-3-Clause** | Production |
| **cuOpt** | GPU MILP/LP/QP/VRP solver (**decision optimization**) | github.com/NVIDIA/cuopt | **Apache-2.0** | v26.06 |
| **NVQLink** | Hardware interconnect spec for QPU ↔ GPU at ~μs latency | nvidia.com/.../nvqlink/ | Hardware spec | 2026 reference platform |
| **DGX Quantum** | Reference server: Grace Hopper + QPU control electronics | Product page | Hardware | Available |
| **NVIDIA Quantum Cloud** | Hosted CUDA-Q + simulators | Product page | SaaS | GA |
| **NIM microservices** | Containerized inference endpoints for Ising models | build.nvidia.com | Proprietary runtime, open models | Preview |

### What's not there

- **No NVIDIA Ising solver / QUBO solver / simulated annealer.** Despite
  the URL, there is no D-Wave-style combinatorial optimizer. cuOpt covers
  linear and mixed-integer programming, not Ising/QUBO — and cuOpt is not
  linked from the "Ising" page at all. (Marketing opted to reuse the Ising
  name for the AI model family.)
- **No public roadmap for an NVIDIA Ising solver.** The press release is
  explicit that "Ising" refers to the open AI model family.
- **No Rust crates.** Searched crates.io for `cuda-quantum`, `cuquantum`,
  `cuopt` — all returned zero results.

---

## 2. Apples to Oranges: NVIDIA vs. Our Existing Stack

| Axis | WeftOS (shipped v0.6.7) | NVIDIA quantum stack |
|------|-------------------------|----------------------|
| Hardware target | Neutral-atom **analog QPU** (Pasqal Fresnel, QuEra Aquila) | Classical **GPU**, plus QPU hardware integration via NVQLink for QPU builders |
| Hamiltonian | Rydberg analog (Ω, Δ, φ over time) | Any gate-model circuit, plus analog dynamics via cuDensityMat |
| Access model | Cloud REST (Pasqal Auth0 → batch submit → poll) | Local library, local GPU, or NVIDIA Quantum Cloud |
| Language | Rust (`QuantumBackend` trait) | C++ / Python |
| Current capability | 25 qubits EMU_FREE, 100 qubits Fresnel | 40 qubits demoed on 1024 GPUs (Eos); tensor-net paths go higher |
| Who it serves | Client / application workload author | QPU builder, quantum algorithm researcher |
| WeftOS role | Accelerate ECC graph-spectral ops (community detection, quantum walks, Born-rule ranking) | Would simulate circuits / evolve Hamiltonians on GPU |

**Verdict: complementary, not competitive.** NVIDIA's stack sits beside
Pasqal/Braket, not in place of it. The realistic integration is using
cuDensityMat (or a `tensornet` backend inside CUDA-Q) as a **local
simulator** that implements the same `QuantumBackend` trait as
`PasqalBackend`, so dev / CI / pre-production can run without burning
Pasqal queue time.

The Ising AI models are useful only if **we operate our own QPU**. We do
not.

---

## 3. Per-Library Assessment

### 3.1 Ising-Calibration (the 35B VLM)

- **What it does:** Consumes images of QPU lab measurements (e.g. Rabi
  oscillation plots) plus a prompt and emits structured calibration
  commands. Fine-tuned from Qwen3.5-35B-A3B (MoE).
- **WeftOS fit:** **None.** Pasqal Cloud ships a pre-calibrated device.
  We never touch calibration; that's the vendor's job. The only path to
  using this model is if we built our own neutral-atom lab, which is not
  happening.
- **License:** `nvidia-open-model-license` — NVIDIA's "Open Model License,"
  reviewed: permissive for commercial use and derivatives but with
  **attribution, export control, and responsible-use clauses**. Base model
  (Qwen3.5) is Apache-2.0. Not a blocker if we ever wanted it; we don't.
- **Integration effort:** N/A.
- **Verdict:** **Ignore.**

### 3.2 Ising-Decoding (3D-CNN pre-decoder)

- **What it does:** Consumes surface-code detector syndromes (space ×
  time), predicts corrections that reduce syndrome density, hands off to
  PyMatching for the final decision. PyTorch training framework; ONNX
  export path; integrates with CUDA-Q QEC for realtime benchmarking.
- **WeftOS fit:** **None.** Surface codes are a low-level QEC primitive
  for gate-model fault-tolerant quantum computing. Pasqal's analog Rydberg
  platform doesn't expose syndromes at all — it's analog, not logical. We
  don't operate a surface-code processor and never will.
- **License:** **Apache-2.0** — compatible. No blocker.
- **Integration effort:** N/A (no valid use case).
- **Verdict:** **Ignore.**

### 3.3 CUDA-Q (the programming model)

- **What it does:** A C++ and Python framework for writing hybrid
  quantum-classical programs. Includes `nvq++` compiler, a runtime, and
  backends for GPU simulation (`nvidia` = cuStateVec, `tensornet`,
  `density-matrix-cpu`), CPU simulation, and real QPUs via vendor plugins.
  Supports an analog-neutral-atom programming surface (`photonics`,
  `dynamics`) and QPU hardware via NVQLink.
- **WeftOS fit:** **Moderate, indirect.** CUDA-Q by itself does not
  evolve Rydberg Hamiltonians — you write a kernel and pick a backend. The
  interesting piece for us is the `dynamics` target plus cuDensityMat,
  which evolves arbitrary time-dependent operators. That gives us a local
  Rydberg simulator path that mirrors Pasqal's `EMU_TN` service but on our
  own hardware.
- **License:** **Apache-2.0**. Clean.
- **Runtime:** Requires CUDA 12+, NVIDIA GPU (Volta+ for cuOpt, Ampere+
  recommended for cuQuantum). CPU fallback exists (`density-matrix-cpu`,
  `qpp`) but the point is the GPU.
- **Rust integration:** No crate. Options:
  1. **Python sidecar** — run `cudaq` from a Python subprocess, talk JSON
     over stdin/stdout or a socket. Lowest effort, matches how we already
     plan to emit Pulser sequences in complex cases (see
     `quantum_pasqal.rs` preamble comment about generating from the
     Pulser Python SDK).
  2. **FFI against libcudaq.so** — works but requires rebuilding on every
     CUDA version and is not a public-C API story NVIDIA promises to
     maintain.
  3. **HTTP to NVIDIA Quantum Cloud** — cleanest but cloud-hosted,
     removes the "runs on your GPU" benefit.
- **Verdict:** **Pilot candidate** as a local simulator backend.

### 3.4 cuQuantum SDK (cuStateVec, cuTensorNet, cuDensityMat, cuPauliProp, cuStabilizer)

- **What it does:**
  - `cuStateVec` — state-vector simulation on GPU (good for small, deep,
    highly entangled circuits).
  - `cuTensorNet` — tensor-network contraction (good for shallow, wide
    circuits; scales further than state vector).
  - `cuDensityMat` — **time-dependent operator action on density matrix.
    This is the analog Hamiltonian simulator**, demoed at 1024 GPUs / 40
    qubits / 1.44M Hilbert-space levels (NVIDIA Eos). Directly comparable
    to what Pasqal's `EMU_TN` does, but we own the runtime.
  - `cuPauliProp` — Pauli-string propagation for Clifford + low-T circuits.
  - `cuStabilizer` — stabilizer-formalism simulation (Clifford-only).
- **WeftOS fit:**
  - `cuDensityMat` is the **interesting one**. It can simulate a Rydberg
    Hamiltonian with the same (Ω, Δ, φ, position) control surface we
    already expose in `EvolutionParams`. A Rust shim that maps our
    `QuantumCognitiveState` + atom register into cuDensityMat inputs would
    let us run the same workload locally on an A100/H100.
  - `cuStateVec` could serve small quantum-walk problems (our ECC cases
    typically 20-100 nodes) if we keep gate-model reductions.
  - `cuTensorNet` is interesting only if we go beyond ~30 nodes and want
    MPS-style approximation.
  - `cuPauliProp` / `cuStabilizer` — not relevant (we don't live in
    Clifford land).
- **License:** C/C++ SDK is NVIDIA proprietary binary (redistributable).
  Python bindings + samples are **BSD-3-Clause**. The SDK EULA permits
  linking from Apache-2.0 software, but binaries cannot be redistributed
  as source. Practical effect: we'd depend on users installing the SDK
  separately, like CUDA itself.
- **Rust integration:** None out of the box. The C API (`cudensitymatXxx`
  functions) is FFI-bindable. A `bindgen` pass against the installed SDK
  headers gets us an `unsafe` crate quickly. Wrapping it in a safe
  `cuDensityMatBackend: QuantumBackend` is ~1 sprint of Rust work.
- **Verdict:** **Pilot candidate**, specifically `cuDensityMat`.

### 3.5 cuOpt

- **What it does:** GPU-accelerated MILP, LP, QP, and vehicle-routing.
  Does **not** solve QUBO or Ising problems natively. Has a Python API,
  C API, and a REST/gRPC server mode.
- **WeftOS fit:**
  - Our ECC community-detection task is discrete/combinatorial. cuOpt's
    MILP solver can express community detection as an ILP (max modularity
    with indicator variables) but at the cost of a quadratic formulation
    blow-up. Likely slower than Louvain/Leiden on CPU for our sizes
    (<10k nodes).
  - Sonobuoy multistatic problem: MILP formulation is plausible (sensor
    allocation / coverage ILP), and cuOpt is well-suited. But that's a
    different product surface than the quantum layer; it would be a
    *classical optimizer* addition to `weftos-research` or a future
    signal-processing crate, not to `clawft-kernel`'s `QuantumBackend`.
- **License:** **Apache-2.0**. Clean.
- **Rust integration:** No crate. C API available (FFI feasible) or REST
  server (subprocess / HTTP client). Python API is the most mature.
- **Verdict:** **Reject for the quantum layer.** Revisit for classical
  optimization (sonobuoy, graph partitioning) in a separate initiative.
  Not relevant to the "quantum cognitive layer" scope.

### 3.6 NVQLink

- **What it does:** Hardware interconnect + real-time C++ API inside
  CUDA-Q for μs-latency QPU↔GPU control loops. Aimed at QPU builders
  running on-site control electronics.
- **WeftOS fit:** **None.** We consume a cloud QPU (Pasqal). We never see
  the QPU control loop. NVQLink is for teams building QPUs in-house.
- **License:** Hardware specification; surrounding software ships with
  CUDA-Q (Apache-2.0).
- **Verdict:** **Ignore.** Interesting context for the robotics ECC story
  (DEMOCRITUS/Sesame) only insofar as it's a proof that μs-latency
  classical ↔ physical-substrate loops are being industrialized — similar
  cognitive motif to ECC's servo loop. No code impact.

### 3.7 CUDA-Q QEC

- **What it does:** QEC library inside CUDA-Q. Integrates PyMatching and
  Ising-Decoding. Targets surface codes.
- **WeftOS fit:** **None.** See §3.2.
- **Verdict:** **Ignore.**

### 3.8 NIM Microservices for Ising

- **What it does:** Hosted inference endpoints for Ising-Calibration /
  Ising-Decoding as a service.
- **WeftOS fit:** **None** (no use case for the underlying models).
- **Verdict:** **Ignore.**

### 3.9 DGX Quantum / Quantum Cloud

- **What it does:** Hardware product and hosted service for running
  CUDA-Q workloads. Quantum Cloud exposes CUDA-Q over a hosted endpoint.
- **WeftOS fit:** If we adopted CUDA-Q as a backend, Quantum Cloud could
  be a zero-install option (pay NVIDIA for GPU time instead of buying an
  H100). Relevant only as a **deployment target**, not as a library.
- **Verdict:** **Defer** (only matters if Option A/B below is taken).

---

## 4. Concrete Integration Options

### Option A — NVIDIA-backed `QuantumBackend` as a third backend

**Shape.** Add `crates/clawft-kernel/src/quantum_nvidia.rs` implementing
`QuantumBackend`, gated by feature `quantum-nvidia`. Under the hood, the
implementation either:

- A1. Spawns a Python sidecar that runs CUDA-Q's `dynamics` target and
     pipes results back as the same `QuantumResults` shape, or
- A2. Links cuDensityMat via FFI, mapping `EvolutionParams` + register →
     time-dependent Lindblad operator → evolved density matrix →
     sampled bitstrings.

**Effort.**
- A1 (Python sidecar): ~1 sprint. Reuses the Pulser-sidecar pattern
  already anticipated in `quantum_pasqal.rs` for complex sequences.
- A2 (FFI): ~2–3 sprints. Includes `bindgen`, a safe wrapper, a build
  script that finds the cuQuantum SDK, and CI infra that installs CUDA.

**Risk.**
- A1: Python dependency in the runtime; every release needs CUDA-Q
  installed on the target. Acceptable only as a dev/CI backend, not a
  shippable product feature.
- A2: binds WeftOS binaries to an NVIDIA-SDK-present build environment.
  Cannot be published to crates.io with the SDK bundled. Users without
  CUDA can't build unless we make the feature optional (which it would be
  anyway).

**Licensing.**
- CUDA-Q source: Apache-2.0 — compatible.
- cuQuantum SDK binaries: NVIDIA SLA — users install themselves, same
  posture as CUDA. OK for Apache-2.0 distribution of WeftOS.
- No AGPL anywhere.

**What changes in WeftOS.**
- New crate feature `quantum-nvidia`, default off (mirrors
  `quantum-pasqal` / `quantum-braket`).
- New file `quantum_nvidia.rs` (~400–600 LOC for A1, 1000+ for A2).
- New ADR-048 "NVIDIA simulator backend" establishing the Python-sidecar
  posture and the envelope of supported CUDA-Q features.
- Docs: new section in `docs/src/content/docs/weftos/quantum.mdx`
  describing when to pick which backend (cost vs. fidelity vs. queue
  time).
- CI: a matrix job that runs the simulator backend on a GPU runner (or
  skips when none available).

**Value.**
- Unblocks integration tests that today must talk to `EMU_FREE`.
- Lets paying WeftOS users pre-validate a Rydberg evolution locally on
  their own H100 before burning Pasqal QPU hours.
- Increases our "vendor optionality" story — three backends instead of
  two (one of which is a stub).

### Option B — Pre-production simulation (circuit pre-flight)

**Shape.** Don't add a new backend at all. Instead, add a
`pre_flight(&sequence) -> PreFlightReport` helper in `quantum_register` /
`quantum_state` that shells out to CUDA-Q (Python) to simulate the same
Rydberg evolution locally and returns an expected-fidelity / atom-
population sketch. Used optionally before `submit_evolution` to catch bad
sequences.

**Effort.** ~0.5 sprint. Sidecar-only; no new trait impl.
**Risk.** Very low. Optional feature, dev-only tooling.
**Licensing.** Same as Option A (CUDA-Q is Apache-2.0).
**What changes.** One helper module, one CLI subcommand
(`weft quantum validate <sequence.json>`), one docs page.
**Value.** Operational: fewer wasted Pasqal submissions. Not a
strategic feature — a quality-of-life add for researchers.

### Option C — Classical Ising solver on GPU for ECC

**Assumption:** use NVIDIA's stack as a **classical** solver to bypass the
QPU entirely for community detection / graph partitioning.

**Reality check.** There is no NVIDIA Ising solver. cuOpt does MILP/LP/QP,
not QUBO or Ising. A MILP formulation of community detection is possible
but (a) expensive and (b) not where cuOpt's sweet spot sits. If we want
GPU-classical community detection, we'd use RAPIDS `cuGraph` (not covered
on the Ising page, but part of NVIDIA's stack and Apache-2.0).

**Verdict on C as stated:** **Reject.** The premise (NVIDIA ships an Ising
solver) does not hold. Revisit as a separate classical-graph-algorithms
initiative using cuGraph if we ever need GPU-classical graph partitioning
for very large causal graphs (>10⁶ nodes). Not on the critical path for
the quantum cognitive layer.

### Option D — Conceptual-only

Keep the NVIDIA Ising announcement in the ECC / quantum robotics mental
model as an **external validator**: major industry actor publishing that
(a) AI is being used as the "control plane" for quantum hardware, (b)
surface-code decoding can be pre-decoded by a small CNN faster than
classical matching. Both echoes of our ECC story:

- The "AI as control plane" framing is literally the DEMOCRITUS servo-loop
  thesis.
- The "small fast CNN pre-decoder feeds into a classical decision
  algorithm" pattern is structurally the same as our HNSW retrieval →
  classical evidence-rank pipeline in ECC.

**Effort.** Zero beyond citing it.
**Value.** Reinforces our public narrative; add a sentence in the ECC
whitepaper and the `quantum.mdx` doc noting NVIDIA's announcement as
supporting evidence for the analog + AI hybrid model.

---

## 5. Recommendation

**Primary: DEFER.** The NVIDIA "Ising" announcement does not move our
roadmap. The two shipped models are vendor-facing (calibration, QEC
decoding) and solve problems that Pasqal already solves for us or that we
don't have.

**Secondary: PILOT cuDensityMat as a local simulator backend, post-GUI.**
Queue this as **ADR-048** and target v0.7.x, scheduled **after the GUI
ships** per the versioning rule in project memory.

- **Sprint target:** First sprint of v0.7.x after GUI work concludes.
- **Path:** Start with **Option B** (pre-flight sidecar, ~0.5 sprint) to
  exercise the CUDA-Q toolchain without committing trait surface. If
  value confirmed, graduate to **Option A1** (Python-sidecar
  `QuantumBackend`), then later **A2** (direct FFI) only if a serious
  user needs binary-only deployment.
- **Feature flag:** `quantum-nvidia`, default off. Mirrors existing
  `quantum-pasqal` / `quantum-braket`.
- **Non-goal:** do not adopt CUDA-Q's programming model for everyday
  WeftOS quantum kernels. Our `EvolutionParams` / `QuantumCognitiveState`
  trait surface stays as-is; CUDA-Q is strictly a target for that trait.
- **Secondary follow-up (not this ADR):** evaluate cuOpt and cuGraph
  separately for classical sonobuoy / causal-graph partitioning work.
  Those do not belong under the quantum cognitive layer.

**Explicit rejects in scope:** Ising-Calibration, Ising-Decoding, CUDA-Q
QEC, NVQLink. None of them fit.

**Conditional adopt:** **none** today. The pilot becomes a proper
"adopt" decision only after Option B and Option A1 spikes land and prove
the latency / fidelity trade vs. Pasqal's `EMU_FREE` and `EMU_TN`.

---

## 6. Open Questions

Questions to resolve during the Option B spike (or by reaching out to
NVIDIA quantum support / Developer Program):

1. **cuDensityMat analog-Rydberg example.** Is there a reference example
   that drives an array of Rb-87 atoms under the same (Ω(t), Δ(t), φ(t))
   control surface Pasqal's `AnalogDevice` uses? The docs list "arbitrary
   operator action" but do not publish a drop-in Rydberg notebook we
   could port directly.
2. **CUDA-Q `dynamics` backend maturity.** Is it production-grade or
   research-preview? The CUDA-Q docs list it but the cuDensityMat Release
   Notes suggest it was added recently.
3. **SDK redistribution.** Can we ship a container (weftos-nvidia:latest)
   with the cuQuantum SDK baked in, or does the NVIDIA SLA require
   end-user download? This affects whether we can offer a one-liner
   install for the `quantum-nvidia` feature.
4. **Sampling semantics.** Does cuDensityMat return bitstring samples
   natively, or do we have to post-sample from the evolved density
   matrix ourselves? Affects the Rust adapter surface.
5. **Python sidecar overhead.** What's the end-to-end latency for a
   10-qubit, 1 μs-evolved sequence through a Python subprocess? If it's
   dominated by Python start-up cost, we'll want a long-running daemon
   rather than per-call subprocesses.
6. **License confirmation for cuQuantum Python.** The Python
   bindings repo is BSD-3-Clause, but the underlying `libcudensitymat`
   binary ships under the NVIDIA SDK EULA. Confirm with NVIDIA legal that
   a user of an Apache-2.0 Rust app linking a BSD-3 Python wrapper that
   loads an NVIDIA-SDK `.so` is unambiguous (we think yes, same as
   CUDA-proper).
7. **NVIDIA Open Model License fine print.** Not needed for the pilot,
   but if Ising-Calibration ever becomes interesting downstream (e.g., a
   robotics calibration model for DEMOCRITUS), re-read the license for
   responsible-use / export-control clauses.

---

## Appendix — Glossary of NVIDIA Quantum Terminology

- **CUDA-Q** — NVIDIA's hybrid quantum-classical programming model. C++
  and Python front-ends, `nvq++` compiler, pluggable backends
  (simulator / QPU).
- **cuQuantum** — NVIDIA SDK of quantum-simulation primitives. Contains
  five sub-libraries: cuStateVec, cuTensorNet, cuDensityMat, cuPauliProp,
  cuStabilizer. Counterpart to cuBLAS / cuDNN but for quantum.
- **cuStateVec** — Full state-vector simulation on GPU. O(2^n) memory.
- **cuTensorNet** — Tensor-network contraction. Handles wide-shallow or
  MPS-factorable circuits beyond state-vector reach.
- **cuDensityMat** — Density-matrix evolution for time-dependent
  Hamiltonians; the "analog Hamiltonian simulator" inside cuQuantum.
  Closest match to what Pasqal's Fresnel physically does.
- **cuPauliProp** — Pauli-operator propagation through circuits; good for
  low-T Clifford-plus-T expectation values.
- **cuStabilizer** — Stabilizer (Clifford-only) simulation, including QEC
  decoding.
- **NVQLink** — Hardware interconnect + real-time C++ API in CUDA-Q for
  μs-latency QPU ↔ GPU control loops; for QPU builders.
- **DGX Quantum** — Reference hardware server (Grace Hopper + Quantum
  Machines OPX+ control electronics).
- **NVIDIA Quantum Cloud** — Hosted service exposing CUDA-Q and
  simulators via the cloud. Pay for GPU-hours.
- **cuOpt** — GPU-accelerated decision optimization (MILP/LP/QP/VRP).
  Not an Ising solver.
- **NIM** — NVIDIA Inference Microservice. Containerized model-serving
  endpoints.
- **Ising (the NVIDIA product)** — Open AI model family (2026-04-14), not
  an Ising physics solver. Two members: Ising-Calibration (35B VLM) and
  Ising-Decoding (small 3D CNN). Named after the Ising mathematical
  model, not *of* it.
- **Ising model (the physics)** — Lattice of spin-½ variables with
  pairwise interactions and external field; equivalent to QUBO up to a
  linear transform; the target Hamiltonian for D-Wave-style annealers.
  **Not** what NVIDIA's product line solves.
- **Surface code** — A class of quantum error-correcting codes on a 2D
  lattice. Foundation of most fault-tolerant gate-model roadmaps. Not
  relevant to the analog-Rydberg WeftOS workload.
- **Pre-decoder (in QEC)** — A fast inner classifier that shrinks the
  syndrome graph before a slower matching decoder (PyMatching) finalizes
  the correction. What Ising-Decoding implements.
- **Rydberg Hamiltonian** — The controllable Hamiltonian of a
  neutral-atom QPU. Parameters: per-atom position, global / local Rabi
  frequency Ω, detuning Δ, phase φ. What we already expose via
  `EvolutionParams`.
- **EMU_FREE / EMU_TN / FRESNEL** — Pasqal Cloud device tiers. EMU_FREE
  is a 25-qubit free emulator, EMU_TN a 100-qubit tensor-network
  emulator, FRESNEL the 100-qubit physical QPU.
