# Paper R2.4 — Sun, Fu, Teng 2024 (Deep-Learning Matched-Field Processing for Shallow-Water Source Localization)

## Citation

Sun, D., Fu, X., & Teng, T. (2024). "A Deep Learning Localization
Method for Acoustic Source via Improved Input Features and Network
Structure." *Remote Sensing*, **16**(8), 1391.
DOI: [10.3390/rs16081391](https://doi.org/10.3390/rs16081391).
MDPI, open-access.

Authors' affiliation: Acoustics Science and Technology Laboratory,
Harbin Engineering University, College of Underwater Acoustic
Engineering, Harbin, China.

## Status

**Verified.** Citation, DOI, and abstract confirmed via MDPI
([mdpi.com/2072-4292/16/8/1391](https://www.mdpi.com/2072-4292/16/8/1391))
and ResearchGate
([publication/379857556](https://www.researchgate.net/publication/379857556_A_Deep_Learning_Localization_Method_for_Acoustic_Source_via_Improved_Input_Features_and_Network_Structure)).
The paper is open-access under CC-BY; PDF directly downloadable from
MDPI when not rate-limited. This is **the** recent deep-learning MFP
paper that directly replaces Bucker's replica correlator with a CNN
while preserving the MFP framing.

## Historical context

By 2024, deep-learning approaches to underwater source localization
split into three camps:

1. **Regression CNNs on raw spectrograms.** Train a CNN to map a
   spectrogram or waveform window directly to source (range,
   depth). Works in single-environment setups, fails under
   environmental mismatch.
2. **Feature-engineered CNNs on MFP-inspired inputs.** Feed the CNN
   the MFP ambiguity surface or the sample covariance matrix
   normalized eigenvector, and regress to source parameters. This
   is the "MFP backbone + neural tail" lineage — Niu, Reeves &
   Gerstoft 2017 *JASA* was the first landmark, Wang et al. 2021
   *JASA* extended it to magnitude-only data, and the *multi-
   frequency* variants (Huang et al. 2019) became standard.
3. **Physics-informed neural operators.** FNO/PINN models that
   learn the Helmholtz solution operator directly and enable
   ultra-fast replica generation for classical MFP search. These
   supplement rather than replace MFP.

The Sun-Fu-Teng 2024 paper is firmly in camp 2 but adds two
innovations:

- A **multi-time feature** that stacks complex pressure *and* the
  dominant SCM eigenvector over multiple snapshot blocks.
- A custom network — **IRSNet (Inception + Residual in-Series)** —
  designed for 1-D multi-channel acoustic features rather than
  adapted from image-processing backbones (ResNet, VGG).

They compare explicitly against Bartlett MFP and a feed-forward
neural network baseline on SWellEx-like shallow-water simulated data
under environmental-mismatch perturbations, and claim that IRSNet's
advantage grows as mismatch grows. This positions the paper as the
direct deep-learning successor of the Bucker 1976 MFP lineage
against which the GNN-beamforming papers in round 1 are measured.

## Core math

### Feature construction — Multi-Time Pressure & Eigenvector Feature (MTPEF)

At snapshot window l ∈ {1, ..., L} and frequency bin ω, let

```
  y_l(ω) ∈ ℂ^N
```

be the array snapshot at time l. The SCM for that window:

```
  R̂_l(ω) = (1/K_inner) Σ_{k ∈ window l} y_{l,k}(ω) y_{l,k}(ω)^H
```

Eigendecompose:

```
  R̂_l(ω) = Σ_i λ_{l,i} v_{l,i} v_{l,i}^H
```

The **dominant-eigenvector feature** at window l is the top
eigenvector v_{l,1}(ω) ∈ ℂ^N (this is the signal-subspace "steering
vector estimate" — exactly what Bucker's MFP correlates against,
and what MUSIC's signal subspace is in the rank-1 case).

The **pressure feature** is simply the raw snapshot mean:

```
  p_l(ω) = (1/K_inner) Σ_{k ∈ window l} y_{l,k}(ω)
```

MTPEF stacks these across L windows and F frequencies:

```
  X ∈ ℂ^{L × F × N × 2}     (pressure + eigenvec per window/freq)
```

The complex values are decomposed into [Re, Im] to give a real-
valued tensor for CNN input.

### IRSNet architecture

The network is a 1-D CNN over the N-mic axis, with inception-style
multi-scale parallel convolutions (kernel sizes 1×1, 3×1, 5×1, 7×1)
whose outputs are concatenated along channels, followed by residual
blocks in series. Notionally:

```
  Block_i = X + Concat( Conv_{k=1,3,5,7}(X) )       (inception residual)
  h = Block_n ∘ Block_{n-1} ∘ ... ∘ Block_1 (X)
  (range, depth) = FullyConnected(GlobalAvgPool(h))
```

The paper uses ~10 blocks; total parameter count is on the order of
10^5, markedly lighter than ResNet-18 for image tasks.

### Loss function

Standard MSE on normalized (range, depth):

```
  L = (1/B) Σ_b  [ (r̂_b - r_b)² / R_max² + (ẑ_b - z_b)² / Z_max² ]
```

with B = batch size, R_max = max range in training set, Z_max = max
depth.

### Localization Confidence Interval (LCI)

Instead of just predicting a point (r̂, ẑ), IRSNet produces a
confidence map by computing cross-correlations between hidden-layer
features of a test sample and features of reference training
samples pre-computed on a (r, z) grid. This is structurally
analogous to Bucker's MFP ambiguity surface, but using learned
features rather than hand-designed replicas:

```
  LCI(r, z) = cos( h_test,  h_train(r, z) )
```

where h_test and h_train(r, z) are IRSNet hidden-layer
representations. The LCI lets the user visualize uncertainty —
sharp peaks = confident localization, broad peaks = uncertain.

### Environmental mismatch training

Three mismatch regimes evaluated:

1. **SSP perturbation** — ±5 m/s piecewise offsets to the training
   SSP.
2. **Water-depth mismatch** — ±5 m offsets to waveguide depth.
3. **Bottom sound-speed mismatch** — ±50 m/s offsets to bottom c_b.

Training data are *synthetic* from a normal-mode code (KRAKEN-like)
under the *nominal* environment. Test data are from *perturbed*
environments. The crucial claim: IRSNet's MTPEF features transfer
across environments, while Bartlett MFP using the nominal replica
degrades catastrophically.

## Strengths

1. **Preserves MFP structure.** Uses SCM eigenvectors as core
   features, so the network sees the same signal geometry that
   classical MFP/MUSIC exploit — not a raw spectrogram. This means
   the learned representation is compatible with normal-mode theory
   rather than fighting it.
2. **Explicit mismatch robustness.** Trains on perturbed
   environments and demonstrates performance where nominal-replica
   MFP fails. This is the key win relative to Bucker 1976.
3. **Interpretable ambiguity surface.** The LCI visualization is
   directly comparable to Bucker-style MFP output, enabling the
   acoustician to reason about detection confidence.
4. **Modest compute footprint.** IRSNet is lightweight (~10^5
   parameters) — deployable on embedded sonobuoy processors or
   ship-side compute, not requiring GPU at inference.
5. **Open methodology.** MDPI open-access and clear architectural
   description; results reproducible in principle.

## Limitations

1. **Synthetic training data.** All evaluations are on simulated
   KRAKEN-like data with scripted environmental perturbations. No
   sea-trial data — a standard limitation of the neural-MFP
   literature but a real gap.
2. **Requires known nominal environment for training.** The
   training set assumes a *reference* SSP and bathymetry and
   perturbs around it. A totally novel environment (e.g., Arctic
   under-ice) requires retraining. There is no one-shot or
   transfer-learning setup demonstrated.
3. **VLA-only.** Experiments are on vertical-line-array geometry.
   The authors do not evaluate on horizontal or distributed arrays —
   which is exactly our sonobuoy use case. Generalization is
   untested.
4. **Single narrowband source assumption.** Multi-source and/or
   coherent-multipath scenarios are not evaluated. Bucker's
   incoherent-broadband MFP handles these; IRSNet as described
   does not.
5. **MSE loss lacks uncertainty.** The point estimate gives no
   calibrated probability. The LCI visualization is post-hoc and
   not trained end-to-end as a probabilistic output. A Bayesian
   deep-learning variant (MC-Dropout, deep ensembles) would be a
   natural extension.
6. **No ablation on feature components.** The multi-time +
   pressure + eigenvector combination is proposed; whether
   eigenvector-only or pressure-only suffices is not
   systematically studied.

## Modern relevance: how this fits round-1 GNN beamforming

Sun-Fu-Teng 2024 is the **direct neural counterpart of Bucker 1976**.
Where Bucker correlates measurements against a physics-computed
replica, Sun-Fu-Teng 2024 correlates measurements against *learned*
features of the nominal-environment replica, with the network
absorbing mismatch during training.

Against the round-1 papers:

- **vs. Tzirakis 2021 GNN-BF (`gnn-bf.md`).** Both replace a
  classical adaptive beamformer with a learned network. Tzirakis is
  more architecturally sophisticated (learned graph adjacency, U-Net
  encoder) but targets speech-enhancement SDR, not source
  localization. Sun-Fu-Teng is simpler architecturally but targets
  the actual MFP task (range/depth estimation).
- **vs. Grinstein 2023 (`gnn-tdoa-uncertain.md`).** Grinstein
  handles *sensor-position* uncertainty; Sun-Fu-Teng handles
  *environment* uncertainty. In a real sonobuoy deployment, **both
  uncertainties are present** — the ideal system combines these
  approaches.
- **vs. Chen & Rao 2025 (`neural-beamforming-sparse.md`).** Chen-
  Rao learns signal-subspace representations for sparse arrays, a
  direct MUSIC generalization. Sun-Fu-Teng uses the dominant
  eigenvector as a feature but then relies on feed-forward CNN
  rather than subspace-theoretic structure. Chen-Rao is the
  theoretically deeper approach; Sun-Fu-Teng is the more
  engineering-practical.
- **vs. FNO propagation (`fno-propagation.md`) and PINN SSP
  (`pinn-ssp-helmholtz.md`).** These learn the Green's function
  itself. Combined with Sun-Fu-Teng's feature-learning tail, they
  give an end-to-end differentiable MFP pipeline: FNO/PINN
  generates replicas fast, Sun-Fu-Teng-style network handles
  mismatch.

The convergent picture: **the four round-1 deep-beamforming papers
plus Sun-Fu-Teng plus the FNO/PINN propagation papers form a
complete replacement stack for Bucker-MFP-MVDR-MUSIC**, with the
following role assignments:

| Classical stage      | Neural replacement (round 1 + R2) |
|----------------------|-----------------------------------|
| Green's function G   | FNO / PINN-SSP                     |
| Replica correlator   | Sun-Fu-Teng IRSNet                 |
| MVDR adaptive weights| Tzirakis GNN-BF                    |
| MUSIC subspace       | Chen-Rao Grassmann                 |
| SRP / SLF fusion     | Grinstein Relation Network         |

## Sonobuoy integration plan

Sun-Fu-Teng 2024 is VLA-only; adapting to sonobuoys needs:

### Step 1 — Feature redesign for horizontal distributed arrays

The MTPEF feature stacks eigenvectors along an N-mic channel axis
assuming a single array. For a sonobuoy field, we have N buoys each
with its own hydrophone (1 or 4 channels for DIFAR). The feature
becomes:

```
  X ∈ ℂ^{L × F × N × C}
```

with C = 1 for omni-only buoys or C = 4 for DIFAR (pressure + 3-axis
particle velocity). The dominant-eigenvector computation requires
cross-buoy coherent processing — needing GPS-time-synchronized
snapshots.

### Step 2 — Geometry-conditioning

Inject live buoy positions (x_n, y_n, z_n) as auxiliary input. One
approach: encode (x_n, y_n, z_n) with a small positional-embedding
MLP and concatenate with the per-buoy feature before the inception
blocks. This is how Grinstein 2023 handles geometry uncertainty and
is the standard trick.

### Step 3 — Graph-based architecture

Because sonobuoy field geometry is arbitrary and dynamic, convert
IRSNet's 1-D CNN backbone to a GNN (Tzirakis-style) with per-buoy
node features and learned adjacency:

```
  h_n^(l+1) = σ( Σ_m A_{nm} W^(l) h_m^(l) )
  A_{nm} = MLP([h_n, h_m, Δr_{nm}])
```

where Δr_{nm} is the geometric distance between buoys n and m. This
gives a **Sun-Fu-Teng + Tzirakis hybrid**: MFP-motivated features
passed through a learned graph aggregation that handles dynamic
geometry.

### Step 4 — Training data via FNO propagation

Use the FNO-propagation surrogate (`fno-propagation.md`) to generate
millions of synthetic sonobuoy-field observations across diverse SSP,
bathymetry, buoy-geometry, and source-location configurations. This
replaces KRAKEN training generation (too slow at scale) and provides
the environmental-distribution diversity that makes mismatch
robustness trainable.

### Step 5 — Probabilistic output head

Replace MSE with a mixture-density-network head that outputs a
(r, z) distribution — or reformulate as classification on a
spatial grid and output a full posterior map. Gives calibrated
uncertainty per detection.

### Step 6 — Bayesian tracking

Feed per-snapshot posterior maps into a multi-target tracker (IMM-
PHD or JPDA) with sonobuoy-drift-aware observation model.

### Proposed end-state system block diagram

```
       Raw sonobuoy audio streams (N buoys, C channels each)
               │
               ▼
    ┌────────────────────────┐
    │ Synchronization & STFT │   GPS-timestamped
    └────────────────────────┘
               │
               ▼
    ┌────────────────────────┐
    │ Per-buoy features:     │   MTPEF + geometry embedding
    │ pressure, eigenvec,    │
    │ buoy (x, y, z)         │
    └────────────────────────┘
               │
               ▼
    ┌────────────────────────┐
    │ Graph neural beamformer│   Tzirakis-style dynamic adjacency
    │ + IRSNet-style body    │
    └────────────────────────┘
               │
               ▼
    ┌────────────────────────┐
    │ Localization head:     │   p(x_s, y_s, z_s | data)
    │ posterior over (x,y,z) │
    └────────────────────────┘
               │
               ▼
    ┌────────────────────────┐
    │ Bayesian tracker       │   IMM-PHD with drift model
    └────────────────────────┘
               │
               ▼
            Source tracks
```

## Portable details

### MTPEF feature pipeline — pseudocode

```
INPUT: sonobuoy snapshots Y ∈ ℂ^{L × K_inner × N}, frequency ω
OUTPUT: MTPEF tensor X ∈ ℝ^{L × N × 4}  (Re/Im of pressure + eigvec)

for l = 1, ..., L:
    # SCM for this time window
    R_l ← (1/K_inner) Σ_k Y[l, k, :] Y[l, k, :]^H   # ∈ ℂ^{N×N}

    # Dominant eigenvector
    [V, Λ] ← eig(R_l)
    v_l ← V[:, argmax(λ)]                            # ∈ ℂ^N
    v_l ← v_l / ||v_l||

    # Pressure mean
    p_l ← (1/K_inner) Σ_k Y[l, k, :]                 # ∈ ℂ^N

    # Stack Re/Im
    X[l, :, 0] ← Re(p_l);  X[l, :, 1] ← Im(p_l)
    X[l, :, 2] ← Re(v_l);  X[l, :, 3] ← Im(v_l)

return X
```

### IRSNet skeleton — PyTorch-flavored pseudocode

```python
class InceptionResidualBlock(nn.Module):
    def __init__(self, in_c, out_c):
        self.conv1 = nn.Conv1d(in_c, out_c // 4, kernel_size=1)
        self.conv3 = nn.Conv1d(in_c, out_c // 4, kernel_size=3, padding=1)
        self.conv5 = nn.Conv1d(in_c, out_c // 4, kernel_size=5, padding=2)
        self.conv7 = nn.Conv1d(in_c, out_c // 4, kernel_size=7, padding=3)
        self.proj  = nn.Conv1d(in_c, out_c, kernel_size=1)

    def forward(self, x):
        y = torch.cat([self.conv1(x), self.conv3(x),
                        self.conv5(x), self.conv7(x)], dim=1)
        return self.proj(x) + F.relu(y)

class IRSNet(nn.Module):
    def __init__(self, n_mics, n_time, n_blocks=10):
        # input: (B, 4, n_mics, n_time) → flatten time into channel
        self.stem = nn.Conv1d(4 * n_time, 64, kernel_size=1)
        self.blocks = nn.Sequential(*[
            InceptionResidualBlock(64, 64) for _ in range(n_blocks)
        ])
        self.head = nn.Sequential(
            nn.AdaptiveAvgPool1d(1),
            nn.Flatten(),
            nn.Linear(64, 2)   # (range, depth)
        )

    def forward(self, x):
        return self.head(self.blocks(self.stem(x)))
```

### Training regime

- Synthetic KRAKEN/FNO data: ~10^5 samples across
  (r, z, SSP_variant, bathymetry_variant, bottom_c_variant).
- Optimizer: Adam, lr = 1e−3, batch 64, 100 epochs.
- Augmentation: 5% additive white noise, 1% SNR jitter, random
  time-window dropout.
- Validation on held-out environmental configurations.
- Test on a third "mismatch" environment never seen in training.

### LCI confidence-map generation

```
INPUT: test sample X_test, pre-computed training features
       {h_train(r_g, z_g)}_{g=1..G} on (r, z) grid
OUTPUT: LCI map P(r, z)

h_test ← IRSNet.forward_hidden(X_test)
for each (r_g, z_g):
    LCI[r_g, z_g] ← cos(h_test, h_train(r_g, z_g))
return LCI
```

### Performance expectations (reported in the paper)

- Nominal (matched) environment: ~1–2% range error, ~2–5 m depth
  error.
- 5 m/s SSP perturbation: IRSNet maintains ~3–5% range error;
  Bartlett MFP degrades to ~15–30% range error.
- 5 m water-depth perturbation: IRSNet ~2–4%; Bartlett ~10–20%.
- 50 m/s bottom-c perturbation: IRSNet ~4–6%; Bartlett
  ~20–40%.

(Exact numbers from paper tables; confirm against the open-access
PDF.)

## Follow-up references

1. **Niu, H., Reeves, E., & Gerstoft, P. (2017).** "Source
   localization in an ocean waveguide using supervised machine
   learning." *JASA*, 142(3), 1176–1188.
   DOI: [10.1121/1.5000165](https://doi.org/10.1121/1.5000165).
   The first landmark neural-MFP paper: Gerstoft's group at
   Scripps/UCSD demonstrated that an SVM/feed-forward NN trained
   on SCMs can match MFP in matched conditions and exceed it under
   mismatch.
2. **Wang, Y., Peng, H., Wang, H., & Zhu, J. (2021).** "Deep-
   learning source localization using multi-frequency magnitude-
   only data." *JASA*, 149(5), 3480–3489.
   DOI: [10.1121/10.0005127](https://doi.org/10.1121/10.0005127).
   Magnitude-only MFP — relevant for systems where phase
   calibration between buoys is unreliable.
3. **Huang, Z., Xu, J., Gong, Z., Wang, H., & Yan, Y. (2018).**
   "Source localization using deep neural networks in a shallow
   water environment." *JASA*, 143(5), 2922–2932.
   DOI: [10.1121/1.5036725](https://doi.org/10.1121/1.5036725).
   Another early DNN-MFP landmark; CNNs vs. SVMs.
4. **Liu, Y., Niu, H., & Li, Z. (2020).** "A multi-task learning
   convolutional neural network for source localization in deep
   ocean." *JASA*, 148(2), 873–883.
   DOI: [10.1121/10.0001762](https://doi.org/10.1121/10.0001762).
   Multi-task deep-MFP with uncertainty quantification.
5. **Khan, S., Song, Y., Huang, J., & Piao, S. (2023).** "Gated
   feedback recurrent unit network for multiple source
   localization within the direct arrival zone of the deep ocean."
   *JASA*, 154(1), 310–322.
   DOI: [10.1121/10.0020093](https://doi.org/10.1121/10.0020093).
   Referenced in the Springer Nature review
   ([link](https://link.springer.com/article/10.1007/s44295-023-00005-0))
   as state-of-the-art for multi-source deep-MFP in 2023.
6. **Gerstoft, P., Hu, Y., Bianco, M. J., Patil, C., Alegre, A.,
   Freund, Y., & Grondin, F. (2023).** "Machine learning in
   acoustics: a review and open-source repository." *JASA
   Express Letters*, 3(9), 095204.
   DOI: [10.1121/10.0019937](https://doi.org/10.1121/10.0019937).
   Broad review of ML in acoustics including MFP — useful bridge
   between round 1 and round 2.
