# Byzantine-Robust Federated Learning — Machine Learning with Adversaries (Krum)

## Citation

- **Title**: Machine Learning with Adversaries: Byzantine Tolerant Gradient Descent
- **Authors**: Peva Blanchard, El Mahdi El Mhamdi, Rachid Guerraoui, Julien Stainer
- **Affiliation**: EPFL (École Polytechnique Fédérale de Lausanne)
- **Venue**: Advances in Neural Information Processing Systems 30 (NeurIPS/NIPS 2017), pp. 119–129
- **arXiv**: [1703.02757](https://arxiv.org/abs/1703.02757) (preprint title: "Byzantine-Tolerant Machine Learning")
- **Proceedings**: https://proceedings.neurips.cc/paper/2017/hash/f4b9ec30ad9f68f89b29639786cb62ef-Abstract.html
- **PDF**: `.planning/sonobuoy/papers/pdfs/byzantine-robust-krum.pdf`

## Status

**verified**. Author list, NeurIPS 2017 venue, page range (119–129), and arXiv preprint ID
1703.02757 all corroborated against arXiv, NeurIPS proceedings, and DBLP. The preprint and
proceedings titles differ slightly ("Byzantine-Tolerant Machine Learning" vs "Machine Learning
with Adversaries: Byzantine Tolerant Gradient Descent") — both refer to the same paper. This is
the canonical Krum paper.

## One-paragraph Summary

Blanchard, El Mhamdi, Guerraoui, and Stainer at EPFL showed — and this is the paper's central
result — that **every linear aggregation rule** (weighted mean, FedAvg, ordinary averaging, any
convex combination of worker gradients) can be driven to an arbitrary output by a **single
Byzantine worker**. In other words, plain FedAvg has *zero* Byzantine tolerance: one corrupt
sonobuoy uploading a giant weight delta pulls the global model anywhere it wants. The authors
formalize an **(α, f)-Byzantine resilience** property an aggregator must satisfy to withstand
f arbitrary failures out of n workers, and introduce **Krum** — a selection rule that picks the
worker whose reported gradient has the smallest sum-of-distances to its (n−f−2) nearest
neighbors. Krum is proven Byzantine-resilient for any attack as long as **2f + 2 < n**, i.e. a
strict majority of honest workers. They also give **Multi-Krum**, a straightforward generalization
that averages the m lowest-scoring workers for better variance at the same resilience guarantee.
Experimentally on MNIST, unprotected averaging under Byzantine attack collapses to near-chance
error while Krum holds the protected model at baseline accuracy. For sonobuoys — where a captured
or spoofed buoy is a realistic threat — **plain FedAvg aggregation is not an option**; Krum or a
descendant is mandatory.

## Methodology

### Threat model

- n workers submit gradients `v_1, ..., v_n ∈ ℝ^d` per step.
- Up to **f of them are Byzantine**: can collude, see the other workers' submissions
  (omniscient attacker), and submit any vector they like — not just noise, but adversarially
  crafted payloads.
- Aggregator is trusted.
- Honest workers sample from the same distribution with bounded variance; their true gradient
  is `g = ∇F(θ)` at the current parameters.

### Why linear aggregators fail

Paper's Proposition 1: For *any* aggregator of the form `A(v_1,...,v_n) = Σ α_i v_i` with α_i
fixed (including the mean, any weighted mean, and any Huber-like weighted combination), a single
Byzantine worker can force A to equal any desired vector by setting `v_j = (target − Σ_{i≠j} α_i v_i) / α_j`.

In FedAvg terms: one corrupt buoy that lies about its n_k or its `w_k` can overwrite the global
update. This is not a probabilistic "bad for large f" result — it's a **deterministic "bad for
f = 1"** result. Averaging is fundamentally indefensible.

### (α, f)-Byzantine resilience (Definition 1)

An aggregator F is (α, f)-Byzantine-resilient with respect to the correct gradient g if, for
any f-subset of Byzantine submissions B,

    ⟨E[F(V)], g⟩ ≥ (1 − sin α) · ‖g‖²

    and  E[‖F(V)‖^r] ≤ C(r) · (bounded-variance terms)   for some r ≥ 2.

In plain English: the aggregator's output, in expectation, **points in roughly the same direction
as the true gradient**, and its moments are bounded. Convergence of projected SGD then follows
from standard arguments (Bottou 1998 / Bertsekas-Tsitsiklis): if every step is a biased but
positively-correlated estimate of g, θ converges to a stationary point.

### The Krum rule

Given worker submissions v_1, ..., v_n and Byzantine count f:

```
for each i:
    let S_i = set of (n − f − 2) nearest vectors to v_i (by Euclidean distance), excluding v_i
    score(i) = Σ_{j ∈ S_i} ‖v_i − v_j‖²
Krum(v_1,...,v_n) = v_{i*}   where i* = argmin_i score(i)
```

Informally: choose the submission that sits in the densest region of the honest-worker cluster.
Byzantine submissions far from the honest cluster get large scores and are rejected; Byzantine
submissions *inside* the honest cluster are close to honest gradients anyway and so do little
harm.

### Condition: 2f + 2 < n (equivalently f < (n − 2) / 2)

Required so that the (n−f−2) nearest-neighbor set for any honest worker contains at least
(n − 2f − 2) other honest workers — a majority. With f ≥ ⌊(n−2)/2⌋ Byzantine workers, Krum's
guarantee fails because Byzantine submissions can form their own dense cluster.

For **n = 10 sonobuoys**, Krum tolerates up to **f = 3** corrupt buoys.
For **n = 20**, tolerates **f = 8**.
For **n = 5**, tolerates **f = 1**.
*Never tolerates 50%+ malicious — that would require multiple parallel aggregations with
cross-witnesses (see Byzantine fault tolerance literature).*

### Multi-Krum variant

Instead of picking just the argmin, **average the m workers with lowest Krum scores**:

    MultiKrum(V) = (1/m) Σ_{i ∈ bottom-m-by-score} v_i

With m = n − f, Multi-Krum has much lower variance than single-select Krum (it averages roughly
the honest submissions) while preserving the same Byzantine-resilience guarantee. This is what
a production system actually uses.

### Complexity

- Pairwise distances: O(n² · d) compute and O(n²) distance-matrix storage.
- For d = 5·10^6 params and n = 20 sonobuoys, that's 20·19/2 = 190 pairwise L2-norms, each
  over 5 M floats. ~1 GFLOP — trivial on the aircraft/ship leader, impossible on a single buoy.
- Multi-Krum adds O(m · d) for the final mean.

### Related robust aggregators (mentioned in paper, important follow-ups)

- **Geometric median (Pillutla et al.'s RFA, 2019)** — iterative Weiszfeld solver; O(T·n·d).
- **Coordinate-wise median / trimmed mean (Yin et al. 2018)** — O(n·d·log n); sometimes tighter
  than Krum when f is small.
- **Bulyan (El Mhamdi et al. 2018)** — composition of Krum + trimmed mean; resilient against
  *more subtle* attacks (e.g. "a little is enough") that vanilla Krum misses.

## Key Results

### MNIST multinomial logistic regression, n = 20 workers

The paper's empirical section uses SGD on MNIST with n = 20 simulated workers, varying f (the
number of Byzantine workers) and the attack strategy.

- **No attack, f = 0**: Krum, averaging, and Multi-Krum all converge to ~92% accuracy (the
  baseline for the chosen simple model).
- **Omniscient attack (Byzantine workers send `−c · Σ_{i honest} v_i`)**, f = 4:
  - **Averaging: diverges** to near-random (~10% accuracy).
  - **Krum: maintains baseline** ~92%.
- **Gaussian attack (Byzantine workers send Gaussian-noise vectors)**, f = 4:
  - Averaging loses ~3–5% accuracy.
  - Krum unaffected.
- **Multi-Krum, f = 4, m = 16**: matches honest-only averaging in variance, same robustness
  as Krum.

### Convergence speed

Krum converges slightly more slowly than plain averaging when f = 0 (you throw away (n−m)
worker signals every step). Multi-Krum at m = n − f recovers most of this; practical overhead
is < 2× rounds in typical settings.

### Cost attribution

The chief empirical takeaway in the sonobuoy context: **under any credible military/adversarial
threat model, replacing averaging with Multi-Krum is essentially free** (small variance
overhead) and provides the only currently-known cheap defense against single-worker sabotage.

## Strengths

- **Formal impossibility for linear aggregators.** The "one Byzantine worker can break any
  averaging rule" theorem is tight; it tells you plainly that defense requires nonlinearity.
- **Simple, deterministic algorithm.** Krum is just pairwise distances + argmin; no iterative
  solver, no gradient step, nothing to tune.
- **Provable guarantee** of positive inner product with the true gradient, from which
  convergence of Byzantine-SGD follows without extra assumptions.
- **Drop-in replacement** for FedAvg's aggregation step. Requires no change to worker code.
- **Multi-Krum is practical.** Sets the template for every later robust-aggregation paper.

## Limitations

- **O(n² d) aggregator cost.** At huge scale (1000+ buoys, 100 M-param model) this becomes
  expensive even on the central ship. Mitigated by hierarchical aggregation (cluster → region
  → global).
- **Requires 2f + 2 < n** — can't handle a majority of bad workers. In practice this means
  sonobuoy fields must have cryptographic attestation (rvf-crypto) to keep the *identities*
  honest, and Krum to keep the *gradients* honest.
- **Vulnerable to "A Little Is Enough" attacks (Baruch et al. 2019).** Byzantine workers that
  submit small-perturbation gradients inside the honest cluster can slowly bias the model; Krum
  does not reject them because their score is not obviously larger. Bulyan fixes this.
- **Assumes IID honest workers.** Under strong non-IID (one buoy sees submarine noise, one sees
  shipping noise), honest gradients naturally differ, and Krum's "pick the densest cluster" may
  discard genuine niche-class signal. Solutions: cluster-aware Krum, or per-class aggregation.
- **Omniscient attacker assumption** is strong. A more realistic model (Byzantine workers
  cannot see each other's submissions) admits cheaper defenses — but if you solve the strong
  version, the weaker one is free.

## Portable Details

### Krum score formula

For worker i, let `NN_i(k)` denote the set of k nearest `v_j` to `v_i` (j ≠ i) by Euclidean
distance. With k = n − f − 2:

    score(i) = Σ_{j ∈ NN_i(n−f−2)} ‖v_i − v_j‖²
    Krum(V)  = v_{arg min_i score(i)}

### Multi-Krum

    MultiKrum_m(V) = (1/m) · Σ_{i ∈ bottom-m-by-score(i)} v_i

### Tolerance condition

    2f + 2 < n       ⟺       f < (n − 2) / 2       ⟺       honest ≥ ⌈(n+3)/2⌉

### Composition with FedAvg

FedAvg's aggregation step `w_{t+1} = Σ (n_k/N) · w_{t+1}^k` is the linear aggregator that Krum
replaces. The weighted-mean replacement becomes:

    v_k = (n_k / N_mean) · Δw_k          # normalized delta
    g_selected = MultiKrum(v_1, ..., v_n)
    w_{t+1} = w_t + g_selected

The `n_k` weighting inside the pre-Krum normalization is optional; if buoys lie about `n_k`,
drop it and fall back to unweighted deltas.

### Composition with DGC

Krum operates on the **dense reconstructed** Δw_k, not on the sparse Top-k payload. Reason: a
Byzantine worker can concentrate its attack in a single Top-k coordinate that happens to align
with honest workers' Top-k, appearing "close" in sparse-support similarity but far in true
L2 distance. Always reconstruct first, then apply Krum.

### Hierarchical Krum for scale

For >100 buoys, partition into `c` subclusters (e.g. by acoustic neighborhood). Each subcluster
aggregates internally with Multi-Krum, producing a "sub-leader's" Δw. The ship/aircraft then
runs Multi-Krum on the c sub-leader outputs. Cost drops from O(n²) to O(c·(n/c)²) = O(n²/c).

## Sonobuoy Integration Plan

### Threat model for sonobuoys

A sonobuoy is a physical object in a physical ocean. Credible adversaries include:

- **Capture-and-replay**: adversary recovers a buoy, extracts keys, produces spoofed messages.
- **Radio spoofing**: fake buoy ID, fake gradient payloads.
- **Compromised fleet mate**: a buoy with software defect or supply-chain implant.
- **Byzantine by accident**: dying battery, corrupted flash, sensor drift — produces
  pathological gradients without malice.

The system must survive all of these. FedAvg's plain averaging does not. Krum/Multi-Krum does
(up to f < n/2).

### Where Krum slots into the WeftOS mesh

- **rvf-crypto** already provides node identity and payload signing. This keeps *who claims to
  have sent what* honest but says nothing about *whether the signed content is good*.
- **Krum is the gradient-plane complement to rvf-crypto's identity plane.** rvf-crypto rejects
  unsigned messages; Krum rejects signed-but-malicious gradients.
- **Aggregation runs on the Raft leader** (`chain.rs` + `mesh_chain.rs`). The leader:
  1. Collects signed, DGC-compressed ΔW from m participating buoys over one radio window.
  2. Verifies signatures (rvf-crypto).
  3. Reconstructs dense ΔW (DGC inverse).
  4. Runs Multi-Krum with m = n − f_expected.
  5. Commits the aggregated Δw to Raft log (`mesh_chain.rs`).
  6. Anchors hash on exochain (`chain.rs`).
- **f is a deployment-time parameter.** For a 20-buoy field with ship leader, default
  f_expected = 5 (25%), matching typical military Byzantine-tolerance assumptions. For
  autonomous swarm ops, go higher.

### Concrete aggregator spec (`weftos-sonobuoy-fl-aggregator` crate)

```
pub struct KrumConfig {
    pub n: usize,           // total participating buoys this round
    pub f: usize,           // assumed Byzantine count
    pub m: usize,           // Multi-Krum bottom-m (default n - f)
    pub mode: AggMode,      // Krum, MultiKrum, Bulyan, CoordWiseMedian
}
pub fn aggregate(updates: &[SignedDeltaW]) -> Result<DeltaW, AggError>;
```

Every update is signed, DGC-encoded, and carries a `node_id` + `round_id`. The aggregator is
stateless across rounds (state lives in Raft log), simple to unit-test against canned Byzantine
inputs.

### Hierarchical deployment

For large fields (100+ buoys), buoys form neighborhoods based on acoustic contact (the
`mesh_discovery.rs` + heartbeat fitness graph already tracks this). Per neighborhood, a
designated head-buoy runs intra-cluster Multi-Krum over its ~10 neighbors and forwards the
aggregate to the surface ship, which runs inter-cluster Multi-Krum. This matches the
hierarchical-mesh topology the WeftOS project already uses.

### Raft interaction

Each FedAvg round is a Raft log entry `FlRound { round_id, aggregated_delta_hash, participants,
krum_config }`. Committing means strict majority of ship/aircraft nodes (not buoys) have witnessed
the aggregation. Buoys are *not* Raft voters — they are data sources.

### Post-attack forensics

Because every gradient submission is signed and chain-anchored, the ship can retroactively
identify which buoy submitted outlier gradients (Krum-score outliers over a window) and either
quarantine or investigate. This is an analytics layer on top of the `chain.rs` event stream.

### ADR implication

*ADR-064 (proposed)*: Federated aggregation uses Multi-Krum as the default rule with f_expected
~= 25% of participating buoys. Plain weighted averaging is disallowed for production. Bulyan is
available as a config flag for higher-threat deployments. The aggregator is a dedicated
stateless crate invoked by the Raft leader per round.

## Follow-up References

1. **El Mhamdi, Guerraoui, Rouault 2018** — "The Hidden Vulnerability of Distributed Learning in
   Byzantium", ICML 2018 (Bulyan). arXiv
   [1802.07927](https://arxiv.org/abs/1802.07927). Composes Krum with trimmed mean to defeat
   "A Little Is Enough"–style attacks.
2. **Yin, Chen, Kannan, Bartlett 2018** — "Byzantine-Robust Distributed Learning: Towards Optimal
   Statistical Rates", ICML 2018. arXiv [1803.01498](https://arxiv.org/abs/1803.01498). Coordinate-wise
   median and trimmed mean with optimal statistical rate bounds.
3. **Pillutla, Kakade, Harchaoui 2019** — "Robust Aggregation for Federated Learning" (RFA),
   IEEE TSP 2022. arXiv [1912.13445](https://arxiv.org/abs/1912.13445). Geometric-median
   aggregation; more tolerant under non-IID honest data than Krum.
4. **Baruch, Baruch, Goldberg 2019** — "A Little Is Enough: Circumventing Defenses For Distributed
   Learning", NeurIPS 2019. arXiv [1902.06156](https://arxiv.org/abs/1902.06156). The canonical
   attack that motivates Bulyan; critical reading before deploying Krum in adversarial conditions.
5. **Cao, Fang, Liu, Gong 2021** — "FLTrust: Byzantine-robust Federated Learning via Trust
   Bootstrapping", NDSS 2021. arXiv [2012.13995](https://arxiv.org/abs/2012.13995). Uses a
   server-side trusted root dataset to bootstrap trust scores on clients — applies naturally to
   sonobuoys where the ship has its own clean hydrophone data.
