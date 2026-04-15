# signSGD — Compressed Optimisation for Non-Convex Problems

## Citation

- **Title**: signSGD: Compressed Optimisation for Non-Convex Problems
- **Authors**: Jeremy Bernstein, Yu-Xiang Wang, Kamyar Azizzadenesheli, Anima Anandkumar
- **Affiliation**: Caltech; UC Santa Barbara; Amazon AI
- **Venue**: Proceedings of the 35th International Conference on Machine Learning (ICML 2018), PMLR 80:560-569, Stockholm
- **arXiv**: [1802.04434](https://arxiv.org/abs/1802.04434) (v1 Feb 2018, v3 Aug 2018)
- **DBLP**: https://dblp.org/rec/conf/icml/BernsteinWAA18.html
- **Caltech record**: https://authors.library.caltech.edu/records/jm2vp-qjb50
- **Reference code**: https://github.com/jxbz/signSGD
- **PDF**: `.planning/sonobuoy/papers/pdfs/signsgd.pdf`

## Status

**verified**. Author list, ICML 2018 venue, Stockholm, arXiv ID, and PMLR volume all corroborated
against arXiv abstract, DBLP, and the Caltech institutional repository. The canonical signSGD
paper. A directly related follow-up — Karimireddy, Rebjock, Stich & Jaggi, *"Error Feedback Fixes
SignSGD and other Gradient Compression Schemes"*, ICML 2019, arXiv:1901.09847 — is treated
here as a pointer rather than a separate analysis because it directly patches this paper's
convergence failure mode.

## One-paragraph Summary

Bernstein, Wang, Azizzadenesheli & Anandkumar (ICML 2018) answer a stark question: *what is the
smallest possible gradient message?* The answer — **one bit per parameter** — is the sign of each
coordinate. Their algorithm, **signSGD**, replaces the SGD update `θ ← θ − η · g` with
`θ ← θ − η · sign(g)`; in the distributed setting, each worker ships only `sign(g_k)` and the
parameter server takes a **majority vote per coordinate**. The paper proves a non-convex convergence
rate of **O(1/√T)** — the same asymptotic rate as uncompressed SGD — under a mild condition on the
ℓ₁/ℓ₂ geometry of the stochastic gradient. A 32× compression ratio over FP32 gradients is obtained
essentially for free on networks where that geometric condition holds; on ImageNet the
momentum-counterpart (Signum) matches Adam. The single major caveat — identified by Karimireddy et
al. 2019 — is that vanilla signSGD does **not** converge to the optimum on some simple convex
problems; the fix is *error feedback* (remember the quantization residual and add it back next
step), which is now the default pairing. For sonobuoys whose only uplink is 340 bps Iridium SBD,
signSGD + error feedback is the "last-bit" building block: any further aggressive compression must
either quantize weights (which means quantizing the *global* model, not the gradient) or skip
participation entirely. One bit/param is the floor.

## Methodology

### The core update

For each worker k and each parameter θ_j:

    g_k,j = ∂L_k/∂θ_j
    m_k,j = sign(g_k,j) ∈ {−1, +1}                  # 1 bit per coordinate
    send m_k,j to server
    At server:  M_j = sign( Σ_k m_k,j )              # majority vote
    θ_j ← θ_j − η · M_j

Each worker uplinks `|θ|` bits (not bytes); the server broadcasts `|θ|` bits back in the basic
variant. For |θ| = 10⁶ this is 125 kB up and 125 kB down — two orders of magnitude smaller than
FP32 and still eight times smaller than FP16.

### Momentum variant (Signum)

    v_k ← β · v_k + (1 − β) · g_k                    # local momentum buffer (FP32)
    m_k = sign(v_k)                                   # sign of smoothed gradient
    server majority vote as above

Signum matches Adam on ImageNet ResNet-50 while shipping one bit per parameter, which is the
paper's headline empirical result.

### Convergence theorem (informal restatement)

Under (i) bounded variance `E‖g − ∇F‖² ≤ σ²`, (ii) Lipschitz gradient with constant L, (iii) the
gradient distribution has *unimodal, symmetric* coordinate-wise noise (a condition the paper
argues holds in practice for deep nets), signSGD with learning rate `η = 1/√T` satisfies

    (1/T) Σ_{t=1}^T E‖∇F(θ_t)‖₁ ≤ O( (F_0 − F*)/√T + L/√T + σ · d^{1/2} / √T )

i.e. the time-averaged ℓ₁ norm of the true gradient converges at rate `O(1/√T)` under appropriate
`η`. The relevant quantity is the **ℓ₁/ℓ₂ geometry**: signSGD converges faster than SGD when the
gradient is dense (flat across coordinates) and slower when sparse (most signal in a few coords).

### Majority-vote distributed variant

With N workers, ℓ₁ convergence scales as `O( σ · d^{1/2} / √(NT) )` under the standard
unimodal-symmetric-noise condition — a **linear speedup in N**, matching distributed SGD. The
majority-vote aggregator is itself 1-bit, so both legs of communication are 1 bit/param.

### Failure mode and patch (Karimireddy 2019)

Vanilla signSGD fails to converge on convex problems where the expected sign of `g` differs from
the sign of `E[g]` — i.e. when individual stochastic gradients disagree with the mean. The fix is
**error feedback (EF-SGD)**:

    e_k ← e_k + g_k − C( e_k + g_k )     # residual from compressor C
    m_k = C( e_k + g_k ) = sign( e_k + g_k )

where `C` is the compression operator (for signSGD, `C(x) = ‖x‖₁/d · sign(x)` scaled or just
`sign(x)`). With error feedback, *every* biased compressor recovers the **same** O(1/√T) rate as
SGD *without* needing the unimodal-symmetric-noise condition — a much more robust guarantee. This
is the operating point WeftOS should adopt.

## Key Results

### ImageNet ResNet-50

- Signum matches Adam's top-1 accuracy (~70.7%) within 0.5% after 90 epochs.
- 1 bit/param in, 1 bit/param out (majority vote): **32× compression vs FP32** exchanges.

### CIFAR-10 ResNet-20

- signSGD converges to ~90% test accuracy (vs ~91% SGD baseline) — 1% loss for 32× compression.

### Distributed speedup

- 8-worker majority-vote signSGD achieves near-linear speedup on CIFAR; at 32 workers the
  convergence slows slightly (heavy-tailed coord-noise breaks the symmetry assumption in the bound).

### Theoretical floor

- At the ℓ∞/ℓ₂ geometric extreme (one coord dominates), signSGD is slower than SGD by a factor
  `√d`. At the ℓ₁/ℓ₂ = √d extreme (fully flat gradient), signSGD is as fast. Real deep-net
  gradients sit near the flat extreme, which is why it works empirically.

### Error-feedback experiment (Karimireddy 2019)

- EF-signSGD on ResNet-18/CIFAR-10 matches SGD accuracy **exactly** and converges on the
  convex counter-examples where vanilla signSGD diverges. With EF, the SGD-level rate proof is
  unconditional on noise symmetry.

## Strengths

- **Minimal codec complexity.** One bit per parameter. No sorting, no residual buffer (without EF),
  no hyperparameters. Implementable on any microcontroller in <100 lines of C.
- **Majority-vote aggregation is Byzantine-resistant at the *bit* level.** An adversary who wants
  to flip `sign(g_j)` must outvote a majority of honest workers; a single Byzantine buoy cannot
  move the aggregated update arbitrarily, unlike linear aggregation. (Contrast FedAvg, where one
  rogue worker with a giant weight delta can dominate.)
- **Deterministic bit budget.** Every round, every worker uplinks exactly `|θ|` bits. No tail, no
  variance, no "sometimes 1 kB, sometimes 100 kB" that sparsification schemes suffer.
- **No residual state required (without EF).** Critical for MCUs with 256–512 KB SRAM that cannot
  store an FP32 residual buffer alongside the weights.
- **Extends to Signum / 1-bit Adam** for near-Adam-quality training under 1-bit comms.

## Limitations

- **Convergence bound depends on a noise assumption that is not always true.** Without error
  feedback, signSGD does not converge on some convex problems. Must be paired with EF for any
  production use.
- **Loses all magnitude information.** A gradient of 10⁻⁶ is treated identically to a gradient of
  10⁶ with the same sign. In highly non-IID FL where some clients have rare classes (the norm in
  sonobuoy fleets where different buoys see different species), the rare-class gradient magnitudes
  are the signal, and signSGD attenuates them.
- **Majority vote is not robust to sybil attacks.** If an adversary controls ≥ N/2 + 1 workers,
  the aggregated sign is theirs.
- **1-bit downlink requires the server to broadcast a 1-bit aggregate**, not the averaged
  float gradient. Can reduce convergence speed; the paper uses 32-bit server broadcast in some
  experiments.
- **No native support for model heterogeneity.** All workers must have identical model shape; this
  is a constraint for mixed-fleet sonobuoy deployments where PAM buoys (long-term, large model) and
  tactical buoys (short-term, tiny model) coexist.
- **Single-step wasted bandwidth.** 32× compression compared to FP32 is good but is 8× worse than
  signSGD-followed-by-TopK + 1-bit quantization (which is what DGC + signSGD layered achieves).

## Portable Details

### Single-buoy bit budget

For a `|θ| = 50 000` sonobuoy model:

    uplink  bits = 50 000 bits = 6 250 B   # ~6 kB per round
    downlink bits = 50 000 bits = 6 250 B   # if aggregator also returns signs

At Iridium SBD sustained 340 bps, one 6 kB sign message takes ~147 s of contiguous uplink time —
feasible for a tactical buoy with a daily 2-minute satellite pass *only if* the model is tiny
(|θ| ≤ ~2 000 for a single SBD packet of ~340 B). See gap-closing addendum for combined schemes.

### Error-feedback signSGD pseudocode

    # one-time:
    e_k ← 0     # residual buffer, FP32, same shape as θ

    # each local step on buoy k:
    g_k ← backward(local_batch)
    r_k ← e_k + g_k
    m_k ← sign(r_k)                          # 1-bit message
    e_k ← r_k − α · m_k                       # keep residual, α ≈ ‖r_k‖₁/d
    send m_k

    # at leader:
    M ← sign( Σ_k m_k )                       # majority vote, 1 bit
    broadcast M
    θ ← θ − η · M

### ℓ₁/ℓ₂ geometry diagnostic

Before adopting signSGD for a given model, compute on a held-out batch:

    r = ‖g‖₁ / (√d · ‖g‖₂)   ∈ [1/√d, 1]

r ≈ 1 means flat gradient — signSGD is as fast as SGD.
r ≈ 1/√d means spiky gradient — signSGD is √d× slower. Use DGC Top-k instead.

### Byzantine resistance at bit level

For N workers with f Byzantine, majority vote is correct on coordinate j when **> (N/2 + 1) − f
honest workers agree on sign(g_j)**. This tolerates up to `f < N/2 − 1` Byzantine workers in the
worst case — a stronger threshold than Multi-Krum's `2f + 2 < N`, though over a weaker property
(just sign, not vector).

### Combining with DGC or FetchSGD

signSGD is an **additive stacking primitive** with Top-k sparsification:

1. Run DGC to select the Top-k indices and their values.
2. For the transmitted values, ship only `sign()` — i.e. 1 bit per selected coord + index.
3. Residual buffer now stores `Δθ − |value_avg| · sign()` rather than zeroing.

This is `|θ| · s + k_idx_bits` where the k_idx term dominates. For s = 0.001, |θ| = 50 000,
this is ~50 indices × 20-bit varint = 1 250 bits ≈ 156 bytes per message — **feasible in one
Iridium SBD packet**.

## Sonobuoy Integration Plan

### Where signSGD slots into WeftOS

signSGD is the *innermost* codec in the FL pipeline: gradient → (sparsify) → **sign** → pack →
sign-crypt → frame. It does not replace DGC, Multi-Krum, or FedAvg; it *layers* under them.

- **Per-buoy component**: a `quant::sign_with_error_feedback` entry in the codec crate
  `weftos-sonobuoy-fl-codec`, usable as the final quantization step before `mesh_framing`.
- **Per-leader component**: a majority-vote aggregator `agg::majority_vote` in
  `weftos-sonobuoy-fl-aggregator`, used when the participating-client mode is "Byzantine-worried,
  bandwidth-starved, accuracy-relaxed" (e.g. during the bootstrap phase of a new fleet).
- **Downlink**: For the global-model broadcast, signSGD alone is **too lossy** — the server must
  broadcast at least the accumulated sum (8-bit or 16-bit), not the 1-bit sign. Asymmetric comms.

### Codec pipeline on the wire (signSGD-only variant)

    [ local Δθ_k ]
        ↓ EF-signSGD encode
    [ {−1, +1}^d packed to 1-bit/param ]
        ↓ delta-RLE if gradient sparse in sign-flips
    [ run-length packed bitstream ]
        ↓ rvf-crypto sign
    [ signed packet ]
        ↓ mesh_framing + Iridium SBD
    [ wire ]

For |θ| = 16 000 (aggressive tiny model), 1-bit encoding → 2 kB. One Iridium SBD packet is 340 B
payload; so at |θ| ≤ 2 720 we fit in one satellite transaction.

### Concrete protocol sketch

1. Buoy trains for E local epochs, maintains EF residual `e_k` across rounds (persistent on flash).
2. At round end: compute `Δθ_k = θ_k − θ_global_at_round_start`.
3. Compute `r_k = e_k + Δθ_k`; `m_k = sign(r_k)`; `e_k ← r_k − αm_k`.
4. Pack `m_k` to 1-bit-per-param, RLE if useful, sign-crypt, queue for next radio window.
5. Leader receives N sign-vectors, takes coordinatewise majority → 1-bit aggregate.
6. Leader broadcasts **int16** accumulated count (not sign) so downlink carries magnitude info;
   buoys apply `θ ← θ − η · count / N` as real-valued update.

### Byzantine coupling

signSGD's majority vote is already `(N/2)`-Byzantine-resistant at coordinate level. To raise this
further, combine with Multi-Krum by ranking workers by the *Hamming distance* of their sign vector
from the median sign vector, and drop the f most distant. Hybrid Hamming-Krum + majority vote
tolerates up to `f < N/4` adversaries with near-zero accuracy loss.

### Deployment-profile fit

- **`sonobuoy-pam` (Iridium SBD, month-scale cadence)**: signSGD + EF, |θ| ≤ 2 000, one round per
  satellite pass. Model is a tiny MLP over hand-crafted acoustic features.
- **`sonobuoy-tactical` (UHF 10-100 kbps, hour-scale cadence)**: Full DGC first, then signSGD on
  the selected values. Model is ≤50 kparam CNN classifier.

### ADR implication

*ADR-082 candidate (see gap-closing addendum)*: Adopt EF-signSGD as the terminal quantization
layer of the FL codec stack. Majority vote as the default aggregator for the bandwidth-starved
deployment profile. Always pair with error feedback; never use naked signSGD.

## How This Closes G5

G5 demands convergence in <10 rounds at <1 kB/buoy/round uplink. signSGD contributes:

- **Theoretical bit floor**: 1 bit/param. Any aggressive FL protocol must converge toward this;
  signSGD shows it is *attainable* with an SGD-level convergence rate.
- **No sort, no residual (without EF), no index overhead**: the sparsification schemes (DGC,
  FetchSGD) incur either index bits or sketch-size bits. signSGD alone does not. For extreme-tiny
  models, this is a genuine advantage.
- **Majority-vote Byzantine resistance** at the bit level, compatible with hierarchical aggregation
  without extra primitives.
- **Deterministic bit budget** — no tail latency from "sometimes I need to ship the big Top-k
  residual". Critical for matching an Iridium SBD window exactly.

The single constraint: signSGD needs `|θ| ≤ ~2000` to fit in a 340 B Iridium SBD packet. Combined
with Top-k (DGC) and per-layer sparsification, this lifts to `|θ| ≤ 100 000` at equivalent bit
budget — the actual operating point for sonobuoy FL.

## Follow-up References

1. **Karimireddy, Rebjock, Stich, Jaggi 2019** — "Error Feedback Fixes SignSGD and other Gradient
   Compression Schemes", ICML 2019, arXiv [1901.09847](https://arxiv.org/abs/1901.09847).
   Mandatory pairing with signSGD; proves that error feedback gives SGD-rate convergence for any
   compressor without additional assumptions.
2. **Seide et al. 2014** — "1-Bit Stochastic Gradient Descent and its Application to Data-Parallel
   Distributed Training of Speech DNNs", Interspeech 2014. The predecessor — also 1 bit/param with
   per-column scaling, but without theoretical guarantees.
3. **Wen et al. 2017** — "TernGrad: Ternary Gradients to Reduce Communication in Distributed Deep
   Learning", NeurIPS 2017, arXiv [1705.07878](https://arxiv.org/abs/1705.07878). 3-value {−1, 0,
   +1} quantization — 2× more bits than signSGD but preserves "gradient is zero" information
   exactly.
4. **Alistarh et al. 2017** — "QSGD: Communication-Efficient SGD via Gradient Quantization and
   Encoding", NeurIPS 2017, arXiv [1610.02132](https://arxiv.org/abs/1610.02132). Parameterized
   quantization with provable convergence; the theoretical elder of signSGD.
5. **Bernstein, Zhao, Azizzadenesheli, Anandkumar 2019** — "signSGD with Majority Vote is
   Communication Efficient and Fault Tolerant", ICLR 2019, arXiv
   [1810.05291](https://arxiv.org/abs/1810.05291). The same authors' follow-up that adds the
   Byzantine-resistance analysis of majority-vote signSGD used above.
