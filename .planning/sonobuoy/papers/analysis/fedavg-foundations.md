# FedAvg — Communication-Efficient Learning of Deep Networks from Decentralized Data

## Citation

- **Title**: Communication-Efficient Learning of Deep Networks from Decentralized Data
- **Authors**: H. Brendan McMahan, Eider Moore, Daniel Ramage, Seth Hampson, Blaise Agüera y Arcas
- **Affiliation**: Google
- **Venue**: Proceedings of the 20th International Conference on Artificial Intelligence and Statistics (AISTATS 2017), PMLR 54:1273-1282
- **arXiv**: [1602.05629](https://arxiv.org/abs/1602.05629) (first posted Feb 2016, final v4 Feb 2023)
- **PMLR**: https://proceedings.mlr.press/v54/mcmahan17a.html
- **PDF**: `.planning/sonobuoy/papers/pdfs/fedavg-foundations.pdf`

## Status

**verified**. Author list, arXiv ID, AISTATS 2017 venue, and the canonical 10–100× round-reduction claim all corroborated against arXiv abstract page, Google Research publications page, and PMLR proceedings. This is the originating FedAvg paper.

## One-paragraph Summary

McMahan et al. 2017 introduces **Federated Learning** as a named problem and gives the algorithm —
**FederatedAveraging (FedAvg)** — that nearly every subsequent FL paper in the literature uses as
either a direct component or a baseline. The core idea is simple but load-bearing: instead of
shipping client data to a central server and running minibatch SGD there, the server ships the
*current model* to a random sample of clients, each client runs *several local SGD epochs* on its
private data, and the server averages the returned model weights to form the next global model.
The surprising empirical finding — the paper's main contribution — is that for realistic federated
workloads (unbalanced, non-IID client data), running multiple local epochs per round before
averaging gives **10× to 100× fewer communication rounds** than synchronous distributed SGD at
equal final accuracy, with only small degradation under pathological non-IID splits. This is the
property that makes federated learning viable on bandwidth-constrained links like mobile radio,
ocean-acoustic mesh, or satellite backhaul: you pay in local compute to buy global bandwidth.

## Methodology

### Problem setting

- K clients, each with a private dataset P_k of size n_k. Total samples n = Σ n_k.
- The global objective is a weighted sum of local empirical risks:

  f(w) = Σ_k (n_k / n) · F_k(w),   where   F_k(w) = (1/n_k) Σ_{i∈P_k} ℓ(w; x_i, y_i).

- Clients may have **non-IID data**, **unbalanced n_k**, and be **massively distributed**
  (thousands to millions of clients, each participating rarely).
- The paper explicitly frames *communication rounds* — not wall-clock or FLOPs — as the scarce
  resource, because for mobile/federated deployments local compute is cheap and the uplink is
  metered/slow.

### FedAvg algorithm (Algorithm 1 in the paper)

```
Server executes:
  initialize w_0
  for each round t = 0, 1, 2, ... :
    m  ← max(C · K, 1)                       # number of clients this round
    S_t ← random subset of m clients
    for each client k ∈ S_t in parallel:
        w_{t+1}^k ← ClientUpdate(k, w_t)
    w_{t+1} ← Σ_k (n_k / Σ_{j∈S_t} n_j) · w_{t+1}^k   # weighted average

ClientUpdate(k, w):
    B ← split P_k into size-B minibatches
    for each local epoch i = 1 ... E:
        for each batch b ∈ B:
            w ← w − η · ∇ℓ(w; b)
    return w
```

Three hyperparameters dominate FedAvg behavior:

- **C** — client fraction per round. Low C (0.1) reduces wall-clock and per-round bandwidth but
  raises round count; high C (1.0) is closer to classic distributed SGD.
- **E** — number of local epochs. E=1 makes FedAvg degenerate into **FedSGD** (one gradient step
  per client per round). E > 1 is where the communication savings come from. Typical E ∈ {1, 5, 20}.
- **B** — local minibatch size. B=∞ means full-batch on the client.

### Convergence analysis

The original AISTATS paper is **primarily empirical**; it does *not* prove convergence of FedAvg
in general. Rigorous convergence bounds came later:

- Li et al. 2020 ("On the Convergence of FedAvg on Non-IID Data", ICLR) proved **O(1/T)** rate
  for strongly-convex, smooth f under bounded dissimilarity — but only if learning rate decays.
- Karimireddy et al. 2020 (SCAFFOLD) showed that on heterogeneous (non-IID) clients, FedAvg
  suffers **client drift**: local models pull toward client-specific optima, so the average is
  biased. SCAFFOLD introduces control variates to fix this.
- Wang et al. 2020 (FedNova) characterize how uneven n_k and uneven E create a *normalization*
  bug that FedAvg cannot correct.

For a bandwidth-constrained sonobuoy deployment this matters: the convergence guarantees we can
count on assume approximately-IID shards and a tamed learning-rate schedule. Ocean acoustics are
not IID (station near a shipping lane vs a quiet canyon is a very different marginal distribution),
so SCAFFOLD-style variance reduction is likely a required add-on, not optional.

### Communication cost per round

- **Uplink** per participating client: `|θ|` parameters (compressed or plain).
- **Downlink** per round: `|θ|` parameters broadcast from server to `m = C·K` clients.
- **No raw data** ever leaves the client.

For a 10 MB model, 100 clients, C=0.1 (so m=10), one round costs ~10 MB × 10 = 100 MB uplink
+ 10 MB × 10 = 100 MB downlink. *Per round*. This is where Deep Gradient Compression (see
`deep-gradient-compression.md`) becomes critical for VHF/UHF-bottlenecked networks.

## Key Results

The paper tests four model/dataset pairs. For each, the reported *speedup* = rounds of FedSGD
(baseline) needed to reach a target accuracy, divided by rounds of FedAvg needed for the same
target. These numbers are widely cited in the FL literature:

### MNIST, 2NN multi-layer perceptron

- Target: 97% test accuracy.
- IID sharding across 100 clients: **~3–12× fewer rounds** (E=5 vs E=1).
- Pathological non-IID (each client sees at most 2 of 10 digits): **~30× fewer rounds** — the
  non-IID setting, counter-intuitively, benefits *more* from local epochs because communication
  is such a bottleneck that even noisy local progress is worth it.

### MNIST CNN

- Target: 99% test accuracy.
- IID: ~18× speedup at E=5, B=10.
- Non-IID: still converges, but with higher variance; E too large (e.g. E=20) begins to *hurt*
  on non-IID because client drift dominates.

### CIFAR-10 CNN

- Target: 80% test accuracy.
- IID over 100 clients, C=0.1: FedAvg reaches target in ~280 rounds vs FedSGD ~ 2400+.
- ~**8–10×** speedup is the honest headline number for CIFAR.

### LSTM language model (Shakespeare by-character, ~1100 clients = authors)

- Task: next-character prediction.
- FedAvg reaches the baseline LSTM accuracy in **~95% fewer rounds** than FedSGD.
- Shakespeare is *naturally non-IID* — each author's writing is its own distribution — so this is
  the most realistic experiment in the paper and shows FedAvg works on genuinely heterogeneous data.

### The overall 10–100× claim

The "10× to 100× reduction" headline comes from aggregating these four tables. It is not the
result of a single experiment. The claim holds reliably when:

1. C is small (~0.1), so each round's bandwidth cost is bounded.
2. E is tuned to the task (E ∈ [1, 20] empirically; too large hurts non-IID convergence).
3. Learning rate decays on the server side.

## Strengths

- **Named the problem.** Before this paper, "federated learning" was not a standard term in ML;
  afterward it is a subfield with its own conferences (FL@NeurIPS workshop, ICML FL workshop).
- **Algorithm is trivially implementable.** FedAvg fits in ~30 lines of pseudocode and is
  essentially weighted parameter averaging. The bar for adoption is extremely low.
- **Empirically validates the key insight.** Pay in local compute, save on communication. This
  trade is correct for phones, and it will be even more correct for sonobuoys.
- **Robust to non-IID data** (at least empirically, at reasonable E). Later theory would formalize
  when it breaks, but the headline "FedAvg does not fall apart on non-IID data" survives.
- **Foundation for an entire subfield.** SCAFFOLD, FedProx, FedNova, Split Learning, Secure
  Aggregation, differentially-private FL — every one of these papers uses FedAvg as the baseline.

## Limitations

- **No convergence proof.** The AISTATS 2017 paper does not theoretically guarantee convergence;
  follow-up work filled this in only for restricted settings.
- **Sensitive to E under non-IID data.** Client drift at large E can cause divergence. The paper
  handles this with hyperparameter sweeping rather than a principled fix.
- **Assumes trusted clients and server.** No Byzantine tolerance (see `byzantine-robust-krum.md`),
  no privacy guarantees (see secure aggregation / DP-FedAvg), no robustness to stragglers — all
  this arrives in follow-up papers.
- **Weighted averaging assumes n_k is known and trusted.** A Byzantine sonobuoy can lie about n_k
  to amplify its weight in the aggregation. Robust aggregation replaces this.
- **Synchronous rounds.** All selected clients must finish before the server averages. In a
  sonobuoy field where radio windows are intermittent, pure synchronous FedAvg wastes rounds.
  Asynchronous variants (FedAsync, FedBuff) address this at the cost of extra analysis.

## Portable Details

### The aggregation equation

The sonobuoy project will implement this as its baseline aggregator:

    w_{t+1} = Σ_{k ∈ S_t} (n_k / N_t) · w_{t+1}^k     where N_t = Σ_{k ∈ S_t} n_k

This is a **convex combination** — a weighted mean — with weights proportional to local dataset
size. In WeftOS this is one reduce-step over `mesh_service_adv` advertisements that include
`{node_id, n_k, w_k}` payloads.

### Communication cost per round per client

    bytes_up   = |θ| · bytes_per_param · (C · K) / K     # expected
    bytes_down = |θ| · bytes_per_param                   # fixed broadcast

For a typical sonobuoy-resident classifier (say a small EfficientNet-B0 at 5 M fp32 params =
20 MB), and a ~kbps uplink, a naive round takes **20 MB / ~1 kbps ≈ 44 hours**. This is the
*reason* the next three papers (compression, split learning, Byzantine robustness) exist.

### FedSGD ≡ FedAvg with E=1, B=∞

Use this fact to share code paths: the same FedAvg implementation with E=1 and full-batch local
"training" *is* synchronous distributed SGD. The sonobuoy stack should treat FedSGD as a
degenerate configuration of FedAvg, not as a separate algorithm.

### The "when is FedAvg good?" heuristic

- High bandwidth, low latency, trusted clients, IID data → FedSGD (E=1) is fine.
- Low bandwidth, untrusted clients, non-IID data → FedAvg (E > 1) with robust aggregation and
  compression. **This is the sonobuoy case.**

## Sonobuoy Integration Plan

### Where FedAvg slots in the WeftOS mesh

WeftOS already has the infrastructure that FedAvg's server role requires:

1. **Raft** (`mesh_chain.rs`, `chain.rs`) — gives us a *total order* over model-update commits.
   This is overkill for raw FedAvg but becomes necessary for reproducible experiments and for
   preventing split-brain aggregations.
2. **Gossip / service advertisement** (`mesh_service_adv.rs`, `mesh_kad.rs`) — handles
   sonobuoy-to-sonobuoy discovery and model-chunk distribution.
3. **rvf-crypto** (identified in project memory) — signs each client update so the aggregator
   can attribute and refuse unsigned weight deltas.

The **mapping** is:

| FedAvg role     | WeftOS primitive                            |
|-----------------|---------------------------------------------|
| Global model broadcast   | gossip over `mesh_service_adv` + chunked artifact store (`mesh_artifact.rs`) |
| Client selection / S_t    | Raft leader picks via `mesh_discovery.rs`+ heartbeat fitness |
| Client update (w_k)      | local compute crate, signed by rvf-crypto |
| Server-side weighted average | Raft leader applies reduce on committed log entries |
| Checkpoint / rollback   | exochain (`chain.rs`) anchors model hashes per round |

### Protocol for a single FedAvg round on a sonobuoy field

Assumption: Ka-band/UHF uplink to an aircraft or surface ship acting as the Raft leader, with
~1 kbps to ~10 kbps per-buoy bandwidth, long windows of radio silence, and an adversarial
threat model (some buoys may be spoofed or captured).

1. **Aircraft/ship = Raft leader** broadcasts `ModelRoundStart(t, sha256(w_t), chunks[])` via
   gossip. Each buoy pulls chunks on demand during radio contact; downlink is the cheapest side.
2. **Client selection**: leader publishes S_t in the Raft log. Each selected buoy is notified
   when it next comes into radio range.
3. **Local training**: each buoy runs E epochs over its on-board hydrophone/DIFAR spool. Training
   is rare and batched — nightly or per-deployment, not continuous — and uses the `eml-core` +
   `democritus` stack already present in `clawft-kernel/src/`.
4. **Compressed upload**: buoy sends `Δw_k = w_k − w_t` as a **Top-k sparsified, quantized,
   signed** payload (see `deep-gradient-compression.md`). For 5 M params at k=0.1%, 8-bit quant,
   that's ~5 kB/round — feasible in a ~1 min radio window.
5. **Robust aggregate**: leader runs **Multi-Krum** on the received Δw_k instead of plain
   weighted mean (see `byzantine-robust-krum.md`), then commits the new w_{t+1} via Raft.
6. **Chain anchor**: `chain.rs` appends `sha256(w_{t+1})` as an audit log so a later reviewer can
   verify which buoys contributed to which global model.

### What FedAvg does *not* fit

- Continuous inference (buoys classify contacts in real time with the *current* model, not the
  round-local model). Inference is asynchronous to the training loop.
- Autonomous peer-to-peer swarms with *no* surface leader. For that case see gossip-FL
  (decentralized SGD / D-SGD) — a planned follow-up paper slot.

### Concrete ADR implication

*ADR-062 (proposed)*: The sonobuoy training plane uses FedAvg at the macro-round level, with
Raft-ordered round commits, Multi-Krum aggregation, and DGC-compressed uplinks. Inference and
training run in separate processes on each buoy; training is gated on available radio window
and battery SOC.

## Follow-up References

1. **Li et al. 2020** — "On the Convergence of FedAvg on Non-IID Data", ICLR 2020.
   arXiv [1907.02189](https://arxiv.org/abs/1907.02189). First rigorous convergence proof of
   FedAvg under heterogeneity.
2. **Karimireddy et al. 2020** — "SCAFFOLD: Stochastic Controlled Averaging for Federated
   Learning", ICML 2020. arXiv [1910.06378](https://arxiv.org/abs/1910.06378). Fixes client
   drift with control variates; critical for genuinely non-IID acoustic clients.
3. **Wang et al. 2020** — "Tackling the Objective Inconsistency Problem in Heterogeneous
   Federated Optimization" (FedNova), NeurIPS 2020. arXiv
   [2007.07481](https://arxiv.org/abs/2007.07481). Addresses uneven local epochs / data sizes.
4. **Li et al. 2020** — "Federated Optimization in Heterogeneous Networks" (FedProx), MLSys 2020.
   arXiv [1812.06127](https://arxiv.org/abs/1812.06127). Adds a proximal term to tame client
   drift under system heterogeneity (straggler buoys, partial participation).
5. **Kairouz et al. 2021** — "Advances and Open Problems in Federated Learning", *Foundations
   and Trends in Machine Learning*. arXiv [1912.04977](https://arxiv.org/abs/1912.04977). The
   canonical 120-page survey; mandatory reading before writing any FL ADR.
