# Split Learning — Distributed Learning of Deep Neural Network over Multiple Agents

## Citation

- **Title**: Distributed learning of deep neural network over multiple agents
- **Authors**: Otkrist Gupta, Ramesh Raskar
- **Affiliation**: MIT Media Lab (Camera Culture Group)
- **Venue**: *Journal of Network and Computer Applications*, Vol. 116, August 2018, pp. 1–8
- **DOI**: [10.1016/j.jnca.2018.05.003](https://doi.org/10.1016/j.jnca.2018.05.003)
- **arXiv**: [1810.06060](https://arxiv.org/abs/1810.06060) (posted to arXiv Oct 2018, after JNCA publication)
- **DSpace@MIT**: https://dspace.mit.edu/handle/1721.1/121966
- **PDF**: `.planning/sonobuoy/papers/pdfs/split-learning.pdf`

## Status

**verified**. Authors, JNCA venue + DOI, arXiv preprint ID, and MIT DSpace archival copy all
corroborated. This is the canonical "split learning" paper, though the term *split learning*
itself was coined in the follow-up Vepakomma-Gupta-Swedish-Raskar preprint a few months later
(arXiv 1812.00564, "Split learning for health"). The 2018 JNCA paper is the architectural
foundation.

## One-paragraph Summary

Gupta & Raskar (JNCA 2018) propose a distributed-learning pattern that is *architecturally
orthogonal* to federated averaging: instead of training a full model copy at each client and
averaging weights, they **cut the network between two layers** and have the client train only
the first `k` layers, the server train the remaining `N − k` layers, and only the **intermediate
activations** (during forward pass) and the **gradients at the cut layer** (during backward pass)
flow over the network. No raw data leaves the client; no full model sits at the client; no
weight averaging happens. The paper's main pragmatic argument is that for **large, deep networks
with small cut-layer feature maps, split learning transmits dramatically less data per training
step than FedAvg** — because an activation map at a deep layer is tiny compared to shipping the
entire model's parameters. This architectural asymmetry — compute-light client, bandwidth-matched
to the intermediate-activation size, privacy-preserving by construction — makes split learning
highly relevant to sonobuoys: a buoy runs only the first few CNN layers over its hydrophone
stream, ships a compressed feature tensor to the ship, and the ship/aircraft runs the rest of
the classifier. Training is the same protocol run in reverse. The three variants introduced —
**vanilla split, U-shaped split, and vertical split (with label sharing)** — map directly to
three sonobuoy deployment patterns.

## Methodology

### The split-learning idea

A neural network with L layers `θ = (θ_1, ..., θ_L)` is partitioned into a **client sub-network**
`θ_C = (θ_1, ..., θ_k)` and a **server sub-network** `θ_S = (θ_{k+1}, ..., θ_L)`. The layer
index `k` is called the **cut layer**.

**Forward pass for one training example (x, y)**:

    a_k = f_C(x; θ_C)               # client computes activations at cut layer
    (send a_k to server)
    ŷ = f_S(a_k; θ_S)               # server completes forward
    L = loss(ŷ, y)                  # typically computed on server (see variant 3)

**Backward pass**:

    ∂L/∂θ_S, ∂L/∂a_k = server_backprop(L, a_k, θ_S)
    (send ∂L/∂a_k to client)
    ∂L/∂θ_C = client_backprop(∂L/∂a_k, θ_C)
    θ_C ← θ_C − η ∂L/∂θ_C           # client update
    θ_S ← θ_S − η ∂L/∂θ_S           # server update

The network on the wire carries **activations at the cut layer (forward)** and **gradients at
the cut layer (backward)**. Each is a tensor of shape matching `a_k`, typically much smaller
than `|θ|`.

### The three protocol variants

**Variant 1 — Vanilla (raw-label) split**:
- Client sends `a_k` forward.
- Server computes loss *using labels that are also sent to the server*.
- Server sends `∂L/∂a_k` backward.
- Simplest. Labels cross the network; privacy weaker than federated averaging.

**Variant 2 — U-shaped split (label privacy)**:
- Client sends `a_k` forward.
- Server computes hidden representation `a_L` (pre-output).
- Server sends `a_L` back to client.
- Client computes final output and loss using its own labels.
- Client sends `∂L/∂a_L` forward to server.
- Server backprops to `∂L/∂a_k`, sends backward.
- Client backprops its two ends.
- Two round-trips per step, but **labels never leave the client**.

**Variant 3 — Vertical split (cross-entity feature concatenation)**:
- Multiple clients each hold *different features* of the same sample (cross-silo).
- Each client runs its own `θ_C^{(i)}` on its features, producing partial `a_k^{(i)}`.
- Server concatenates partials, completes the model.
- Relevant when (e.g.) different buoy *types* — passive hydrophone, DIFAR, active — each
  contribute a different feature tensor for the same contact.

### Multi-client sequential training

For N clients sharing one server-side model, the paper describes a **relay protocol**: client 1
trains the client-side parameters for an epoch (with the server in the loop), then ships its
client-side weights to client 2, which continues training. This is distinct from FedAvg — there
is no averaging; the client model is a single shared object that migrates. Later "SplitFed"
work (Thapa et al. 2020) combines split learning with FedAvg on the client side, removing the
relay requirement.

### Convergence

Because split learning trains *the exact same network* as a monolithic SGD would — just with
forward/backward computation split across machines — its convergence is **identical** to
centralized training, assuming deterministic cut-layer communication and no compression of the
tensors on the wire. There is no FedAvg-style "client drift" problem because there's no local
averaging. This is the single strongest theoretical argument for split learning vs FedAvg.

### Communication cost per training step

    bytes_per_step = size(a_k) + size(∂L/∂a_k) + size(labels)   [Variant 1]
                   = 2 · size(a_k) + 2 · size(a_L)              [Variant 2]

Compare to FedAvg:

    bytes_per_round = 2 · |θ|                                    [one up + one down]

For a CNN with a deep-cut feature map of, say, 128 channels × 14 × 14 at FP16, that's
128·14·14·2 = 50 kB per step. For |θ| = 20 MB, FedAvg is 40 MB per round. So per *step*, split
learning is ~800× cheaper in bandwidth. But split learning needs a step per training example
(or per minibatch if you pipeline), while FedAvg amortizes over E local epochs. The correct
comparison is *per wall-clock training run*:

- **Small client data, large model** → split learning wins (FedAvg's |θ| cost dominates).
- **Large client data, small model, rare rounds** → FedAvg wins (split's per-batch cost dominates).

For sonobuoys this means: during *rare batched training windows* on many samples, FedAvg wins.
During *continuous inference* (which is also a forward pass with no backward), split inference
wins by the same 800× factor for small cut-layer activations.

## Key Results

The Gupta & Raskar 2018 paper's main empirical exhibits:

### CIFAR-10 / CIFAR-100 with VGG

- Cut layer after the first block (first convolution + ReLU + maxpool). Client holds ~10% of
  parameters; server holds ~90%.
- Accuracy matches centralized training to within ~0.5%. This is expected — the math is the
  same, only the compute is split.

### ILSVRC / ImageNet with ResNet-50

- Cut layer early (after stem). The forward activation at that cut is 64 × 56 × 56 = ~200 KB
  per image at FP32; ~100 KB at FP16.
- Per-image client bandwidth: ~200 KB forward + ~200 KB backward = ~400 KB per image per step.
- Compared to shipping ResNet-50 full weights (~100 MB per round for FedAvg) — split is
  ~250× smaller per sample.

### Per-client compute

The client computes only ~5–10% of the total FLOPs (the stem + one block). This is the
load-bearing *compute* argument: unlike FedAvg, where each client runs the whole model, split
learning lets a weak edge device (think: microcontroller-class sonobuoy) carry only the cheap
early layers.

### Multi-agent relay experiments

With N = 100 clients relaying the client-side model, total accuracy matches centralized. No
averaging artifacts, no drift.

### The paper does **not** claim**

- Any convergence-rate speedup vs centralized training (nor could it — they're equivalent).
- Any strict privacy guarantee. The intermediate activations `a_k` can often be partially
  inverted back to input; *differential privacy* or *adversarial masking* must be layered on top
  for real privacy. Later work (Abuadbba et al. 2020, Vepakomma et al. 2020 on NoPeek)
  addresses this explicitly.

## Strengths

- **Client is compute-light.** 5–10% of FLOPs, 5–10% of parameter memory. A sonobuoy can run a
  client-side CNN stem in SRAM; it could not run the whole ResNet-50.
- **Activation-size bandwidth, not parameter-size bandwidth.** For deep networks with small
  cut-layer activations, the per-step cost is orders of magnitude smaller than FedAvg's
  per-round cost.
- **Same convergence as centralized.** No client drift, no averaging bias. What you train is
  what you get.
- **Raw data never leaves the client.** Weaker privacy than DP-FedAvg (activations can be
  inverted), but stronger than shipping raw audio.
- **Naturally supports heterogeneous hardware.** A cheap buoy runs a 2-layer stem, the ship
  runs a 50-layer trunk. Each side matches its compute to its hardware.
- **Symmetric with inference.** Once trained, *inference* uses exactly the same split — buoy
  computes stem, ship computes trunk. No separate inference protocol needed.

## Limitations

- **Latency-sensitive.** Each forward/backward requires a round trip on the cut link.
  High-latency radio + small minibatch = stalled client. Pipelining across minibatches helps
  but breaks equivalence to centralized training.
- **Label privacy requires U-shape.** Vanilla split ships labels; in a military acoustic context
  the labels *are* the classification outcomes (submarine yes/no), so this matters.
- **Activation inversion attacks.** A malicious server can train an inverter network that maps
  `a_k → x̂`. For early cut layers, `x̂` can be quite faithful. Mitigations: NoPeek regularizer,
  feature masking, DP noise on activations.
- **Sequential multi-client relay is slow.** N clients sharing one model → training time scales
  linearly in N. SplitFed (split + FedAvg on client) parallelizes this.
- **Server becomes a trust / bottleneck concentration.** All cut-layer activations from all
  buoys flow through one trunk; compromise the ship's GPU and you see everything, unlike FedAvg
  where a server sees only parameter deltas.
- **Non-IID is implicitly a single-model problem.** If buoys A and B have very different data
  distributions, there is no "local personalization" step by default.

## Portable Details

### Forward / backward equations at the cut

Let `f_C` and `f_S` be the client and server sub-networks; `a_k ∈ ℝ^{d_k}` the cut-layer
activation.

    Forward:    a_k = f_C(x; θ_C);   ŷ = f_S(a_k; θ_S)
    Loss:       L   = ℓ(ŷ, y)
    Backward:   ∂L/∂a_k = transmit from server
                ∂L/∂θ_C = BackpropClient(∂L/∂a_k, cache_C)
                ∂L/∂θ_S = BackpropServer(∂L/∂ŷ, cache_S)

### Bandwidth formula (Variant 1)

    per_step = (d_k · B · bytes_per_elem)    # forward activation, batch B
             + (d_k · B · bytes_per_elem)    # backward gradient
             + (d_y · B · bytes_per_label)   # labels
             ≈ 2 · d_k · B · bytes_per_elem  when labels are small

### Cut layer selection rule of thumb

- **Cut deeper** → smaller activations (good for bandwidth), larger client model (bad for
  memory), harder to invert activations (good for privacy).
- **Cut shallower** → larger activations (bad for bandwidth), tiny client (good for memory),
  activations reveal more about input (bad for privacy).

For sonobuoys: cut after **first 2–3 conv blocks** of a compact CNN, yielding feature maps of
~32 channels × 16 × 16 = ~8 KB per 1-second window. At 1 kbps radio, that's ~64 seconds to ship
one training step's activation — still infeasible for raw per-step protocol, but *trivial* for
FedAvg-style rare batched rounds.

### Composition with FedAvg (SplitFed)

Run split learning as the per-round inner loop, FedAvg as the outer loop:
- Each buoy does one split-learning training pass on its local batch (sending activations to
  ship, receiving gradients), then *ships its updated client-side weights θ_C^{(k)}* to the
  ship.
- Ship averages `{θ_C^{(k)}}` into a single `θ_C` (FedAvg on the client side).
- Ship also updates its own `θ_S` directly.
- This is the Thapa et al. 2020 SplitFed protocol. Matches the sonobuoy pattern well.

### Inference mode

Exactly the forward path, no backward: `a_k = f_C(x; θ_C)` on the buoy, ship to vessel,
`ŷ = f_S(a_k; θ_S)` on the vessel. Already compressed vs shipping raw waveform, and stays on
the same codepath as the training-plane forward.

## Sonobuoy Integration Plan

### Which split variant fits which sonobuoy scenario

- **Variant 1 (raw-label split)**: the ship already has labeled data (e.g. it's running its own
  classifier on a reference hydrophone). Acceptable if labels do not leak tactical info upward.
- **Variant 2 (U-shape)**: the buoy somehow gets labels (e.g. from a separate active-sonar
  cueing source). Labels stay on the buoy; only activations and pre-output representations
  cross. **Most privacy-preserving, best for classified deployments.**
- **Variant 3 (vertical split)**: heterogeneous buoy types — passive, DIFAR, active — each
  contribute different features for the same contact. Server concatenates. This is the
  **heterogeneous-device federated learning** case and is the single most compelling match for
  real sonobuoy fields with mixed hardware.

### Where split learning slots into the WeftOS mesh

Split learning is **not** a mesh-primitive; it's a client-server protocol that rides on top of
the existing mesh. The mapping:

- **Cut-layer activation tensor** is just another `mesh_artifact` payload, typed
  `SplitActivation { round_id, sample_id, shape, dtype, quant, data }`.
- **Cut-layer gradient tensor** is the inverse payload direction.
- **Flow control**: use existing `mesh_framing.rs` + `mesh_tcp.rs` or `mesh_ws.rs` for the
  underlying transport. On UHF/VHF radio, replace `mesh_tcp` with a KISS/AX.25 variant (future
  work).
- **Sequencing / reliability**: existing `reliable_queue.rs` handles at-least-once delivery
  with ACK tracking.
- **Trust**: rvf-crypto signs each activation/gradient; a Byzantine buoy cannot corrupt the
  ship's training without being detectable.

The *role* mapping:

| Split-learning role       | WeftOS component                           |
|----------------------------|--------------------------------------------|
| Client `f_C`                | per-buoy crate `weftos-sonobuoy-stem`    |
| Server `f_S`                | ship-resident crate `weftos-sonobuoy-trunk` |
| Cut-layer transport         | `mesh_artifact` + `reliable_queue`         |
| Signing/auth                | rvf-crypto (already present)               |
| Round sequencing            | Raft log entry per round                   |
| Inference mode (no backward)| same stem/trunk at eval time                |

### Concrete protocol for a sonobuoy deployment

**Inference (continuous)**:
1. Every ~1 s of hydrophone audio, buoy's stem produces `a_k` (~8 KB, INT8-quantized).
2. Buoy signs and queues for next radio window.
3. During window, buoy uplinks batched a_k frames. Ship runs trunk → contact detection.
4. No backward flow; inference is one-way.

**Training (rare, batched)**:
1. Ship signals a training window. Each participating buoy retains its last N labeled samples
   (labeled by the ship during inference, via periodic label echo on the downlink).
2. In the training window, buoy runs forward → cut → activations up; ship backprops → gradients
   down → client updates locally.
3. Entire per-sample cost is ~16 KB (activation + gradient + label). 1-minute radio window at
   9600 baud handles ~4 samples per window.
4. Periodically (every Q rounds), apply SplitFed on the client-side stem to synchronize buoys
   via FedAvg on the client-side weights.

### Comparison with FedAvg for sonobuoys (one-table summary)

| Axis                        | FedAvg                    | Split learning (incl. SplitFed) |
|-----------------------------|---------------------------|---------------------------------|
| Per-round bandwidth         | 2·\|θ\| (can be 10s of MB) | Small activations (KB)          |
| Per-sample bandwidth        | 0 (training is local)      | \|a_k\| · 2 per sample          |
| Client compute              | Full model                | Stem only (~10%)                |
| Convergence                 | O(1/T) under non-IID      | Same as centralized             |
| Privacy (raw data)          | Never leaves               | Never leaves                    |
| Label privacy               | Yes (labels local)        | Needs U-shape                   |
| Byzantine resilience        | Plug in Krum              | Gradients signed; activations harder to attack — early work needed |
| Good for                    | Rare batch training         | Continuous inference + occasional training |

### ADR implication

*ADR-065 (proposed)*: The sonobuoy ML plane uses a split-learning architecture at the stem/trunk
boundary. Stem runs on the buoy, trunk on the ship. Inference is a pure forward stem-uplink;
training is periodic SplitFed (split per-sample + FedAvg periodic on client weights). Variant 2
(U-shape) is used whenever labels are tactically sensitive; Variant 3 (vertical) is required to
combine heterogeneous buoy feeds (passive + DIFAR + active) into one trunk.

## Follow-up References

1. **Vepakomma, Gupta, Swedish, Raskar 2018** — "Split learning for health: Distributed deep
   learning without sharing raw patient data", NeurIPS ML4H workshop. arXiv
   [1812.00564](https://arxiv.org/abs/1812.00564). Coins the term "split learning" and gives
   the vanilla/U/vertical variant taxonomy.
2. **Thapa, Chamikara, Camtepe 2020** — "SplitFed: When Federated Learning Meets Split Learning",
   AAAI 2022. arXiv [2004.12088](https://arxiv.org/abs/2004.12088). Combines FedAvg on the
   client side with split learning on the server side; removes the sequential relay bottleneck
   of vanilla split learning.
3. **Abuadbba et al. 2020** — "Can We Use Split Learning on 1D CNN Models for Privacy Preserving
   Training?", AsiaCCS 2020. arXiv [2003.12365](https://arxiv.org/abs/2003.12365). Studies
   activation-inversion attacks on 1D CNNs — directly relevant to raw-audio hydrophone models.
4. **Vepakomma, Singh, Gupta, Raskar 2020** — "NoPeek: Information leakage reduction to share
   activations in distributed deep learning", arXiv
   [2008.09161](https://arxiv.org/abs/2008.09161). Adds a mutual-information regularizer to
   harden activations against inversion. Needed before deploying split learning in adversarial
   waters.
5. **Poirot, Vepakomma, Chang, Kalpathy-Cramer, Gupta, Raskar 2019** — "Split Learning for
   Collaborative Deep Learning in Healthcare", NeurIPS ML4H workshop. arXiv
   [1912.12115](https://arxiv.org/abs/1912.12115). Comparative study of split learning vs
   FedAvg bandwidth across different model sizes; the empirical reference for "when does split
   beat FedAvg?".
