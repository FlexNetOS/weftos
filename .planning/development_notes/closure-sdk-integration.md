# Closure-SDK — Integration Assessment for WeftOS

**Date**: 2026-04-14
**Evaluator**: Research agent (Opus 4.6, 1M context)
**Source repo**: https://github.com/faltz009/Closure-SDK (cloned to `/tmp/closure-sdk`)
**Commit evaluated**: `7d026a3` — 2026-04-14, 43 commits total, single author.
**License**: **AGPL-3.0-only** (see `/tmp/closure-sdk/LICENSE`, header of every `Cargo.toml`).
**Primary workload**: Rust (`closure_ea/`, `rust/`) + Python bindings (`closure_sdk/`, `closure_dna/`).

---

## Executive summary

Closure-SDK ("A Geometric Computer") is a single-author research implementation of
a cognitive architecture that represents every computational state as a unit
quaternion on S³ and evolves state via the Hamilton product. The Rust crate
`closure_ea` (roughly 11,700 lines across 17 files) is the interesting piece: a
five-layer stack (substrate → memory → execution → brain → learning) built on a
minimal sphere arithmetic (`compose`, `inverse`, `sigma`, `slerp`), a Hopf
decomposition of S³ into S² × S¹, a DNA/epigenetic "genome" store, a
"three-cell" diabolo ingest loop, and a consolidation/hierarchy layer. It reads
more like a paper artifact than a product SDK — much of the commentary is
Kabbalistic metaphor over algebraic content — but the algebra underneath is
real, 151 Python tests plus 55 Rust tests pass, and the primitives are
well-isolated.

The approach partially **overlaps** WeftOS's `QuantumCognitiveState` (complex
amplitudes over causal-graph nodes, unitary evolution, Born measurement) and
would **conflict** with WeftOS's classical `CausalGraph` as a replacement
substrate — Closure fundamentally does not expose a typed-edge DAG. It does
**not** overlap ECC's HNSW service, impulse queue, cross-reference store, mesh,
or governance paths. Its most transferable ideas are (a) quaternion-based
associative memory as an experiment alongside `QuantumCognitiveState`, (b)
the "seven-step ingest" / σ-gap prediction-error loop as a candidate
DEMOCRITUS inner-loop, and (c) the Hopf decomposition as a lightweight
"coherence vs. arrangement" classifier on any sequence.

**The AGPL-3.0 license is the dominant constraint.** WeftOS ships as permissively
licensed crates (weftos-*); linking AGPL code into the kernel forces the whole
network-distributed kernel to AGPL. This kills any form of direct linking.
Viable integration paths therefore sit at three places: a **separate
AGPL-isolated side-car process** (e.g. a `weftos-closure-bridge` daemon behind
an IPC boundary), a **clean-room reimplementation of the quaternion primitives**
(~500 lines of published algebra, not IP), or **conceptual influence only**
(no code reuse).

**Recommendation: defer / conceptual-only.** Closure's math is interesting and
its "σ-gap as prediction error" loop has real overlap with DEMOCRITUS and ECC
coherence scoring, but the licensing makes direct adoption unacceptable for a
permissively licensed product, the repo is single-author and under active churn
(three weeks old, 43 commits), there is no community traction to share
maintenance burden, and nothing in the existing ECC / EML / quantum / sonobuoy
roadmaps currently needs a quaternion substrate. Revisit only if (a) the
author relicenses under Apache-2.0 or dual-licenses, or (b) we identify a
concrete ECC workload — most plausibly associative recall or sequence-integrity
verification in DEMOCRITUS — where a quaternion-backed implementation measurably
beats the current vector/spectral approach.

---

## 1. The Closure-SDK method

### 1.1 Repository layout

The repository is a polyglot monorepo. Verified file inventory:

| Path | Language | Role | Size |
|------|----------|------|------|
| `closure_ea/` | Rust | "The Geometric Computer" — full brain | 17 `.rs` files, ~11,700 lines |
| `rust/` | Rust | Shared Rust core with PyO3 bindings | 9 `.rs` files, ~3,700 lines |
| `closure_sdk/` | Python | `pip install closure-sdk` data-integrity SDK | 9 `.py` modules |
| `closure_dna/` | Python | "Geometric database" | ~14 `.py` modules + Rust kernel |
| `closure_cli/` | Python | CLI surface | 7 `.py` modules |
| `benchmarks/`, `examples/`, `tests/`, `docs/` | Mixed | Support |  |

The `closure_ea/Cargo.toml` declares:

```toml
license = "AGPL-3.0-only"
dependencies = { sha2, serde, serde_json }    # only three, all permissive
```

So the Rust brain is dependency-light (3 third-party crates) — the license is
the problem, not a transitive-dependency explosion.

### 1.2 Core primitives (`closure_ea/src/sphere.rs`, 112 lines)

The entire substrate is five functions on `[f64; 4]`:

```rust
pub const IDENTITY: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
pub fn compose(a: &[f64; 4], b: &[f64; 4]) -> [f64; 4]     // Hamilton product + renormalize
pub fn inverse(a: &[f64; 4]) -> [f64; 4]                   // Star involution: conjugate
pub fn sigma(a: &[f64; 4]) -> f64                          // arccos(|w|) — geodesic distance from identity
pub fn slerp(a, b, t) -> [f64; 4]                          // Spherical linear interpolation
```

These are textbook quaternion operations — the file header is heavy on
Kabbalistic framing ("Chalal (Vacated Space) — S³ is the space produced by
Tzimtzum", "Hamilton product = Shefa — the divine flow composing the Sefirot")
but the code is unremarkable. There is nothing patentable or novel about the
mathematics itself; it is the five-layer architecture built on top that is the
author's contribution.

### 1.3 Hopf decomposition (`hopf.rs`, 465 lines)

Every carrier `[w, x, y, z]` splits into:

- `W` channel (scalar part `w`) — "existence / completion" axis.
- `RGB` channel (vector part `(x, y, z)`) — "position / arrangement" axes.

This is used for two purposes: (1) classifying divergences between two
sequences as either "missing record" (W dominates) or "reorder" (RGB
dominates) — the two incident types the SDK advertises as "algebraic inverses,
there is no third type"; (2) factorized addressing for the genome, where axis
queries find "semantic type" (on S²) and angle queries find "cyclic position"
(on S¹).

### 1.4 Memory model: `buffer`, `genome`, `field`

- **Buffer** (`buffer.rs`, 161 lines) — ring buffer of recent carriers. EMBED
  writes into it, ZREAD reads from it.
- **Genome** (`genome.rs`, 1083 lines) — two-layer key-value store keyed by
  quaternion address:
  - `Dna` layer: permanent, seeded from bootstrap. Never mutated after phase 5.
    Analogous to read-only anchors (Watson-Crick pairs, S³ topology).
  - `Epigenetic` layer: mutable, written by ingest. Supports reinforce / correct
    / create based on `address_gap` thresholds (`reinforce <= 0.05 < novelty <= 0.35`).
  - A third `Response` layer (not fully described in the header comments) holds
    "reality corrections" written by the evaluative half-loop.
- **Field** (`field.rs`, 555 lines) — two read operators over the `genome ∪ buffer`
  population:
  - `RESONATE`: nearest-address match (hard selection).
  - `ZREAD`: weighted composition over all entries within a π/3 σ-neighborhood
    (coalition / soft read). Weight falloff is `cos(σ)`.

### 1.5 Verification verb (`verify.rs`, 152 lines)

`verify(a, b)` computes `cycle = compose(a, inverse(b))` and classifies the
gap:

```rust
pub enum ClosureKind { Identity, Balance, Open }    // σ=0, σ=π/4, or neither
pub enum HopfDominance { W, Rgb, Balanced }         // scalar vs. vector part dominates
```

π/4 is the "BKT phase boundary" where the W and RGB channels carry equal norm.
The file asserts this is forced by the geometry ("unit sphere + 1=3 condition"),
not a tunable parameter.

### 1.6 Hierarchy & closure detection (`hierarchy.rs`, 459 lines)

A stack of `ClosureLevel`s. Each level composes incoming carriers into a
running Hamilton product, detects closure when:
- The product returns near identity (`Carry` role) or the balance locus (`FixedPoint`
  role), AND
- An excursion peak, local minimum, and nontrivial support conditions are met
  ("Law 1").

On closure, the localized packet (found in O(log n) via `localize` — `localization.rs`)
is written to the genome and cascaded upward to the next level. The author
frames this as the "Sefirotic tree"; operationally it is hierarchical sequence
compression with closure detection.

### 1.7 Three-Cell diabolo (`three_cell.rs`, 2061 lines — the largest file)

The main ingest loop — this is what makes the thing a "brain". Per-tick:

```
1.  bytes ← outside world
2.  c ← EMBED(bytes)
3.  buffer ← buffer ∪ {c}
4.  f ← ZREAD over (genome ∪ buffer)
5.  s' ← RESONATE(f)
6.  s ← LOAD(slot of s' in genome)
7.  ev ← VERIFY(s, s')
```

Each `ingest` call produces a `Step` record with 15+ fields: prediction error
σ, self-free-energy σ, valence (signed coherence signal), fiber parity,
closure events, consolidation reports, semantic frame, and two neuromodulation
tones (arousal, coherence). Cell A is the fast oscillator (raw-input running
product), Cell C is the slow accumulator of closure events, and the "diabolo"
is the coupled dynamics between them. `evaluate_prediction` is the evaluative
half-loop — after reality judges a staged output, correction writes back to the
Response layer.

### 1.8 Neuromodulation (`neuromodulation.rs`, 168 lines)

Session-ephemeral exponential-moving-average filter over step pressure and
valence. Two tones in [0,1] and [-1,1] respectively. Not persisted. EMA alpha
derived as `1 - 1/buffer_lifetime`.

### 1.9 Learning (`teach.rs`, 357 lines) and execution (`execution.rs`, 426 lines)

- `teach` — `(input, target)` curriculum pairs feed the same ingest runtime,
  measured by σ-gap. Convergence is framed as a fixed-point theorem.
- `execution` — `MinskyMachine` (2-counter Turing-complete), `FractranMachine`
  (Collatz/prime orbits), `OrbitRuntime`. Demonstrates Turing completeness
  on the carrier substrate; orthogonal to the main ingest loop.

### 1.10 What is genuinely novel

Stripping away the Kabbalistic framing, the **architecturally novel claims** are:

1. **Unified arithmetic**: every mutation is a single `compose` call (Hamilton
   product + renormalize). No other arithmetic is needed at the substrate level.
2. **Two-incident-type closure**: every divergence between two ordered streams
   lands on exactly the W axis (missing) or the RGB axes (reorder), and these
   are algebraic inverses of each other. Called out as the "Zeroth Law" in
   the standalone PDF.
3. **σ-gap as prediction error**: the geodesic distance between Cell C's
   accumulated model and the incoming carrier is used directly as free-energy,
   with no learned error function.
4. **BKT phase boundary at σ = π/4**: the equator of S³ is claimed to be a
   topologically forced consolidation threshold, not a hyperparameter.
5. **DNA/epigenetic separation**: a layered memory where bootstrapped anchors
   are permanent and learned traces are mutable, with a one-way path from
   epigenetic → response but not back.

Claims (1)–(3) are plausibly transferable ideas; (4) is a specific claim whose
empirical validation outside this codebase would need verification; (5) is a
pattern, not a technique.

### 1.11 What is not there / red flags

- **No distributed or multi-node story.** The entire brain assumes a single
  in-process state machine. There is no mesh, no gossip, no CRDT, no clock.
- **No CRDT / OT / event-sourcing story.** Convergence is local, not
  distributed. The "field read" over `genome ∪ buffer` is single-actor.
- **No effect system or capability model.** Unlike effect handlers or
  capability-ring designs, Closure does not reason about side effects.
- **No actor / process model.** There is one loop, not many.
- **The three-cell file is 2061 lines**, most of it implementing metaphors
  described in the top-of-file Kabbalistic commentary. Code-level coupling is
  high; the commentary:code ratio approaches 1:1 in some files.
- **"Turing complete" on S³** is demonstrated by embedding a 2-counter Minsky
  machine in the execution layer. This is a proof-of-principle — the Hamilton
  product can simulate any computation. It is not an argument that doing so
  would be efficient.
- **The author's framing** ("Kabbalistic correspondence", "Senchal 2026"
  theorems cited inline) is unusual for a production SDK and suggests the
  work is at paper-artifact maturity, not production maturity.
- **Single author, 43 commits, ~4 weeks old.** No community. No bus-factor.
  Support section of README lists BTC/ETH/PIX addresses for donations.

---

## 2. Comparison to the WeftOS substrate

Module-by-module analysis of
`/claw/root/weavelogic/projects/clawft/crates/clawft-kernel/src/`.

### 2.1 `causal.rs` — CausalGraph DAG

**WeftOS**: typed, weighted directed edges (`Causes`, `Inhibits`, `Correlates`,
`Enables`, `Follows`, `Contradicts`, `TriggeredBy`, `EvidenceFor`). Lock-free
via `DashMap`. Universal Node IDs (32-byte hashes) for cross-structure
reference. BFS traversal, path finding.

**Closure**: no typed-edge DAG. Edges in the genome are "sequential" — what
followed what in the stream — and are implicit to the running Hamilton
product. There is no semantic edge type, no weight, no explicit path finding.
The "genome entries" have addresses and values, but the relationship between
two entries is encoded geometrically (σ distance) rather than as an edge.

**Verdict**: **Fundamental mismatch.** Closure's memory model does not expose
the kind of graph operations ECC needs (traversal, typed edges, path queries).
You cannot drop Closure under ECC as a substrate. This is the main
architectural blocker.

### 2.2 `cognitive_tick.rs` — ECC heartbeat

**WeftOS**: configurable tick interval (default 50ms), adaptive to compute
budget, boot-calibrated, advertised to peers.

**Closure**: no tick. Ingest is call-driven, synchronous, one carrier at a
time. The inner seven-step loop runs per ingest call, not per wall-clock tick.

**Verdict**: **Orthogonal.** If Closure were to be embedded, the `ingest` call
would be the "compute" portion of a WeftOS cognitive tick; the tick layer
still belongs to WeftOS.

### 2.3 `quantum_state.rs` / `quantum_backend.rs` — experimental quantum layer

**WeftOS**: `QuantumCognitiveState` stores complex amplitudes per causal-graph
node (`psi: Vec<Complex>`), applies `exp(-i*dt*H)` evolution using the graph
Laplacian as H, and implements Born-rule measurement. Optional offload to
Pasqal Fresnel neutral-atom hardware via `QuantumBackend`.

**Closure**: no complex amplitudes, no Hamiltonians, no measurement. But it
*does* do a related thing: carriers are unit quaternions (SU(2) ≅ S³), and
`exp_su2_gates` experiment exhibits a single-qubit gate dictionary. The
`sphere.rs` operations are exactly what you need to simulate one qubit; the
resonance/field layer is loosely analogous to a superposition over states.

**Verdict**: **Closest point of potential overlap.** WeftOS's quantum layer
uses a full complex state vector over N graph nodes (2N real amplitudes);
Closure uses a single 4-element quaternion (4 real numbers) as the entire
"state". These are different scales of representation solving different
problems. A quaternion-based associative memory could live *beside*
`QuantumCognitiveState` as a cheap classical coherence check, but it is not
a replacement.

### 2.4 `eml_kernel.rs` — EML learned-function layer

**WeftOS**: replaces hardcoded thresholds (governance scoring, restart backoff,
health thresholds, dead-letter policy, gossip timing, complexity threshold)
with small trainable `EmlModel` instances, with fallback to the original
heuristic when untrained.

**Closure**: no learned functions in the ML-threshold-replacement sense.
Genome learning is geometric (SLERP toward observed carriers) and proceeds
from σ-gap, not from labeled training pairs. The `teach` module runs
curriculum-style training but the "model" is the genome itself.

**Verdict**: **Strictly orthogonal.** EML models specific kernel heuristics;
Closure models a whole cognitive loop. Neither subsumes the other.

### 2.5 `hnsw_service.rs` — vector index

**WeftOS**: thread-safe HNSW store, cosine similarity, ~280 lines, pluggable
with DiskANN and hybrid backends for scale.

**Closure**: no HNSW, no vector index in the ML-embedding sense. Nearest-
neighbor lookups are linear scans of the genome using σ distance on
quaternions. For the scale the author targets (demos, tests) this works;
for ECC scale (thousands to millions of entries) it would need replacement
by an index — at which point the Closure-specific contribution is just the
distance metric, not the retrieval pipeline.

**Verdict**: **Closure would need HNSW replacement for any WeftOS workload.**

### 2.6 `impulse.rs` — inter-structure event queue

**WeftOS**: ephemeral ordered queue of events between the four ECC structures
(`BeliefUpdate`, `CoherenceAlert`, `NoveltyDetected`, `EdgeConfirmed`,
`EmbeddingRefined`). HLC-sorted for causal ordering.

**Closure**: no inter-structure messaging because there are no independent
structures to message between — the whole brain is one module.

**Verdict**: **Orthogonal — Closure has no analog.**

### 2.7 `crossref.rs` — cross-structure references

**WeftOS**: `UniversalNodeId` (BLAKE3 hash + structure tag + local ID),
`CrossRef` with typed relationships (`DerivedFrom`, `References`, `Contradicts`,
`Supersedes`), forward/reverse index.

**Closure**: no cross-reference. Everything the brain knows lives in one
genome, addressed geometrically.

**Verdict**: **Orthogonal.**

### 2.8 `governance.rs`, `ipc.rs`, mesh/cluster

**WeftOS**: full governance effect-vector scoring, named-pipe IPC, mesh
bootstrap/discovery/noise/tcp/ws stack, Kad DHT, mDNS, gossip.

**Closure**: none of the above. Closure is a pure cognition library with no
I/O, no networking, no security story, no governance.

**Verdict**: **Closure would be a leaf in WeftOS, not a replacement for any of
these subsystems.**

### Module-by-module summary

| WeftOS module | Closure overlap | Verdict |
|---------------|-----------------|---------|
| `causal.rs` | None (no typed-edge DAG) | Fundamental mismatch |
| `cognitive_tick.rs` | None (call-driven, not ticked) | Orthogonal |
| `quantum_state.rs` | Quaternion ≈ 1-qubit; no amplitudes over graph | Closest overlap |
| `quantum_backend.rs` | None | Orthogonal |
| `eml_kernel.rs` | None (learns a brain, not thresholds) | Orthogonal |
| `hnsw_service.rs` | Linear σ-scan of genome; no index | Closure needs HNSW to scale |
| `impulse.rs` | None (single-module brain) | Orthogonal |
| `crossref.rs` | None | Orthogonal |
| `governance.rs`, `ipc.rs`, mesh | None | Orthogonal |

---

## 3. Comparison to ECC / EML / quantum layer

### 3.1 ECC

Closure and ECC solve adjacent but not overlapping problems. ECC gives WeftOS
a **forest of trees** (ExoChain, Resource Tree, Causal Graph, HNSW) linked by
CrossRefs and Impulses, with explicit causal edges and semantic search. Closure
gives its author a **single-manifold cognitive engine** where all memory,
addressing, and dynamics happen on S³ and are read by geodesic distance.

The closest ECC concepts to Closure concepts:

| ECC concept | Closure analog | Same idea? |
|-------------|----------------|------------|
| CausalGraph edge weight | σ-gap between genome entries | Different formalism: graph edges vs. geometric distance |
| HNSW nearest-neighbor | RESONATE (linear σ-scan) | Similar semantics, incompatible scale |
| Impulse (CoherenceAlert) | σ-gap exceeding π/4 | Potentially same pattern, expressed differently |
| Novelty detection | `novelty_threshold = 0.35` create-new-entry path | Different mechanisms |
| DEMOCRITUS two-tier (EML + Lanczos) | ZREAD soft / RESONATE hard | Similar two-tier idea, incompatible implementations |

**The strongest ECC concept overlap** is at the **coherence scoring** boundary.
ECC's `CoherenceAlert` impulse fires when the graph becomes incoherent; Closure
fires closure events when a running product crosses π/4 or returns to identity.
Both are **coherence-as-geometry** ideas applied at different scales. It is
conceivable that the σ-gap formulation could inform a new coherence metric
inside `eml_coherence.rs`, but the implementation would be
clean-room reimplementation of two-line math — nothing to directly adopt.

### 3.2 EML

EML is a threshold-replacer; Closure is a cognitive substrate. They operate
at different levels. Closure could, in principle, be wrapped as a single
`EmlModel` — "given a history of carriers, predict the next one" — but that
would throw away everything interesting about Closure (the genome, the
hierarchy, the consolidation) and reduce it to a regressor. Not useful.

### 3.3 Quantum layer

This is the most interesting comparison. Both systems:

- Use a continuous state space (Closure: S³, Quantum: complex projective space CP^(n-1)).
- Measure evolution against a reference (Closure: IDENTITY, Quantum:
  measurement basis).
- Track probability/amplitude-like quantities (Closure: σ-gap falloff
  `cos(σ)`, Quantum: `|<k|ψ>|²`).

But they differ critically in dimension and scope:

| Property | `QuantumCognitiveState` | Closure state |
|----------|-------------------------|---------------|
| Dim | 2 × N (N = # graph nodes) | 4 per carrier |
| Operator | Graph Laplacian H | Hamilton product |
| Evolution | `exp(-i·dt·H)` | `compose(a, b)` |
| Hardware offload | Pasqal Fresnel QPU (Rydberg) | None |

A plausible bridge: use Closure-style quaternion carriers as the **per-node
cognitive state** that dresses each node of the causal graph, with the
graph-level evolution still driven by `QuantumCognitiveState`. This gives each
node a lightweight associative-memory head while keeping the macro-scale graph
dynamics unchanged. But this would be a novel design, not an adoption of
Closure — and we would have to write the quaternion code ourselves (see §6).

### 3.4 Symmetry and prior decisions

From the HP decisions (`HP-14..16`) and Sprint 17 work (ECC substrate, 83
tests, quantum layer at 27 tests), WeftOS has already committed to the
causal-graph / HNSW / impulse / quantum-amplitude architecture. Switching the
substrate to a quaternion manifold would invalidate nine ADRs and roughly
one sprint of shipped work for unclear benefit.

---

## 4. Comparison to the sonobuoy project

The sonobuoy project (`/claw/root/weavelogic/projects/clawft/.planning/sonobuoy/`)
is a K-STEMIT-extended dual-branch spatio-temporal GNN for distributed
hydrophone arrays. Its core problems are:

- **Beamforming over a sparse, irregular buoy array** (GNN message passing).
- **Detection / bearing / species ID** (three task heads over a shared fused
  embedding).
- **Physics priors** (Helmholtz PINN, FNO propagation, FiLM-conditioned
  thermocline).
- **Foundation-model transfer** (BEATs, SurfPerch/Perch, AudioMAE).

None of these map naturally onto Closure-SDK. The problems are:

1. **High-dimensional signal processing.** DEMON / spectrogram features, GCC-PHAT
   TDOA, 1280-d foundation-model embeddings. Closure operates on 4-d
   quaternion carriers. Scale mismatch.
2. **Graph-structured arrays.** Sonobuoy positions form a weighted graph used
   for spatial message passing. This is where WeftOS's `quantum_register`
   graph-to-layout mapping is explicitly reused. Closure has no graph model.
3. **Foundation-model retrieval.** Species ID benchmarks use HNSW-indexed
   foundation embeddings. Closure's linear σ-scan cannot compete here.
4. **Distributed processing.** Sonobuoy's K-STEMIT architecture is inherently
   multi-node. Closure is single-module.

**Where Closure could contribute**, tangentially:

- **Sequence integrity over buoy telemetry.** The `closure_sdk` Python library
  (not the brain) claims "one-comparison" detection of missing/reordered
  records in ordered streams, with O(log n) localization via the hierarchy
  layer. If buoy packets are ordered and loss-detection is currently
  implemented via sequence numbers + timeouts, Closure's Hopf-decomposed
  incident classification could in principle tell "missing" apart from
  "late" in one geometric step. This is a marginal optimization; sequence
  numbers plus HLC already work.
- **Conceptual influence only.** The σ-gap formulation of prediction error
  could inspire the detection-head loss function, but any practical
  implementation would use the published sonobuoy papers' formulations, not
  Closure's.

**Verdict: not applicable.** The sonobuoy project does not benefit from Closure
beyond a marginal sequence-integrity optimization that is already solved by
simpler means.

---

## 5. Integration options

### 5.1 Option A — Surgical adoption (clean-room quaternion module, no code reuse)

**What**: Reimplement the six quaternion primitives (IDENTITY, compose, inverse,
sigma, slerp, Hopf decomposition) in a new WeftOS module
`crates/clawft-kernel/src/quaternion_assoc.rs` under the existing WeftOS
license (Apache-2.0 / MIT dual). Use it as an experimental **classical
associative memory** service that sits alongside `QuantumCognitiveState` for
workloads where a 4-d coherence check is cheaper than a full amplitude
evolution. Optionally add a `QuaternionCoherenceAlert` impulse type.

**Effort**: ~2-3 days. The published math (Hamilton product, SLERP, Hopf
decomposition) is textbook. The interesting work is deciding what the
carriers represent in WeftOS (causal-graph node states? HNSW centroids?
Sonobuoy signal phases?).

**Risk**: Low on the code side, high on the "why are we doing this"
side. Without a specific benchmarkable workload, the module would be
unused glue.

**What changes in WeftOS**: one new ~500-line module, maybe one new impulse
variant, zero changes to causal graph, HNSW, EML, mesh, governance.

**What doesn't**: everything else.

**Licensing**: No Closure code is copied. Algebra is not copyrightable.
Safe.

### 5.2 Option B — Substantial crate-level integration (AGPL side-car)

**What**: Include `closure_ea` as an optional dependency in a new
`crates/weftos-closure-bridge/` crate that is **also AGPL-3.0-only** and is
**not linked** into the main kernel. The kernel talks to it over IPC (existing
`named_pipe.rs`) or a local HTTP endpoint. The bridge crate exposes a small
RPC: `embed(bytes) -> carrier`, `ingest(carrier) -> Step`, `resonate(carrier) -> hit`.
Kernel code calls into it opportunistically when a user opts in at runtime.

**Effort**: 1-2 weeks including IPC surface, tests, build-feature wiring,
and documentation making the license boundary explicit.

**Risk**:
- License correctness is subtle. The FSF's position on AGPL over IPC is
  nuanced; if a court later decided IPC does constitute "network interaction"
  in the AGPL sense, the kernel could inherit AGPL obligations.
- Users who deploy WeftOS with the bridge activated would need to ship
  source for any modifications to the bridge over the network. This is a
  surprise for most WeftOS consumers.
- Single-author upstream, 4-week-old repo, no community — high maintenance
  risk if upstream goes inactive or pivots.

**What changes in WeftOS**: one new optional crate, one new feature flag
(`closure-bridge`), documentation of the licensing boundary. No changes to
kernel-proper.

**What doesn't**: the main kernel, all existing tests, all existing crates.

**Licensing**: **High risk.** AGPL side-car is legally defensible but
operationally hostile to commercial WeftOS users. Recommend against unless
the author dual-licenses.

### 5.3 Option C — Conceptual-only influence

**What**: Read the `GEOMETRIC_CLOSURE_PAPER.md` (41 KB, in the repo) and the
Zenodo papers. Extract the three transferable ideas — (a) σ-gap as prediction
error, (b) W/RGB incident classification, (c) π/4 consolidation threshold —
and evaluate whether any fit into existing WeftOS roadmap items:

- ECC coherence scoring (`eml_coherence.rs`): can σ-gap inform the coherence
  score? Probably yes, but as inspiration not code.
- DEMOCRITUS inner loop: the seven-step ingest loop has the shape of a
  predictive-coding cycle. Could DEMOCRITUS's step be reformulated as "predict
  → ingest → verify → correct"? Worth a design spike.
- Sonobuoy detection-head loss: could a σ-style manifold distance serve as a
  loss? Unlikely to beat published approaches; not worth pursuing.

**Effort**: 1 day of paper reading + a design memo in
`.planning/development_notes/` if anything sticks.

**Risk**: Zero licensing risk (ideas are not copyrightable). Zero code risk
(no code is added). The only risk is the opportunity cost of the time spent.

**What changes in WeftOS**: nothing until a concrete spike lands.

**What doesn't**: everything.

### 5.4 Summary of options

| Option | Effort | License risk | WeftOS change | When it's the right call |
|--------|--------|--------------|---------------|--------------------------|
| A: Clean-room primitives | 2-3 days | None | ~500-line module | We identify a real quaternion-flavored workload |
| B: AGPL side-car | 1-2 weeks | High | New optional crate | Closure relicenses OR we accept AGPL for a subset |
| C: Conceptual only | 1 day | None | Maybe a memo | Default — we learn something without committing |

---

## 6. Licensing and dependency analysis

### 6.1 Closure-SDK license

From `/tmp/closure-sdk/LICENSE` (first line: "GNU AFFERO GENERAL PUBLIC LICENSE
Version 3, 19 November 2007") and every Cargo.toml (`license = "AGPL-3.0-only"`):

- **AGPL-3.0-only**, not "-or-later".
- The AGPL-3.0 triggers its source-distribution requirement on **network
  interaction** with modified versions, not just on binary distribution. This
  is the key difference from GPL-3.0.

### 6.2 What this means for WeftOS

WeftOS ships as a kernel that participates in a distributed mesh, advertises
capabilities over the network, and serves HTTP/WS APIs from `http_api.rs` and
`mesh_ws.rs`. Any AGPL code linked into the kernel would make the **entire
deployed kernel** — including every other crate in the workspace — subject to
AGPL's network-distribution clause. In practice this means:

- Every WeftOS deployment would have to expose source for any kernel
  modifications to users over the network.
- Any customer who embeds WeftOS in a commercial product would effectively
  have to ship their modifications as open source.
- Most of the existing WeftOS crates are permissively licensed (Apache-2.0 /
  MIT). Mixing AGPL in would force all of them to AGPL downstream.

This is incompatible with the WeftOS GTM story (selling WeftOS to clients for
knowledge-graph / automation use cases — see `MEMORY.md` GTM note).

### 6.3 Closure-SDK dependency tree

From `/tmp/closure-sdk/closure_ea/Cargo.toml` and `/tmp/closure-sdk/rust/Cargo.toml`:

```
closure_ea:    sha2, serde, serde_json                    (all permissive — BSD/MIT/Apache)
closure_rs:    pyo3, numpy, rand, sha2, serde, serde_json (all permissive)
```

No transitive AGPL dependencies in the Closure code itself. The license
problem is purely Closure's own choice, not inherited. This matters: if the
author ever relicenses under Apache-2.0 or dual-licenses, the dependency tree
is immediately compatible.

### 6.4 Algebra is not code

The mathematics Closure-SDK uses (Hamilton product, SLERP, Hopf fibration, BKT
transitions, Minsky machines) is all in the public domain — centuries to
decades old. **Reimplementing the quaternion primitives from the textbook is
not infringement.** Only the specific code, its comments, and the identifier
naming are copyrighted. Option A in §5 is therefore legally clean even while
the upstream is AGPL.

### 6.5 Conclusion

- **No direct linking** into the kernel at any license (AGPL blocks it).
- **Side-car IPC** is legally defensible but operationally hostile.
- **Clean-room reimplementation** is legally fine.
- **Conceptual influence** is legally fine.

---

## 7. Recommendation

**Defer with conceptual-only engagement (Option C).** Concretely:

1. **Do not adopt Closure-SDK as a dependency** in any form. The AGPL license,
   single-author status, 4-week age, and research-artifact maturity all rule
   it out for direct use in WeftOS.
2. **Read the accompanying paper** (`closure_ea/docs/GeometricComputer.pdf`,
   plus Zenodo 19578024 and 19140055) once if time permits. Capture any
   transferable ideas — specifically σ-gap prediction error and W/RGB incident
   classification — as a short design-memo addendum to this file if they
   prove to have a concrete application.
3. **Monitor upstream for relicensing.** If the author moves to Apache-2.0 or
   dual-licenses, revisit — Option A (clean-room primitives) stops being
   necessary because Option B becomes viable.
4. **Do not pursue Option A (clean-room module) speculatively.** A
   quaternion-based classical associative memory is a cool idea, but
   WeftOS already has HNSW + `QuantumCognitiveState` covering that space.
   Unless a sonobuoy or DEMOCRITUS workload specifically needs a
   4-d manifold coherence metric — and none does today — adding code is
   premature.
5. **Revisit if Sprint 20+ identifies a concrete quaternion-flavored
   workload.** Candidates: (a) sonobuoy packet-integrity check as
   competition to sequence-numbers, (b) ECC coherence signature for
   per-node cognitive state, (c) robotics pose composition in the
   DEMOCRITUS servo loop (quaternions are already the standard pose
   representation).

**Go/no-go**: **No-go on direct integration. Soft-yes on periodic
re-evaluation when upstream matures or relicenses.**

---

## Appendix A — Glossary of Closure-SDK terminology

| Term | Meaning |
|------|---------|
| **S³** | The unit 3-sphere: unit quaternions with the Hamilton product. |
| **IDENTITY** | `[1, 0, 0, 0]` — the quaternion identity. "North pole of S³". |
| **σ (sigma)** | `arccos(|w|)` — geodesic distance from identity. 0 = perfect coherence; π/2 = antipodal. |
| **BKT threshold / π/4** | The equatorial σ value where W and RGB channels carry equal norm. Claimed to be a forced consolidation threshold. |
| **Hopf fibration** | The map S³ → S² × S¹ that factors a quaternion into a 2-sphere base (S²) and a circle fiber (S¹). |
| **W axis / channel** | The scalar part of a quaternion. Associated with "existence / completion" / "missing" incidents. |
| **RGB axes / channel** | The vector part (x, y, z) of a quaternion. Associated with "position / arrangement" / "reorder" incidents. |
| **EMBED** | SHA-256-based hash of bytes to a carrier on S³. |
| **COMPOSE** | Hamilton product + renormalize. The only verb at the substrate level. |
| **VERIFY** | `A ?= A` — computes `compose(a, inverse(b))`, reads σ, classifies closure kind. |
| **RESONATE** | Nearest-address match from `genome ∪ buffer` (hard selection). |
| **ZREAD** | Weighted composition over all population entries within π/3 σ-neighborhood of a query (soft / coalition read). |
| **DNA layer** | Permanent bootstrapped genome entries. Never mutated. "Nitzotzot / sparks". |
| **Epigenetic layer** | Mutable learned genome entries. Written by ingest; consolidated by sleep. |
| **Response layer** | Reality-correction genome entries written by the evaluative half-loop. |
| **ThreeCell / diabolo** | The main ingest loop. Cell A = fast oscillator (raw input running product), Cell C = slow accumulator (closure events), "diabolo" = coupled dynamics. |
| **Cell A σ** | Sigma of the running Hamilton product of raw input since the last closure. |
| **Cell C σ** | Sigma of the brain's accumulated prediction. |
| **Prediction error** | `σ(cell_c, incoming_carrier)` — external free energy. |
| **Self-free-energy (SFE)** | `σ(cell_c, zread_at_query(cell_c))` — how surprised the brain is at its own genome. |
| **Valence** | `SFE(t-1) - SFE(t)` — signed coherence change. Positive = improving model. |
| **Arousal tone** | Low-pass EMA of normalized step pressure. ∈ [0, 1]. |
| **Coherence tone** | Low-pass EMA of normalized valence. ∈ [-1, 1]. |
| **Closure event** | Running product returned to IDENTITY (Carry) or equator (FixedPoint). Localized packet emitted. |
| **ClosureRole** | `Carry` (inter-level handoff, σ→0) or `FixedPoint` (intra-level balance, σ→π/4). |
| **ClosureKind (verify)** | `Identity` (σ=0), `Balance` (σ=π/4), or `Open` (neither). |
| **Gilgamesh** | SDK operation: spatial divergence between two complete sequences. |
| **Enkidu** | SDK operation: temporal divergence — is record absent or just late? |
| **Sefirot / Nitzotzot / Chalal / Tzimtzum / Shefa / Parochet / Partzuf** | Kabbalistic metaphors used throughout for architectural components. Not functional terms. |
| **Zeroth Law** | Author's claim that divergence between ordered streams falls on exactly W (existence) or RGB (position) axes, no third type. |

---

## Appendix B — Open questions for Closure-SDK maintainers

1. **License.** Is the AGPL-3.0-only choice intentional and permanent, or is
   a dual-license (Apache-2.0 + AGPL) under consideration? Direct integration
   into permissively licensed downstream projects is gated on this.
2. **π/4 as a forced threshold.** The code asserts that σ = π/4 is
   topologically forced rather than tunable. Is there a proof in the
   accompanying paper that is independent of the carrier encoding choice? Or
   is it forced conditional on the EMBED scheme?
3. **Scale.** RESONATE is a linear scan of the genome. What is the largest
   genome size the author has tested under real workload? At what size does
   RESONATE become the bottleneck relative to ZREAD?
4. **Distribution.** Is there any plan for a multi-node brain — genome sync,
   gossip, CRDT — or is single-process the intended endpoint?
5. **Non-SHA-256 carriers.** EMBED uses SHA-256 to map bytes to S³. Has any
   alternative (e.g. learned embedding → projection to S³) been tested? How
   sensitive are the closure thresholds to the embedding distribution?
6. **Incident completeness claim.** The Zeroth Law asserts "no third
   incident type". Does this hold under adversarial inputs (bit-flips
   in mid-sequence, partial reorders, cross-stream insertions)? Is there an
   empirical test harness for this claim?
7. **Cell A solenoidal gap.** `three_cell.rs` acknowledges Cell A is the
   least developed and references Senchal §9.3 open question Q8. What is
   the current best guess at the gap, and does it affect convergence
   guarantees?
8. **Neuromodulation as session-ephemeral.** Why is the neuromodulation
   state intentionally not persisted? In a long-running agent this seems to
   throw away slow-timescale signal. Is that by design?
9. **Comparison to existing associative memories.** Has the author benchmarked
   the genome against Hopfield networks, modern Hopfield (Hopfield is all you
   need), or VSA/HRR representations, which solve analogous problems?
10. **Turing completeness via Minsky.** The `exp_turing` experiment
    demonstrates Turing completeness but does not argue for efficiency. Is the
    efficiency argument somewhere, or is Turing completeness claimed only
    at the theoretical level?

---

*End of assessment.*
