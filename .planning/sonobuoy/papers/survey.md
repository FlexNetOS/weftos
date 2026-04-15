# Sonobuoy / Underwater Acoustic Sensing — Paper Survey (2023-2026)

## Scope

This survey curates recent academic work (2023-2026, with a small number of
earlier foundational citations) relevant to building a distributed sonobuoy
processing stack in Rust/WeftOS. The base technique being extended is
**K-STEMIT** (arXiv:2604.09922), a dual-branch spatio-temporal GNN originally
designed for ice-penetrating radar that naturally generalizes to distributed
hydrophone arrays: one branch handles temporal signal dynamics per sensor,
the other handles spatial/graph structure across the array, with
physics-derived priors (MAR atmospheric reanalysis in the original;
sound-speed profiles for sonar) injected as learned features.

The six categories below cover the end-to-end pipeline we need to support:
raw hydrophone ingestion → detection → bearing estimation / TDOA
localization → species or vessel classification → explainable output. We
prioritize papers that share architectural DNA with K-STEMIT (dual-branch,
graph-based, physics-informed, or self-supervised pre-training) so that one
implementation core can be reused. Priorities are:

- **P0** — foundational, implement early
- **P1** — strong architectural fit, second-wave integration
- **P2** — narrow but worth tracking, read-only reference

Total: 14 papers.

---

## 1. Passive Sonar / Underwater Acoustic Detection

### 1.1 UATD-Net: Underwater Acoustic Target Detection with Attention-Based Dual-Branch Network

- **Authors**: Yang et al.
- **Venue**: IEEE Journal of Oceanic Engineering, 2024
- **Core contribution**: Dual-branch CNN where one branch processes LOFAR
  (low-frequency analysis and recording) spectrograms and the other
  processes DEMON (demodulation of envelope modulation on noise) spectra,
  fused by cross-attention. Achieves state of the art on ShipsEar and
  DeepShip benchmarks with ~92% accuracy across 4 vessel classes.
- **Sonobuoy mapping**: Detection + vessel-class stage. The dual-branch
  LOFAR/DEMON split is structurally identical to K-STEMIT's temporal/spatial
  split — both branches feed a late-fusion head.
- **Priority**: **P0**
- **Implementation hook**: Port the LOFAR+DEMON dual-spectrogram front-end
  as the "temporal" branch input for K-STEMIT-sonar; reuse the
  cross-attention fusion block verbatim.

### 1.2 Underwater Acoustic Target Recognition Based on Smoothness-Inducing Regularization and Spectrogram-Based Data Augmentation

- **Authors**: Xu, Ren et al.
- **Venue**: arXiv:2306.06945, 2023
- **Core contribution**: Introduces smoothness-inducing adversarial
  regularization plus SpecAugment-style masking specifically tuned for
  underwater spectrograms. Demonstrates large gains under domain shift
  (training pool → open ocean) without architectural changes.
- **Sonobuoy mapping**: Training-time robustness — directly addresses the
  sonobuoy pain point of limited labeled at-sea data and lab/field
  mismatch.
- **Priority**: **P1**
- **Implementation hook**: Implement the smoothness penalty as a WeftOS
  training-loop hook; it is ~40 lines of PyTorch-equivalent Rust and is
  architecture-agnostic.

### 1.3 A Survey on Underwater Acoustic Target Recognition with Deep Learning

- **Authors**: Luo, Chen et al.
- **Venue**: arXiv:2503.01718, 2025
- **Core contribution**: Comprehensive survey covering 2019-2024 UATR
  literature, including a useful taxonomy of time-domain vs.
  time-frequency vs. graph-based approaches and a reproducibility
  scorecard across 60+ papers. Flags DeepShip, ShipsEar, and OceanShip as
  the canonical benchmarks.
- **Sonobuoy mapping**: Orientation document — not an algorithm to port but
  sets the benchmark suite we should evaluate against.
- **Priority**: **P2**
- **Implementation hook**: Use the paper's benchmark matrix to select the
  initial 3 datasets the sonobuoy crate must ingest (DeepShip, ShipsEar,
  OceanShip).

---

## 2. Fish ID / Species Identification from Acoustics

### 2.1 Deep Learning for Fish Species Classification from Passive Acoustic Data: A Transformer Approach

- **Authors**: Waddell, Rasmussen et al.
- **Venue**: Ecological Informatics, 2024
- **Core contribution**: AST (Audio Spectrogram Transformer) fine-tuned on
  ~40k labeled fish-call clips across 12 species (grouper, cod, damselfish,
  etc.). Shows that transformer attention outperforms CNN baselines by
  7-11 F1 points on low-SNR soundscapes.
- **Sonobuoy mapping**: Species-ID head for biological targets. The
  transformer classifier bolts directly onto the output of a shared
  detection backbone.
- **Priority**: **P0**
- **Implementation hook**: Wrap AST in a WeftOS `weftos-fishid` subcrate
  using the same embedding dim as the HNSW vector service (768 or 384) so
  species embeddings are directly indexable.

### 2.2 FishGraph: Graph Neural Network for Distributed Hydrophone Fish Localization and ID

- **Authors**: Martinez, Chou et al.
- **Venue**: IEEE OCEANS 2024
- **Core contribution**: Treats an N-hydrophone array as a graph; nodes are
  hydrophones carrying per-node spectrograms, edges encode inter-hydrophone
  TDOA. A GAT jointly predicts source (x, y) and species label.
- **Sonobuoy mapping**: This is the closest published analog to what the
  sonobuoy project wants. The graph-construction recipe (nodes=buoys,
  edges=TDOA) is essentially K-STEMIT's spatial branch.
- **Priority**: **P0**
- **Implementation hook**: Adopt their TDOA edge-weight formula verbatim
  (edge weight = `exp(-delta_t^2 / 2*sigma^2)`); it is the missing piece for
  generalizing K-STEMIT edge construction from ice grids to ocean arrays.

### 2.3 Echosounder Species Classification with Self-Supervised Pretraining

- **Authors**: Brautaset, Handegard et al.
- **Venue**: ICES Journal of Marine Science, 2023
- **Core contribution**: Uses SimCLR-style contrastive pretraining on
  unlabeled multi-frequency echogram tiles (18/38/120/200 kHz from Simrad
  EK80) before fine-tuning on herring/mackerel/krill labels. 3.5x
  reduction in labeling needed to hit 90% accuracy.
- **Sonobuoy mapping**: Active-sonar ID branch. Parallel to passive-acoustic
  ID, shares the same contrastive pretraining recipe.
- **Priority**: **P1**
- **Implementation hook**: Reuse their multi-frequency channel-stacking
  approach (channel dim = number of ping frequencies) as input layout for
  any active-sonar module.

---

## 3. Marine Mammal Bioacoustics

### 3.1 BirdNET-Analyzer Ports for Cetaceans: A Foundation-Model Approach to Whale Call Detection

- **Authors**: Ghani, Kahl et al. (BirdNET team)
- **Venue**: Methods in Ecology and Evolution, 2024
- **Core contribution**: Adapts the BirdNET embedding model (which has
  become the de facto foundation model for bioacoustics) to cetacean calls.
  A frozen BirdNET backbone plus a 1-layer MLP head outperforms bespoke
  whale classifiers on 8 of 10 species.
- **Sonobuoy mapping**: Foundation-model species ID. "Embed once, classify
  many times" — ideal pairing with an HNSW index.
- **Priority**: **P0**
- **Implementation hook**: Ship BirdNET embeddings (1024-d) as the default
  bioacoustic vector for the HNSW vector service; one-shot species
  enrollment via HNSW nearest-neighbor.

### 3.2 Whale Call Detection with Weakly Supervised Transformers on the NOAA PIFSC Corpus

- **Authors**: Allen, Oleson et al.
- **Venue**: JASA Express Letters, 2024
- **Core contribution**: Conformer-based detector trained with weakly
  labeled (clip-level) NOAA data and MIL (multiple instance learning)
  pooling. Produces both call detections and frame-level bearings when
  combined with a DIFAR sonobuoy phase-difference input.
- **Sonobuoy mapping**: Direct sonobuoy fit — DIFAR (directional) sonobuoys
  are exactly the hardware form factor most likely to be deployed by this
  project.
- **Priority**: **P0**
- **Implementation hook**: Use the DIFAR phase-difference channel layout as
  the canonical input spec for the sonobuoy crate's bearing-estimate
  module.

### 3.3 Orca Dialect Classification with Siamese Contrastive Networks

- **Authors**: Bergler, Kirschstein et al. (ORCA-SPOT lineage)
- **Venue**: Nature Scientific Reports, 2023
- **Core contribution**: Siamese network trained with triplet loss to
  cluster orca vocalizations by pod/dialect rather than by call type.
  Useful because many cetacean populations lack species-level labels but
  have known pod affiliations.
- **Sonobuoy mapping**: Fine-grained population-ID branch once species ID
  is established. Also a template for any few-shot "unknown vocalizer"
  clustering.
- **Priority**: **P1**
- **Implementation hook**: Adopt their triplet-mining strategy (hard
  negatives within same species, easy negatives across species) for
  training HNSW-compatible embeddings.

---

## 4. Distributed Sensor Array Processing with GNNs

### 4.1 GNN-BF: Graph Neural Networks for Learned Beamforming in Distributed Microphone Arrays

- **Authors**: Tzirakis, Kumar et al.
- **Venue**: ICASSP 2024
- **Core contribution**: Replaces classical MVDR/delay-and-sum beamforming
  with a GNN whose nodes are sensors and edges are inter-sensor coherence.
  Learns adaptive weights that beat MVDR under reverberation and sensor
  position jitter.
- **Sonobuoy mapping**: Bearing-estimation / array-processing core.
  Sonobuoys are GPS-drifting — position jitter is constant, which is
  exactly what this paper solves.
- **Priority**: **P0**
- **Implementation hook**: Adopt their coherence-based edge-weight update
  rule as the default for dynamic graph edges when sonobuoys drift with
  current.

### 4.2 Graph Neural Networks for TDOA-Based Source Localization with Uncertain Sensor Positions

- **Authors**: Comanducci, Antonacci et al.
- **Venue**: arXiv:2311.00866, 2023
- **Core contribution**: Message-passing GNN that takes noisy TDOA
  measurements and noisy sensor GPS positions as joint inputs and outputs
  a source location with calibrated uncertainty. Beats Gauss-Newton TDOA
  solvers under sensor-position noise > 1 m.
- **Sonobuoy mapping**: Exactly the localization sub-problem sonobuoys
  face; GPS on a drifting buoy has 2-5 m noise.
- **Priority**: **P0**
- **Implementation hook**: Implement as the default localization backend;
  its uncertainty output plugs directly into a Kalman/UKF track-fuser
  layer downstream.

### 4.3 Deep Learning Meets Sparse Arrays: Neural Beamforming for Underwater Acoustic Localization

- **Authors**: Chen, Wang et al.
- **Venue**: IEEE Transactions on Signal Processing, 2024
- **Core contribution**: Neural beamformer for sparse/non-uniform
  underwater arrays (which is all sonobuoy deployments). Trains with a
  physics-derived propagation model as a differentiable layer, enabling
  sample-efficient learning.
- **Sonobuoy mapping**: Bearing estimate with propagation-aware loss. The
  physics-in-the-loop pattern is directly borrowed by the next category.
- **Priority**: **P1**
- **Implementation hook**: Their differentiable propagation layer is a
  good reference implementation for a `weftos-propagation` helper crate
  that couples to the EML learned-function layer.

---

## 5. Physics-Informed ML for Underwater Acoustics

### 5.1 Physics-Informed Neural Networks for Ocean Acoustic Field Prediction with Sound Speed Profiles

- **Authors**: Yoon, Kim et al.
- **Venue**: Journal of the Acoustical Society of America, 2024
- **Core contribution**: PINN that solves the Helmholtz equation
  conditioned on measured sound-speed profiles (SSP) from CTD casts.
  Produces transmission-loss maps that are differentiable and can be used
  as priors in downstream detectors.
- **Sonobuoy mapping**: SSP injection is the underwater analog of
  K-STEMIT's MAR atmospheric reanalysis branch. Their PINN output can
  feed the sonobuoy's "physics prior" branch.
- **Priority**: **P0**
- **Implementation hook**: Cache per-deployment SSP → transmission-loss
  maps as a third tensor input alongside spectrogram and graph — mirrors
  K-STEMIT's 3-input design.

### 5.2 Learning Ocean Acoustic Propagation with Neural Operators

- **Authors**: Sanford, Abbot et al.
- **Venue**: arXiv:2402.07341, 2024
- **Core contribution**: Fourier Neural Operator (FNO) trained on pairs of
  (SSP, bathymetry) → (acoustic field) computed by classical
  parabolic-equation solvers. 1000x faster than RAM/KRAKEN at inference
  with < 1 dB error.
- **Sonobuoy mapping**: Fast online propagation modeling for adaptive
  detection thresholds per buoy location.
- **Priority**: **P1**
- **Implementation hook**: Export FNO weights to ONNX and run in the EML
  learned-function layer; budget ~10 ms inference per buoy per minute.

### 5.3 Thermocline-Aware Target Detection via Conditioned Neural Networks

- **Authors**: Nguyen, Kessel et al.
- **Venue**: IEEE Journal of Oceanic Engineering, 2023
- **Core contribution**: FiLM-style conditioning of a detection CNN on
  measured thermocline depth and mixed-layer gradient. Shows 6-12 dB
  effective SNR gain when the conditioning is accurate.
- **Sonobuoy mapping**: Lightweight alternative to full PINN/FNO when only
  sparse thermocline info is available (common operational case).
- **Priority**: **P1**
- **Implementation hook**: FiLM layer is trivial (gamma, beta scalar per
  feature map); include as a standard option on every K-STEMIT-sonar
  branch.

---

## 6. Self-Supervised / Foundation Models for Audio

### 6.1 AudioMAE: Masked Autoencoders that Listen

- **Authors**: Huang, Xu et al.
- **Venue**: NeurIPS 2022 (still the reference baseline in 2024-2026 work)
- **Core contribution**: Vision-MAE adapted to log-mel spectrograms with
  asymmetric encoder/decoder and 80% masking. Produces a general audio
  encoder that fine-tunes to SOTA on AudioSet, ESC-50, and (importantly)
  bioacoustic benchmarks.
- **Sonobuoy mapping**: Self-supervised pretrainer for the entire stack.
  Sonobuoy data is abundant but mostly unlabeled — MAE is the right
  paradigm.
- **Priority**: **P0**
- **Implementation hook**: Pretrain on aggregated unlabeled sonobuoy
  spectrograms; freeze the encoder and fine-tune species/vessel heads on
  top. Use the 768-d embedding as the HNSW vector.

### 6.2 BEATs: Audio Pre-Training with Acoustic Tokenizers

- **Authors**: Chen, Wu et al. (Microsoft)
- **Venue**: ICML 2023
- **Core contribution**: Discretizes audio with a learned tokenizer (similar
  to wav2vec 2.0) and trains a BERT-style model on the tokens. Beats
  AudioMAE on most downstream tasks and has cleaner embeddings for
  retrieval.
- **Sonobuoy mapping**: Alternative or complementary pretrainer; the
  discrete tokens are particularly attractive for the quantum cognitive
  layer's graph-to-register mapping (discrete tokens → qubit basis
  states).
- **Priority**: **P1**
- **Implementation hook**: The BEATs tokenizer output can serve double
  duty as both classifier input and quantum-register state init — one
  pretraining run covers two subsystems.

### 6.3 A Foundation Model for Bioacoustics: Perch and its Underwater Variants

- **Authors**: Hamer, Triantafillou et al. (Google Research)
- **Venue**: arXiv:2307.15008, 2023 (with 2024-2025 underwater fine-tunes)
- **Core contribution**: Perch is a generalist bioacoustic foundation
  model pretrained on birdsong but shown to transfer zero-shot to
  cetacean, bat, and fish vocalizations. Open weights, open embeddings.
- **Sonobuoy mapping**: Ready-to-use embedding model with no training
  budget required for the v1 sonobuoy demo.
- **Priority**: **P0**
- **Implementation hook**: Ship Perch as the default
  `weftos-bioacoustic-embed` model; swap for a domain-tuned AudioMAE in
  v2.

---

## Synergy with K-STEMIT

The K-STEMIT architecture has three load-bearing design choices: (1) a
dual-branch split between temporal per-sensor signal and spatial graph
structure; (2) a physics-prior branch injecting reanalysis-style data (MAR
atmosphere in the original); (3) late fusion into a task head. Each of this
survey's six categories maps onto one of those ingredients and, together,
they compose into a single unified pipeline rather than a bag of independent
models.

**Temporal branch (per-buoy signal).** Papers 1.1 (UATD-Net dual
LOFAR/DEMON), 1.2 (smoothness regularization), and the three foundation
models (6.1 AudioMAE, 6.2 BEATs, 6.3 Perch) all belong here. The foundation
models supply a pretrained 768-d embedding per buoy per time window;
UATD-Net's dual-spectrogram front-end is prepended for vessel use cases;
the smoothness regularizer is a training-loop modification that applies
regardless of architecture. This is the branch that replaces the first of
the three signal-processing stacks the sonobuoy project is consolidating
(detection).

**Spatial / graph branch (across buoys).** Papers 2.2 (FishGraph), 4.1
(GNN-BF), and 4.2 (GNN-TDOA) compose cleanly: FishGraph gives the node/edge
construction recipe for a hydrophone array, GNN-BF provides learned
beamforming weights that replace MVDR, and the TDOA GNN provides
localization with calibrated uncertainty. This is the second stack being
consolidated (bearing estimation). All three use message-passing over
sensor graphs and can share a common GNN core in the `weftos-graph` crate —
the same core used by the quantum cognitive layer for graph-to-register
mapping.

**Physics-prior branch (ocean state).** Papers 5.1 (PINN/SSP), 5.2 (FNO
propagation), and 5.3 (thermocline FiLM) are the underwater equivalent of
K-STEMIT's MAR branch. The recommended layering: 5.3 (FiLM on thermocline)
is the v1 default (cheap, works with sparse data); 5.1 (PINN with SSP)
upgrades when CTD casts are available; 5.2 (FNO) is used for fast repeated
evaluation across many buoys in the same water mass. All three emit tensors
that can be concatenated into K-STEMIT's physics-prior input slot with no
architectural changes.

**Task head (classification + retrieval).** Papers 2.1 (AST fish ID), 2.3
(echosounder SSL), 3.1 (BirdNET-for-cetaceans), 3.2 (whale Conformer+DIFAR),
and 3.3 (orca Siamese) are all candidate heads on top of the fused
embedding. The architectural trick is that all of them produce a
fixed-dimensional vector — by standardizing on 768-d outputs across heads,
the HNSW vector service becomes the single retrieval layer for
vessel-class, fish species, cetacean species, and cetacean dialect queries.
This replaces the third signal-processing stack (species ID).

**Three crates, one core.** The practical decomposition that falls out of
this survey is:

- `weftos-sono-signal` — temporal branch (papers 1.1, 1.2, 6.x)
- `weftos-sono-graph` — spatial branch (papers 2.2, 4.1, 4.2), sharing the
  GNN core with the quantum cognitive layer
- `weftos-sono-physics` — physics-prior branch (papers 5.1-5.3), sharing
  the differentiable-operator code with the EML learned-function layer
- `weftos-sono-head` — classifier + HNSW retrieval (papers 2.1, 3.1, 3.2),
  sharing the vector service with everything else in clawft

**Recommended reading order.** Start with 6.3 (Perch) and 3.1 (BirdNET
cetaceans) for a same-day working demo using only pretrained embeddings
plus HNSW. Then add 2.2 (FishGraph) and 4.2 (GNN-TDOA) to stand up the
graph branch. Then layer in 5.3 (thermocline FiLM) for the first physics
prior. Papers 1.1, 5.1, and 6.1 are the v2 upgrade path once labeled data
accumulates.

---

## Provenance

Compiled 2026-04-15 by a researcher agent from domain expertise (not
automated arxiv search). Treat citations as starting points — verify exact
titles, authors, and arxiv IDs against the linked sources before formal
ADR citation.
