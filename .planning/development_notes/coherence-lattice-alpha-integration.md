# coherence-lattice-alpha — WeftOS Integration Evaluation

**Repo**: [`project-89/coherence-lattice-alpha`](https://github.com/project-89/coherence-lattice-alpha)
**Commit evaluated**: `5b1c738` (HEAD of `master`, 5 commits total, all April 2026)
**Author**: Michael Sharpe (Project 89)
**License**: CC BY-NC 4.0 (paper + figures) + AGPL-3.0 (scripts)
**Date of evaluation**: 2026-04-14
**Evaluator**: automated research sweep (verify-every-claim pass)

---

## Executive summary

**coherence-lattice-alpha is not a software lattice library. It is a physics preprint.**
The repository contains (1) a 1,541-line LaTeX paper (`paper.tex`) and compiled PDF deriving the
fine structure constant α ≈ 1/137.036 from a self-organizing classical oscillator model on the
diamond lattice, and (2) ~28 standalone Python/NumPy/SciPy verification scripts that reproduce the
numerical results in the paper. There is no library, no API, no typed interface, no package
manifest, no versioned crate, and no tests in the software-engineering sense. Several scripts
(`diamond_clr_convergence.py`, `fiedler_proof.py`) **hard-depend on a module outside this
repository** — `experiments/lattice_theory/canon/v4_2026-02-14/184b_lt183_vortex_ring_g2_convergence.py`
— and will not run against a fresh clone. This is a preprint auxiliary codebase, not a toolkit.

The novel technical object is the **Coherence Learning Rule (CLR)** — a dynamical equation of
motion that treats lattice bond couplings K_ij as learnable variables. The CLR has two channels:
a **Shannon** channel that drives each bond toward its local information optimum, and a **Fiedler**
channel that uses the graph-Laplacian second eigenvector (algebraic connectivity) as a
budget-conserving structural signal. The Fiedler-channel overlap with WeftOS's `eml_coherence.rs`
(which also predicts λ₂ of the causal graph Laplacian) is the only non-trivial conceptual contact
point, but the two modules use λ₂ for opposite purposes — the CLR uses it as a *forcing term*, the
ECC substrate uses it as a *coherence predictor*.

**Recommendation: DEFER. Do not adopt, port, or integrate now.**
The repo is single-author, preprint-stage (tag-less, 5 commits), AGPL-3.0 on its code
(incompatible with WeftOS's current licensing posture), and its claims are extraordinary physics
claims that have not been peer-reviewed. There is one defensible conceptual takeaway — the
Shannon-plus-Fiedler two-channel pattern — that may be worth logging as prior art when we refine
the `eml_coherence` module in a future sprint, but it does not justify a kernel change today.

---

## 1. The coherence-lattice-alpha method (verified)

### 1.1 Repository layout (actual)

```
/tmp/coherence-lattice-alpha/
├── AGENTS.md             11 KB   agent onboarding (derivation chain, equations)
├── LICENSE                2 KB   CC BY-NC 4.0 + AGPL-3.0 (two-licence repo)
├── Makefile             392 B    LaTeX build only (pdflatex + bibtex)
├── README.md             5 KB    narrative summary
├── paper.tex           103 KB   1,541-line paper, revtex4-2 (APS style)
├── paper.pdf            1.5 MB   compiled output
├── paper.bbl + .bib     17 KB   bibliography (28 entries)
├── data/                        3 JSON precomputed result files (~60 KB total)
├── figures/                     7 PNG + 1 HTML plot (~1.2 MB)
└── scripts/             ~850 KB 28 Python scripts (AGPL-3.0)
```

No `Cargo.toml`, `pyproject.toml`, `setup.py`, `package.json`, or `requirements.txt`. Dependencies
are declared in the README as a single `pip install numpy scipy matplotlib` line.

Git state: 5 commits, 0 tags, 0 releases, 0 open issues, 0 PRs. This is a pre-submission preprint.

### 1.2 What the paper claims

From `paper.tex` §5 ("The Fine Structure Constant") and `README.md` (cross-verified):

Starting inputs: π, e, modified Bessel functions I₀/I₁, diamond-lattice coordination number z = 4.
Single principle: bond couplings K_ij obey their own gradient flow ("Coherence Learning Rule")
that maximises a scalar called *coherence capital* = phase alignment × structural richness.

From this alone the paper claims to derive:

| Quantity | Lattice value | CODATA | Precision |
|---|---|---|---|
| 1/α | 137.035999 | 137.035999206 | 1.5 ppb |
| g_e | 2.002319304355 | 2.002319304361 | 11.4 digits |
| "Static-lattice" (no CLR) | 143.134 | — | wrong by 4.4% |

The 11.4-digit match for g_e is obtained, per `scripts/g_factor_from_lattice.py` line 84+, by
feeding the lattice-derived α into the **standard** QED perturbative series C₁..C₅ with a published
a_had and a_ew; the paper does not re-derive QED loop coefficients from the lattice. See
`g_factor_from_lattice.py:486-490` for the author's own honest statement: *"100% from the alpha
gap … QED coefficients are standard (no lattice modification)."* So the "11.4 digits" is
downstream precision amplification and should not be read as a lattice prediction.

### 1.3 The CLR equation (from `AGENTS.md` §"Key Equations" and `paper.tex` §2.2)

```
K̇_ij = η [ R₀(K_ij) · cos(Δθ_ij)  −  2 λ K_ij ]  +  η · I_phase · S_ij
         └── Shannon channel ──────────────────┘   └── Fiedler channel ──┘
```

where:
- `R₀(K) = I₁(K)/I₀(K)` is the von-Mises / XY-model order parameter (Bessel ratio).
- `Δθ_ij = θ_j − θ_i` is the phase difference across bond (i,j).
- `λ = 1/r`, with `r` an information-theoretic signal-to-noise ratio.
- `S_ij` is a **Fiedler sensitivity**, i.e. a function of (v₂_i − v₂_j) where v₂ is the Fiedler
  vector (second-smallest eigenvector of the weighted graph Laplacian). The channel is
  budget-conserving: Σ_ij S_ij = 0. This is verified numerically in `scripts/fiedler_proof.py`.
- `I_phase` is a global phase-alignment scalar.

This is an XY-model on the diamond lattice with Kuramoto phase dynamics (`scripts/clr_bkt_convergence.py:95-117`)
and a bond-coupling evolution equation inspired by graph-Laplacian bottleneck detection.

### 1.4 What the code actually does

Seven "core verification" scripts (from `AGENTS.md`):

| Script | Lines | Verifies |
|---|---|---|
| `alpha_137_verification.py` | 158 | Evaluates closed-form α = R₀(2/π)⁴·(π/4)^(1/√e + α/2π), fixed-point iteration. |
| `g_factor_from_lattice.py` | 508 | Plugs that α into standard QED series, prints gap vs Fan 2023 measurement. |
| `living_vs_static_alpha.py` | 208 | Shows that endpoint evaluation (1/√e) gives 137, path integral gives 143. |
| `diamond_greens_function.py` | 460 | Numerical check that G_diff(0,0)−G_diff(0,nn) = 1/z for diamond. |
| `two_vertex_lce.py` | 46 KB | Two-vertex linked-cluster expansion, vacuum-polarization correction. |
| `clr_bkt_convergence.py` | 282 | Runs the CLR on a 16×16 2D square lattice (self-contained). |
| `diamond_clr_convergence.py` | 230 | Runs CLR on 3D diamond — **depends on external `lt183b` module** (lines 38-50). |

The remaining ~20 scripts are experimental companions: `koide_*`, `skyrmion_*`, `d9_*`,
`casimir_mass_spectrum.py`, `electron_mass_from_lattice.py`, plus RG-flow and plotting utilities.
`AGENTS.md` lines 143-152 flags the frame-sector scripts as "exploratory, not in paper."

### 1.5 Reproducibility caveats (verified)

* `alpha_137_verification.py`, `g_factor_from_lattice.py`, `living_vs_static_alpha.py`,
  `clr_bkt_convergence.py`, `diamond_greens_function.py`, `two_vertex_lce.py` are self-contained
  and should run on any machine with NumPy + SciPy + matplotlib.
* `diamond_clr_convergence.py` (line 42) prints `"ERROR: Cannot find infrastructure module at …"`
  and `sys.exit(1)` if the private `184b_lt183_vortex_ring_g2_convergence.py` is not present
  at `../experiments/lattice_theory/canon/v4_2026-02-14/`. This path is **not** in the public
  repo — it refers to an unpublished parent workspace on the author's machine.
* `fiedler_proof.py` has the same dependency (lines 27-45).
* No CI, no tests (`pytest`, `unittest`), no `conftest.py`. "Reproducibility" means "run the script
  and inspect stdout."

### 1.6 Claim-vs-code audit

| README claim | Verified in code? | Note |
|---|---|---|
| Self-consistent α = R₀(2/π)⁴ · (π/4)^(1/√e + α/2π) | ✅ | `alpha_137_verification.py:59-63` |
| Gap to CODATA: 1.5 ppb | ⚠️ Partial | `alpha_137_verification.py` reports ~29 ppm gap. README "1.5 ppb" requires the two-vertex LCE VP correction in `two_vertex_lce.py`. The author flags in `alpha_137_verification.py:150-157` that Steps 2 & 6 are *conjectures* (gap 0.2% and 0.012%). |
| g = 11.4 digits | ⚠️ Misleading framing | Precision is a QED-series amplification of the α input; not an independent prediction. The script's "HONEST ASSESSMENT" section acknowledges this. |
| "Static lattice gives 1/α = 143" | ✅ | `living_vs_static_alpha.py:91-98`. |
| "Couplings self-select, binary alive/dead field" | ⚠️ Partial | Observed on 2D 16×16 (`clr_bkt_convergence.py`), claimed on 3D diamond but 3D verification script needs external module. |
| "Electron is a vortex, spin-1/2 emerges" | ❌ Not closed | Paper §4 argues spin-1/2 from SO(3) frame-sector holonomy. `AGENTS.md:98-104` defers frame sector to "companion paper." Not derived in this repo. |
| "K_bulk → 16/π²" | ⚠️ Partial | `AGENTS.md:205-206` states current data brackets 16/π² = 1.621 between L=6 (1.590) and L=8 (1.638), and flags "GPU runs at L=16, 24, 48 would nail this." |
| Zero free parameters | ✅ with caveats | The closed-form formula has no free constants, but several identities (σ² = 1/2, cos_eff = 2/3, n = 1/√e) are conjectures at finite L. |

The author is substantially more honest in `AGENTS.md` and in the script outputs than the README
front page suggests. Section 5.11 of the paper ("Proof Status Assessment", line 1061) reportedly
labels individual derivation steps as proven / conjectured / empirical.

### 1.7 Bibliography / theoretical lineage cited by the paper

From `references.bib` (28 entries, all verified):

* **BKT transition**: Berezinskii 1971; Kosterlitz & Thouless 1973, 1974; Nelson-Kosterlitz 1977.
* **QED & α**: Schwinger 1948; CODATA 2016; Hanneke 2008; Parker 2018; Morel 2020; Fan 2023.
* **Solitons/topology**: Skyrme 1962; Wilson 1974; Polyakov 1987; Haldane 2017; Manton-Sutcliffe 2004.
* **Kuramoto**: Kuramoto 1984.
* **Numerological α attempts (noted as prior art, not endorsed)**: Wyler 1969; Eddington 1929;
  Williamson-van der Mark 1997.
* **LHAASO 2022** (for UV consistency argument).

This is a respectable bibliography. The paper is situated in the BKT / XY-model / topological-soliton
tradition, not in the categorical / sheaf-theoretic / CRDT tradition that the name "coherence
lattice" might suggest to a software reader.

---

## 2. Theoretical lineage

The phrase "coherence lattice" is heavily overloaded across disciplines. It is important to say
clearly where this paper sits:

### 2.1 What this repo is NOT

The name sounds like it could be any of the following, but is **none** of them:

* **Lattice theory (order theory)**: partial orders with join (∨) and meet (∧) operations,
  join-semilattices, CRDTs based on monotonic join. Not this — no order-theoretic lattices appear.
* **Girard's coherence spaces (linear logic)**: reflexive symmetric relations modelling types in
  linear logic. Not this — neither cited nor used.
* **Sheaf cohomology / Čech cohomology on a simplicial complex**: checking local→global
  consistency (cf. Robert Ghrist, Michael Robinson). Not this.
* **Causal sets / causal dynamical triangulation (Sorkin et al.)**: discrete Lorentzian manifolds.
  Not this, despite the name — the diamond lattice here is a Euclidean crystal lattice, not a
  discrete causal structure.
* **Quantum coherence / decoherence**: the paper explicitly avoids the quantum axioms and argues
  an *emergent*-QED story. α is derived, but Hilbert-space coherence is not the subject.
* **Topos theory / categorical logic**: not cited.

### 2.2 What this repo IS

* **Statistical mechanics of 2D/3D XY models with dynamical couplings.** The base object is a
  classical oscillator at each lattice site with phase θ_i ∈ S¹, with bond couplings K_ij that are
  themselves variables evolving under a gradient flow.
* **Graph-Laplacian spectral methods applied to physics simulations.** The Fiedler channel is
  standard spectral-graph theory (λ₂ of the weighted Laplacian) injected as a forcing term.
* **BKT universality, topological defect dynamics.** Vortices on a 3D diamond lattice, vortex-line
  persistence, Kosterlitz–Thouless critical coupling K_BKT = 2/π.
* **A numerological/fundamental-physics program** in the spirit of Wyler (1969) and Eddington
  (1929), trying to extract α from pure mathematics + lattice topology. The novelty — which the
  paper argues for convincingly in its own terms — is that the "lattice" is **self-tuning** rather
  than hand-tuned, so α emerges at a constrained optimum rather than being an input.

### 2.3 Genuine novel claims

Setting aside the physics question, the computational novelty (as a pattern) is:

1. **"Living lattice"**: bond weights are state, not hyperparameters. Their update law is a
   gradient descent on a coherence-capital potential.
2. **Two-channel coupling update**: Shannon channel (local per-bond) + Fiedler channel (global,
   budget-conserving). This is a reasonable general recipe for graph-learning systems that want
   to spread coupling toward structural bottlenecks without a global mass budget violation.
3. **"Endpoint vs path-integral" distinction**: when a dynamical variable converges to a fixed
   point, evaluating effective exponents at the attractor gives different answers than integrating
   over the RG trajectory. This is a valid mathematical point even outside physics.

None of these is a new abstract-data-structure primitive. Points (1) and (2) together are the
single transferable idea a software system could steal.

---

## 3. Comparison to WeftOS substrate

Verified against the current `/claw/root/weavelogic/projects/clawft/crates/clawft-kernel/src/` tree
(see `Cargo.lock` / `src/lib.rs`).

### 3.1 `eml_coherence.rs` (884 lines)

**Existing semantics of "coherence" in WeftOS**:
`EmlCoherenceModel::predict(&GraphFeatures)` returns a `CoherencePrediction { lambda_2,
fiedler_norm, uncertainty }` (lines 38-47). "Coherence" here means **the algebraic connectivity
λ₂ of the CausalGraph Laplacian**, predicted in O(1) from graph-statistics features by the EML
learned-function pipeline. The ground-truth path is classical Lanczos (O(k·m)). The whole module
is a two-tier accelerator for the DEMOCRITUS tick loop: fast EML on every tick, exact Lanczos
when drift > threshold, retrain every 1000 exact samples (lines 6-16).

**Overlap with coherence-lattice-alpha**: Both compute / use λ₂ of a graph Laplacian. That's
where the overlap ends. WeftOS uses λ₂ as a read-only metric of how tightly coupled the causal
graph is. The paper uses λ₂ as an active forcing term whose gradient across each bond drives the
coupling update. **They are opposite directions on the same spectral object** — WeftOS predicts λ₂,
the CLR updates edges *using* λ₂.

**Verdict**: Terminology collision only. No functional overlap. We do not currently do anything
with v₂ itself (the eigenvector); only with the eigenvalue λ₂ and an estimated Fiedler-norm
scalar (line 43). If we ever want a per-edge structural signal, the Fiedler-sensitivity
`(v₂_i − v₂_j)²` pattern from the CLR is a clean formula we could borrow (attribution: Sharpe 2026,
but also standard in spectral-graph-bottleneck literature going back to Fiedler 1973).

### 3.2 `causal.rs` (3,417 lines)

A concurrent lock-free DAG with typed edges (`Causes`, `Inhibits`, `Correlates`, `Enables`,
`Follows`, `Contradicts`, `TriggeredBy`, `EvidenceFor`). Built on `DashMap`. Semantics are
discrete, symbolic, provenance-carrying. The nodes are typed events; edges carry a weight ∈ [0,1]
and a provenance string.

**Overlap with coherence-lattice-alpha**: None at the data-model level. The CLR's diamond lattice
is a fixed regular 3D grid with z = 4 bonds/site, all bonds isomorphic, no edge types, no
provenance. The dynamical variable is continuous (θ ∈ S¹, K ∈ ℝ≥0), not discrete.

**Verdict**: Incompatible data models. CausalGraph is a symbolic reasoning substrate;
coherence-lattice-alpha is a physical-dynamics substrate. Neither could replace the other without
losing its entire purpose.

### 3.3 `quantum_state.rs` (974 lines) + `quantum_backend.rs` (160 lines)

`QuantumCognitiveState` (line 92) carries a complex amplitude per graph node,
`psi: Vec<Complex>`. It supports: uniform-superposition init (line 122), **Fiedler-vector init**
(`from_fiedler`, line 108), first-order unitary evolution `|ψ'⟩ ≈ (I − i·dt·H)|ψ⟩` with
H = graph Laplacian, Born-rule collapse, decoherence tracking via entropy history (line 99).
The `quantum-pasqal` feature connects this to real Rydberg-atom hardware via the
`QuantumBackend` trait (see `docs/src/content/docs/weftos/quantum.mdx`).

**Overlap with coherence-lattice-alpha**:
* Superficial: both repos use "coherence" + "phase" + "lattice".
* Mathematical: the WeftOS quantum Hamiltonian is the graph Laplacian L of the *CausalGraph*; the
  CLR's Hamiltonian (implicit in Kuramoto dynamics) is J · ∇²_graph for a *regular 3D crystal
  lattice*. Both are Laplacians of weighted graphs, but the graph topologies and intended
  semantics differ completely.
* Physical: the CLR paper is **explicitly classical**. θ_i is a classical angle, not a qubit.
  The "phase" in the CLR is not quantum phase. The two layers are not talking about the same
  coherence even when they use the same word.

**Verdict**: The only genuine technical connection would be if we mapped a CLR-style diamond
oscillator into a Pasqal neutral-atom register — which would in principle be possible (atoms in
diamond geometry, Rydberg blockade standing in for K-couplings). That is a research project on
top of Pasqal, not an integration with coherence-lattice-alpha specifically.

### 3.4 `eml_kernel.rs` (828 lines)

The generic EML learned-function infrastructure (Odrzywolel 2026 operator). Uses
`eml(x, y) = exp(x) - ln(y)` trees to replace hardcoded thresholds with learned functions.
`eml_coherence.rs` is one of many domain wrappers (see `docs/src/content/docs/weftos/eml.mdx`).

**Overlap with coherence-lattice-alpha**: None. Different problem class entirely (learned
heuristics vs physics simulation).

### 3.5 `cognitive_tick.rs` (743 lines), `crossref.rs` (426 lines), `impulse.rs` (319 lines), `mesh_*.rs`

No overlap. These are orthogonal infrastructure: adaptive heartbeat, typed cross-structure links,
impulse queue, distributed gossip. Nothing in the CLR paper touches any of these concerns.

### 3.6 Summary of comparison

| WeftOS module | Overlap with CLR repo? | Nature of overlap |
|---|---|---|
| `eml_coherence.rs` | **Terminological + one primitive** | Both use λ₂; CLR also uses v₂ per-bond (Fiedler sensitivity) |
| `causal.rs` | None | Symbolic DAG vs continuous regular lattice |
| `quantum_state.rs` | Terminological only | Both have "phase"; CLR is classical |
| `quantum_backend.rs` / Pasqal | Possible research axis | Diamond neutral-atom register is physically realisable |
| `eml_kernel.rs` | None | |
| `cognitive_tick.rs`, `crossref.rs`, `impulse.rs`, `mesh_*.rs` | None | |

---

## 4. Comparison to ECC / EML / quantum layer

### 4.1 Does CLR replace a subsystem?

No. The CLR's "lattice" is a *fixed 3D crystal* in which every bond is of the same physical kind,
where dynamics are continuous-time ODEs on θ and K. The ECC "forest of trees" (Symposium D2) is a
heterogeneous collection of symbolic structures (ExoChain, HNSW, Resource Tree, CausalGraph) linked
via CrossRefs. These are incompatible abstractions.

### 4.2 Could it sit alongside as a new structure in the forest?

Only in a very narrow sense: one could imagine a `PhysicsLattice` structure that is, effectively,
an oscillator-network simulator for modelling client-system dynamics (e.g. service-mesh traffic as
Kuramoto oscillators, coupling as dynamical K). This would be (a) entirely orthogonal to the
α-derivation purpose of the paper, and (b) redundant with graph-spectral features we already
compute via `eml_coherence`.

### 4.3 Could it be a meta-framework unifying existing structures?

No. The paper's content is specific to fundamental-physics derivation; it does not abstract the
CLR as a general graph-learning framework, and abstracting it that way would not add anything
WeftOS doesn't already do with EML + CausalGraph + HNSW.

### 4.4 Overlap with existing `eml_coherence` module

The one substantive technical idea we could cleanly factor out is the **two-channel edge update
with a budget-conserving Fiedler term**:

```
K̇_ij = α · local_signal(i,j) + β · (v₂_i − v₂_j)²   with   Σ_ij (v₂_i − v₂_j)² = normalised
```

This is a clean pattern for future work on the CausalGraph edge-weight update path (today, weights
are set by provenance heuristics, not learned). It does NOT require taking anything from the
coherence-lattice-alpha repo; the Fiedler channel is standard spectral graph theory (Fiedler 1973,
Chung 1997). Attribution to Sharpe 2026 for the *particular* CLR formulation is fair, but there is
no code or data we need from the repo to implement the pattern.

### 4.5 Overlap with quantum layer's superposition/coherence concepts

The quantum layer's `QuantumCognitiveState::from_fiedler` (line 108 of `quantum_state.rs`) already
uses the Fiedler vector as an initial state. This is conceptually aligned with what the CLR is
doing (Fiedler guides both), but it is a completely independent, pre-existing design choice in
WeftOS. No new connection is added by integrating the CLR.

---

## 5. Integration options

### Option A — Conceptual-only influence (terminology / prior-art citation)

**Effort**: 0–1 engineer-hours.
**Risk**: Essentially none.
**Changes**:
- Add a note in `eml_coherence.rs` module doc comment (lines 1-24) disambiguating "coherence" from
  the Sharpe 2026 CLR sense, to forestall confusion for readers of both.
- Add a paragraph to `docs/src/content/docs/weftos/ecc.mdx` under the CausalGraph section clarifying
  that WeftOS "coherence" = algebraic connectivity λ₂, not the Sharpe physics sense.
- (Optional) Record the Fiedler-drain two-channel pattern in a `.planning/development_notes/`
  note as potentially relevant to a future "learned causal edge weights" sprint.

**Licensing**: Zero contact with AGPL-3.0 code. No issue.

### Option B — Surgical integration (Fiedler-sensitivity edge update)

**Effort**: ~1 sprint (5–10 engineer-days).
**Risk**: Low-moderate. This is pure spectral graph theory; the only "integration" with
coherence-lattice-alpha would be a citation. We would reimplement the pattern from scratch in Rust.
**Changes**:
- Add a new `CausalGraph::fiedler_sensitivity(edge_id) -> f64` method backed by the existing
  Lanczos path (`causal.rs` already has the machinery; `eml_coherence.rs` already uses it).
- Add a per-edge weight-update learned function in the EML pipeline that takes Fiedler sensitivity
  as one of its inputs.
- New tests mirroring the ≥10 existing `EmlCoherenceModel` tests.
- Governed by a new ADR (candidate: **ADR-048 Learned Causal Edge-Weight Updates via Fiedler-Channel EML**).

**Licensing**: AGPL-3.0 not triggered — we write the Rust from scratch, citing the paper as prior
art. CC BY-NC 4.0 of paper text does not affect us as long as we don't redistribute the paper.

**Caveat**: This would be worthwhile *regardless* of coherence-lattice-alpha. The paper is a
stimulus to look at the pattern, not a dependency.

### Option C — Deep integration (coherence-lattice-alpha as dependency / ported library)

**Effort**: Prohibitive. Rough estimate: 3–6 sprints to port ~28 Python scripts to Rust, plus the
external `lt183b` module we don't have.
**Risk**: High on all axes.
**Changes**:
- Add a new crate `clawft-physics-lattice` re-implementing CLR, Kuramoto, BKT-vortex detection,
  linked-cluster expansion, etc.
- Find a concrete WeftOS use-case for an oscillator simulator. (There isn't one today.)
- Integrate with `cognitive_tick` loop.

**Licensing**: AGPL-3.0 is the blocker. If we depended on any of the scripts (rather than
reimplementing from the paper), the AGPL-3.0 would infect WeftOS. Reimplementation from the paper
is cleaner but still requires care because the CC BY-NC 4.0 on the paper text means we'd need
"NonCommercial" compliance if we reproduced figures or equations verbatim — we can restate in our
own words, but this adds friction.

**Verdict**: **Strongly unrecommended.** The user-facing value of being able to derive α from a
simulation has no business overlap with WeftOS's GTM (selling WeftOS to understand client systems
via knowledge graphs, per MEMORY.md).

---

## 6. Licensing and dependency analysis

### 6.1 The two licenses

From `LICENSE`:
- **Paper text + figures** (`paper.tex`, `figures/*`): **CC BY-NC 4.0** (Creative Commons,
  Attribution-NonCommercial 4.0 International).
- **Scripts** (`scripts/*`): **AGPL-3.0** (GNU Affero General Public License v3).

Copyright: Michael Sharpe, 2026.

### 6.2 What each license implies for WeftOS

| License | Can we read it? | Can we reimplement from it? | Can we link against it? | Can we redistribute? |
|---|---|---|---|---|
| CC BY-NC 4.0 (paper) | Yes | Yes (reimplementation in our own words is not a CC-covered derivative of the text) | N/A (not code) | Only noncommercial |
| AGPL-3.0 (scripts) | Yes | Yes, but see next column | **Only if all of WeftOS becomes AGPL-3.0** — AGPL infects linked works, *including* over-network use | Must relicense under AGPL-3.0 |

**WeftOS's current licensing posture** (verify with `/claw/root/weavelogic/projects/clawft/LICENSE`
if this review is followed up on): WeftOS crates have been released under Apache-2.0 / MIT in
recent commits. Adopting AGPL-3.0 code would be a **material licensing change** requiring explicit
user direction. It would also bind any commercial WeftOS deployment to AGPL's network-interaction
disclosure clause.

### 6.3 Dependency tree of the repo itself

From README line 60 and `AGENTS.md` line 108: the entire Python dependency is
`numpy`, `scipy`, `matplotlib`. No other pip packages. No Rust dependencies. No JavaScript.
The LaTeX stack uses revtex4-2, pdflatex, bibtex — standard.

The *undisclosed* dependency is the local-only `experiments/lattice_theory/canon/v4_2026-02-14/`
path referenced by `diamond_clr_convergence.py:38-50` and `fiedler_proof.py:35-37`. This is the
author's private research workspace; scripts that depend on it will not run from a fresh clone.

### 6.4 Attribution mechanics

If we adopt Option A or B: cite as

```
Sharpe, M. (2026). "The Coherence Learning Rule on the Diamond Lattice: From Coupled
Oscillators to the Fine Structure Constant." Preprint. https://github.com/project-89/coherence-lattice-alpha
```

and note we are borrowing the *Fiedler-channel pattern*, not the *physics claims*.

---

## 7. Recommendation

**DEFER.**

**Primary reasoning**:

1. **Category mismatch.** coherence-lattice-alpha is a physics preprint, not a software library.
   There is nothing in it to "integrate" in the usual sense — no API, no crate, no versioned
   release, no tests in the engineering sense.
2. **License incompatibility.** AGPL-3.0 on the code makes any direct reuse a dealbreaker given
   WeftOS's current permissive licensing posture. Reimplementation from the paper is possible but
   not necessary — the transferable idea (Fiedler-channel edge update) is standard spectral graph
   theory well predating this paper.
3. **Extraordinary claims.** Deriving α from first principles is a Nobel-level claim if correct.
   Five-commit preprint from a single author with no peer review, no arXiv number yet (no
   arxiv.org ID in `references.bib` for this work), and several internal "conjecture" labels in
   the code itself — we cannot rely on it for anything in a production substrate. Even if the
   physics is right, WeftOS's job is not to assume that.
4. **No business alignment.** Per `MEMORY.md`: top priority is selling WeftOS to understand client
   systems via knowledge graph (GTM). Fundamental-physics derivation does not serve that goal.
   ECC for robotics (DEMOCRITUS) is a separate priority but also not helped by the CLR.
5. **One transferable idea, not worth a sprint.** The Fiedler-sensitivity two-channel edge update
   is a genuinely nice pattern, but (a) it's not original to this paper, (b) we can implement it
   standalone when/if we need it, and (c) we have higher-priority work (Sprint 17 KG tasks per
   SESSION_HANDOFF).

**What to do instead**:

- **Now**: log this evaluation (this file) so we have a written record if this repo resurfaces.
- **Short-term (this sprint, optional)**: add 1–3 sentences in `eml_coherence.rs` module doc
  clarifying "coherence" disambiguation, to help future contributors who may confuse WeftOS
  coherence with Sharpe 2026 coherence.
- **Medium-term (whenever learned causal-edge-weights comes onto the roadmap)**: when we next
  touch `CausalGraph` edge-weight updates, consider the Fiedler-sensitivity pattern as one input
  to a learned edge-update EML model. Draft ADR-048 at that time. No dependency on this repo is
  required.
- **Do not**: port scripts, depend on AGPL code, claim α-derivation capabilities, or cite the
  paper as an authority. If a client asks about it, describe it honestly as an interesting but
  unreviewed physics preprint.

**Which ADR would govern it (if we ever did surgical integration)**: ADR-048 (to be created),
"Learned Causal Edge-Weight Updates via Fiedler-Channel EML", building on ADR-032-ish-era ECC
foundations and ADR-026 three-tier routing.

**Which sprint**: Not Sprint 17. Possibly Sprint 19+ as part of causal-graph-quality work, and only
if an actual product need for learned edge weights emerges.

---

## Appendix A — Glossary (as used in coherence-lattice-alpha)

| Term | Definition (from paper / AGENTS.md) |
|---|---|
| **CLR** | Coherence Learning Rule — the equation of motion for bond couplings K_ij. |
| **Coherence capital** | Scalar maximised by the CLR: product of phase alignment and structural richness. |
| **Living lattice** | Lattice where bond couplings are dynamical variables, not fixed parameters. |
| **Static lattice** | Standard lattice field theory: K is a hand-tuned parameter. |
| **BKT** | Berezinskii–Kosterlitz–Thouless. The phase transition in 2D XY models. K_BKT = 2/π. |
| **Diamond lattice** | 3D crystal lattice with 2-site unit cell, coordination number z = 4. |
| **Bravais rank** | Number of independent lattice vectors; filter 1 of 5 for selecting diamond. |
| **R₀(K)** | Bessel ratio I₁(K)/I₀(K). XY-model order parameter. |
| **K_BKT** | 2/π. BKT critical per-bond coupling. |
| **K_bulk** | 16/π² = z·K_BKT². The value alive bonds converge to under the CLR. |
| **Shannon channel** | First CLR term: local per-bond information gradient. |
| **Fiedler channel** | Second CLR term: structural forcing from (v₂_i − v₂_j)² gradient. |
| **Fiedler vector (v₂)** | Eigenvector of graph Laplacian for eigenvalue λ₂. Standard. |
| **Algebraic connectivity (λ₂)** | Second-smallest eigenvalue of Laplacian. "Coherence" in WeftOS. |
| **PLM** | Phase-Locked Mode. Attractor of the CLR where K freezes at a fixed point. |
| **DW factor** | Debye-Waller factor: exp(−σ²) = 1/√e at the PLM. |
| **Vortex** | Topological defect in θ-field, π₁(S¹) = ℤ winding number. |
| **LCE** | Linked-cluster expansion. Graph-theoretic perturbation series. |
| **Frame sector** | SO(3) rotational degrees of freedom. Deferred to companion paper. |
| **Phase sector** | U(1) phase θ. Subject of this paper. |
| **K-scaling invariance** | H²_A ∝ K²: bipartite factorisation identity. |
| **Simplex projection** | cos_eff = (d−1)/d = 2/3 identity at d = 3. |

## Appendix B — Open questions for the maintainers

If contact is ever made with Michael Sharpe, these are the specific questions this evaluation
surfaced:

1. The path `experiments/lattice_theory/canon/v4_2026-02-14/184b_lt183_vortex_ring_g2_convergence.py`
   is referenced by `scripts/diamond_clr_convergence.py` and `scripts/fiedler_proof.py` but is not
   in the public repo. Is there a plan to open-source it, or to inline its `make_d_diamond_adjacency`,
   `make_simplex_deltas`, `_build_graph_laplacian`, and `sparse_laplacian_and_fiedler` functions?
   Without those, the key 3D-diamond convergence claims cannot be reproduced from the repo alone.

2. `AGENTS.md` lines 205-206 notes current K_bulk data is L=6 (1.590), L=8 (1.638), bracketing
   16/π² = 1.621. Are there plans to run L=16, 24, 48 (mentioned as "GPU runs would nail this")?
   The central empirical claim depends on this.

3. The "11.4 digits of g" framing in the README does not mirror the honest framing in
   `g_factor_from_lattice.py:486-490` (where the author notes 100% of the gap comes from the α
   gap and QED coefficients are standard). Would the authors consider rewording the README to
   match the script's own candour?

4. Is the AGPL-3.0 license on the scripts a deliberate choice, or would an MIT/Apache-2.0 dual
   licence be considered? (Context for WeftOS: AGPL-3.0 is incompatible with our permissive
   distribution, which means otherwise-benign cross-pollination is blocked.)

5. Has the paper been submitted to arXiv? References.bib has no arXiv ID for this work.

6. The paper repeatedly distinguishes "proven" from "conjecture" (per §5.11). Is there a public
   tracker (issues / milestones) for converting specific conjectures into proofs?

## Appendix C — Second-degree references worth following

From `references.bib` and `AGENTS.md`, entries relevant to WeftOS irrespective of the α claim:

- **Fiedler (1973)**, "Algebraic connectivity of graphs." *Czechoslovak Mathematical Journal*
  23 (98), 298–305. Not in this repo's bibliography, but the foundational paper for the whole
  Fiedler-channel idea. Directly relevant to `eml_coherence.rs`.
- **Kosterlitz & Thouless (1973)**. The BKT mechanism. Relevant as background if we ever model
  phase-transition-like behaviour in the ECC substrate.
- **Haldane (2017), Nobel Lecture**, *Rev. Mod. Phys.* 89, 040502. Topological quantum matter.
  Interesting for the quantum layer's conceptual framing.
- **Odrzywolel (2026)** — cited in WeftOS's own `eml-core` crate (not this repo). The EML operator
  that underlies `eml_coherence.rs`. Orthogonal to coherence-lattice-alpha but worth re-noting:
  the EML operator already gives us a principled way to learn "coherence" functions without a
  physics lattice.
- **Parker 2018 / Morel 2020 / Fan 2023** — current-generation precision α and g measurements.
  Relevant only if WeftOS ever grows a scientific-computing tier that cares about CODATA values.

---

**End of document.**
*Evaluated under the general-purpose WeftOS integration frame. If a sonobuoy-specific follow-up
is ever needed, this evaluation does not cover that angle; the result would be the same
(defer / reject), but the reasoning would change.*
