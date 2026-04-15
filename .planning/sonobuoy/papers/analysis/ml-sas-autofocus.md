# Paper 5.3 — Gerg & Monga 2021, "Real-Time, Deep Synthetic Aperture Sonar (SAS) Autofocus"

## Citation

Gerg, I. D., & Monga, V. (2021). "Real-Time, Deep Synthetic Aperture
Sonar (SAS) Autofocus." *Proc. IEEE International Geoscience and
Remote Sensing Symposium (IGARSS 2021)*, Brussels, Belgium, pp.
3205–3208.

- **arXiv:** [2103.10312](https://arxiv.org/abs/2103.10312) (v3, 1 Jun 2021)
- **IEEE Xplore:** https://ieeexplore.ieee.org/document/9554141/
- **PDF:** https://arxiv.org/pdf/2103.10312

**Funding:** ONR grants N00014-19-1-2638, N00014-19-1-2513. Data:
Naval Surface Warfare Center — Panama City Division (NSWC-PC). Code
reference (author's sibling repo): https://github.com/isaacgerg/synthetic_aperture_sonar_autofocus.

**Note on predecessor:** arXiv:2010.15687 v2 ("Deep Autofocus for
SAS") was withdrawn by the authors in 2021 and superseded by this
IGARSS 2021 paper. Some earlier literature (including the SurveyPaper
Round 1 synthesis if SAS had been included) may cite 2010.15687; that
ID is **withdrawn**, and 2103.10312 is the canonical reference.

## Status

**Verified.** Cited paper downloaded (2.5 MB, 4 pages), parsed with
PyMuPDF, full content extracted (abstract, architecture diagram,
equations, results tables, references). Author affiliations (Penn
State ARL + Penn State EECS), ONR funding, NSWC-PC data, IGARSS 2021
venue, and arXiv record all cross-verified. Author Isaac D. Gerg has
a companion PhD dissertation (2022) and a 2024 journal extension
("Deep Adaptive Phase Learning" IEEE TGRS 2024, DOI
10.1109/TGRS.2024.3385010) that refines this work; we analyze the
IGARSS 2021 version because it is the canonical first CNN-autofocus
SAS paper with an open arXiv copy.

PDF: `.planning/sonobuoy/papers/pdfs/ml-sas-autofocus.pdf` (2.5 MB, 4
pages).

## One-paragraph summary

This is the first paper to autofocus synthetic aperture sonar with a
deep neural network in a genuinely end-to-end, self-supervised way. The
classical autofocus pipeline (Callow SPGA, Paper 5.2) iterates on a
hand-crafted sharpness metric with hand-crafted weighting functions;
each image takes dozens of iterations and the weighting function must
be tuned per scene. Gerg & Monga replace the iterative optimizer with a
single forward pass through a DenseNet-121 CNN + MLP that ingests the
dynamic-range-compressed magnitude and phase of the single-look
complex (SLC) image and outputs the coefficients of a 10-degree phase
polynomial; the correction is applied in k-space and the network is
trained end-to-end with a **self-supervised sharpness-improvement
loss** — no ground-truth focused/defocused pairs needed. On 264 test
SLC images from a real NSWC-PC HF-SAS with synthetic 10-degree
polynomial phase corruption, Deep Autofocus beats the four dominant
sharpness metrics (MNS, ME, OSF, SSI) on both PSNR and MS-SSIM, and
runs at **18 ms/image vs 340 ms/image for the iterative baselines — a
~19× speedup**. Crucially, Deep Autofocus does not catastrophically
fail on pathological images (long left tail of the violin plot), which
is the most important practical property for UUV deployment.

## Methodology — Deep Autofocus architecture

### Problem formulation (§ II)

Start from the classical phase-error model:

```
G_e = (e^{iφ} ⊗ 1_T) ⊙ G                            (Eq. 1)
G   = F_u { g }           (along-track 1D FFT)
```

where `g ∈ C^{M×M}` is the focused SLC, `G_e ∈ C^{M×M}` is the
k-space of the defocused observation, `φ ∈ R^M` is the along-aperture
phase error, `⊗` is Kronecker product (row broadcast), `⊙` is
Hadamard product.

Classical sharpness-metric autofocus solves:

```
φ̂ = arg min_φ  −M(|F^{-1}{(e^{-iφ} ⊗ 1_T) ⊙ G_e}|)     (Eq. 2)
```

where `M` is one of the four sharpness metrics in Table I (quoted
verbatim from the paper):

| Ref | Metric | Equation |
|---|---|---|
| Fortune 2001 | Mean Normalized Stddev (MNS) | `M_MNS = stddev(|g|)/mean(|g|)` |
| Zeng 2013 | Minimum Entropy (ME) | `M_ME = Σ_xy |g|² ln|g|²` |
| Schulz 2006 | Optimal Sharpness Function (OSF) | `M_OSF = Σ_xy ln(|g|² + b)` |
| Fienup 2000 | Sum of Squared Intensity (SSI) | `M_SSI = Σ_xy |g|⁴` |

Solved by gradient descent; 10-50 iterations typical. Hand-crafted
weighting function `w` applied to magnitude before sharpness metric
to exclude "bad" regions.

### Deep Autofocus replaces the optimizer (§ III)

Goal: learn a single function `f` with parameters Θ such that

```
ĝ = f(g_e, Θ)                                       (Eq. 5)
```

holds for images drawn from a typical population. Parameters Θ are
solved by

```
arg min_Θ  L(g, f(g_e, Θ))                          (Eq. 6)
```

The function `f` is explicitly:

```
f(g_e, Θ) = |F^{-1}{ (i · exp(h(f_DRC(g_e), arg(g_e), Θ) ⊗ 1_T)) ⊙ G_e }|   (Eq. 7)
```

where `h` is the CNN + MLP, `f_DRC` is dynamic-range compression, and
`arg(·)` extracts phase. The CNN output is a length-8 vector
representing coefficients 2-9 of a **10-degree polynomial phase model**
(coefficients 0 and 1 are discarded — constant shift and linear slope
have no effect on sharpness). The polynomial is expanded into an
M-length phase vector and applied in k-space.

### Self-supervised loss (§ III)

No ground-truth focused image is needed. The loss is the **relative
sharpness improvement** between defocused input and network output:

```
L(g_e, ĝ) = −(M(ĝ) − M(g_e)) / M(g_e)              (Eq. 8)
```

where `M = M_MNS` (mean-normalized stddev) was chosen empirically for
best training convergence. Minimizing L = maximizing relative
sharpness gain. Trained with SGD, learning rate 10⁻¹, mini-batch 32,
10,000 epochs.

### Architecture specifics

- **Feature extractor:** DenseNet-121, initialized from ImageNet
  weights. Input: concat[DRC_magnitude, phase_map] as 2-channel image.
  Output: R^{8×8×1024}.
- **Global average pooling** → R^{1024}.
- **MLP regression head:** 1024 → 512 → 256 → 128 → 64 → 32 → 8,
  LeakyReLU between each. Initialized with Glorot.
- **Polynomial expansion:** 8 coefficients → 10-degree polynomial (skip
  degrees 0,1) → M-length phase vector.
- **k-space application:** FFT → multiply by e^{-iφ̂} → IFFT → magnitude.
- **Fully differentiable end-to-end.** FFT, DRC (Schlick 1995
  rational tone map), polynomial expansion, and Hadamard multiply are
  all differentiable; gradients flow to CNN weights via
  backpropagation through the whole pipeline.

### Dynamic Range Compression (Eq. 9-10, Schlick 1995)

```
f_DRC(g) = q·|g| / ( (q−1)·|g| + 1 )
q        = (0.2 − 0.2·median(|g|)) / (median(|g|) − 0.2·median(|g|))
```

Rationally-scaled tone mapping. Median-adaptive — handles SAS's wide
dynamic range (strong seabed highlights alongside shadows).

## Key results — the numbers

### Dataset

- 504 SLC images from an HF-SAS mounted on a UUV (NSWC-PC data)
- Each 256×256 pixels, ω-k beamformer output (baseline reconstruction)
- 7 seabed classes: rock, packed sand, mud, small ripple, large ripple,
  sea grass, shadow
- Train/val/test: 120 / 120 / 264 images
- Phase corruption: 10-degree polynomial, coefficients ~ U[-1,1],
  scaled by U[-18, 18] radians

### Image quality (Figure 3 in paper)

Mean scores across 264 test images, higher is better:

| Method | PSNR (dB) | MS-SSIM |
|---|---|---|
| MNS-GD (10 iter) | ~17 | ~0.72 |
| ME-GD (10 iter) | ~17 | ~0.71 |
| OSF-GD (10 iter) | ~17 | ~0.71 |
| SSI-GD (10 iter) | ~18 | ~0.74 |
| **Deep Autofocus (1 iter)** | **~22** | **~0.84** |

Deep Autofocus wins PSNR by ~4 dB and MS-SSIM by ~0.10. More
important: the **left tail of the violin plot** (worst cases) is
significantly narrower for Deep Autofocus, meaning it does not
catastrophically fail.

### Runtime (Table II)

Wall-clock per image, all on NVIDIA Titan X GPU:

| Method | Iterations | Runtime/image |
|---|---|---|
| MNS-GD | 10 | 340 ms |
| ME-GD | 10 | 340 ms |
| OSF-GD | 10 | 340 ms |
| SSI-GD | 10 | 340 ms |
| **Deep Autofocus** | **1** | **18 ms** |

**~19× speedup** vs the fastest classical baseline. In SWaP-constrained
UUV deployment, this is the headline.

### Catastrophic-failure case (Figure 4)

An example where all four sharpness metrics fail to recover the
image (left-tail of violin plot) and Deep Autofocus succeeds — same
defocused input, visibly different qualitative outcomes.

## Strengths

- **Single-iteration inference.** Unlike SPGA / classical sharpness
  autofocus which iterates 10-50×, Deep Autofocus does one forward
  pass. Real-time UUV viability.
- **Self-supervised training.** No ground-truth focused/defocused
  pairs needed — which is essential because we rarely have them
  (field-collected SAS data has *real* phase errors, not synthetic).
- **Implicit weighting.** The hand-crafted weighting function `w` of
  classical methods (Fienup 2000) is replaced by the CNN's learned
  feature extraction. No per-scene tuning.
- **Robustness.** The violin plot's narrow left tail shows Deep
  Autofocus doesn't catastrophically fail the way sharpness metrics
  can when trapped in local extrema.
- **Phase map input.** Using both magnitude (DRC) and phase of the
  SLC as CNN input is essential — an ablation shows magnitude-only
  fails (phase information is needed to distinguish phase-error sign
  in symmetric PSFs).
- **Differentiable FFT + dynamic-range compression.** The pipeline
  is end-to-end differentiable, so gradients flow through the
  physics. This is a template for other coherent-imaging ML work.

## Limitations

- **10-degree polynomial phase model.** Only captures low-frequency
  phase error (bulk sway + SSP drift). High-frequency
  micro-sway, turbulent medium fluctuation, and yaw-induced errors
  are not in the model. Classical SPGA at higher orders may beat
  Deep Autofocus on adversarial scenes.
- **HF-SAS only.** Trained on ~100 kHz single-band data. Medium-
  frequency (10-30 kHz) and low-frequency (< 10 kHz) SAS has very
  different phase-error characteristics and the network would need
  retraining.
- **Synthetic phase errors.** Testing uses injected 10-degree
  polynomial corruption, not measured real-world errors. Good
  controlled experiment, but the authors' later work (DAPL 2024,
  TGRS) demonstrates on real defocusing.
- **Small dataset.** 504 images is tiny for modern deep learning.
  Train/val/test split of 120/120/264 means effective training set
  is ~120 images. Data augmentation (random polynomial phase) partly
  rescues this but may overfit to polynomial structure.
- **DenseNet-121 is heavy.** 7M parameters for a 4-page conference
  paper's simple regression task is overkill. A lighter backbone
  (MobileNet, EfficientNet-B0) would likely match performance at
  <1M params — essential for UUV edge deployment.
- **Ignores range-variant phase errors.** The uniform-phase model
  (Eq. 1, phase vector depends only on along-track, not on range)
  is the same spotlight-SAR assumption that SPGA explicitly
  rejected for stripmap SAS. This is a real limitation for wide-beam
  wide-band SAS.
- **No public code.** The companion GitHub
  (isaacgerg/synthetic_aperture_sonar_autofocus) is a simpler
  non-deep variant, not the Deep Autofocus weights.

## Portable details — the math we will reuse

### The full forward equation (Eq. 7 restated)

Given defocused SLC `g_e ∈ C^{M×M}`, CNN `h` with parameters Θ:

```
Input to CNN:   [f_DRC(g_e),  arg(g_e)]       ∈ R^{2×M×M}
CNN output:     c ∈ R^8                         (polynomial coeffs 2..9)
Polynomial:     φ(u) = Σ_{k=2..9}  c_k · (u/M - 0.5)^k     u = 0..M-1
k-space apply:  G_e = F_u{g_e}
                Ĝ   = (e^{-iφ} ⊗ 1_T) ⊙ G_e
Output:         ĝ = |F_u^{-1}{ Ĝ }|
```

### The self-supervised loss (Eq. 8)

```
L(g_e, ĝ) = − (M_MNS(ĝ) − M_MNS(g_e)) / M_MNS(g_e)
M_MNS(x)  = stddev(|x|) / mean(|x|)
```

No ground truth needed. Minimizing L maximizes relative sharpness
improvement. Training objective equivalent to "make the output
sharper than the input, by a maximum relative margin."

### Phase-corruption simulator (for training data augmentation)

```
Pick polynomial order d ~ Uniform{2, 3, ..., 10}
Pick coefficients c_k ~ Uniform[-1, 1] for k = 0..d
Normalize so max|poly(u)| = 1 over u ∈ [0, 1]
Scale by s ~ Uniform[-18, 18] radians
φ(u) = s · normalized_polynomial(u)
G_e  = (e^{iφ} ⊗ 1_T) ⊙ F_u{g}
g_e  = F_u^{-1}{G_e}
```

This augmentation strategy is directly portable — it's a clean way
to bootstrap a focused dataset into a (focused, defocused) training
pair set.

### Dynamic range compression (Schlick 1995)

```
f_DRC(g) = q · |g| / ( (q - 1) · |g| + 1 )
q        = (0.2 - 0.2·median|g|) / (median|g| - 0.2·median|g|)
```

Useful outside SAS for any high-dynamic-range imaging input to a CNN.

## Sonobuoy integration plan

### Deep Autofocus is THE method we want for drifting-buoy SAS

The buoy-SAS scenario has two properties that make Deep Autofocus
more applicable than classical SPGA:

1. **GPS-limited phase error is large and low-frequency.** Buoy
   position error is dominated by GPS bias (~2 m) and slow drift
   (~0.1 m/s random walk). This is exactly the low-degree
   polynomial regime Deep Autofocus is trained for. Classical SPGA
   would converge slowly on such large initial errors; Deep
   Autofocus hits convergence in one shot.
2. **Edge-deployability matters.** Buoys are SWaP-constrained. A
   single-forward-pass ~20 ms CNN on a small edge GPU (Jetson Orin
   Nano 6W) is feasible; iterative SPGA is not.

### Drop-in architecture for the `weftos-sonobuoy-active` crate

```rust
pub struct DeepAutofocus {
    model: CnnModel,             // DenseNet-121 + MLP or lighter
    phase_order: usize,          // default 10, tunable for buoy SAS
    device: Device,              // CPU, CUDA, Metal, WASM
}

impl DeepAutofocus {
    pub fn autofocus(&self, slc: &Slc) -> (Slc, PhaseCorrection) { ... }
}

pub struct PhaseCorrection {
    pub polynomial_coeffs: Vec<f32>,   // 8 elements (deg 2..9)
    pub estimated_sway:    Vec<f32>,   // for telemetry / ADR-style audit
}
```

Training data: **we need a focused SAS image corpus** — this is a
project dependency. Possibilities:

- Public datasets: NSWC-PC releases a small SAS image set periodically;
  PondHopper dataset from Hansen/FFI.
- Synthetic: simulate focused buoy-SAS from known seabed scattering
  models (Lambertian + glint) using platform trajectories from
  HYCOM/ROMS ocean-current models.
- Bootstrap: collect real drifting-buoy SAS data with classical SPGA
  autofocus as "pseudo-focused" training targets, then train Deep
  Autofocus to match.

### Phase-model refinement for drifting-buoy regime

The 10-degree polynomial is probably **under-parameterized** for
drifting-buoy SAS because buoy drift is not polynomial — it has
tidal (period ~12 h) and inertial-oscillation (period ~15 h)
components, and wave-induced surge at 0.1-1 Hz. Proposed extensions:

- **Fourier-basis phase model:** represent φ(u) as Σ_k a_k sin(kω₀u)
  + b_k cos(kω₀u), with ω₀ tied to expected drift timescales.
- **Physics-prior injection:** feed the CNN the buoy GPS trajectory
  as an auxiliary input. The CNN then outputs a *residual* phase
  correction on top of the GPS-derived baseline. This is much
  easier than learning the absolute phase from scratch.
- **Multi-head for per-buoy corrections:** in multistatic SAS
  (Paper 5.4), each buoy has its own phase error; a multi-head CNN
  can output N independent correction polynomials.

### Target ADR-063

> **ADR-063: Deep Autofocus as primary phase-error corrector in
> `weftos-sonobuoy-active`.**
>
> **Decision:** Implement Gerg-Monga 2021 Deep Autofocus with
> modifications: (a) replace 10-degree polynomial phase basis with
> Fourier basis tied to expected drift timescales; (b) inject
> GPS-derived trajectory as residual-learning prior; (c) expose
> estimated phase correction as telemetry for ECC/audit (never
> hide estimation results).
>
> **Fallback:** SPGA (classical) when no trained model available
> or for adversarial scenes.

## Follow-up references (second-degree citations)

1. **Cook, D. A., Brown, D. C., Hughes, D. J., et al. (2008).**
   "Analysis of phase error effects on stripmap SAS." *IEEE JOE*,
   **34**(3), 250–261. — The paper that justifies the polynomial
   phase-error model in Deep Autofocus (ref [1] in Gerg-Monga).
2. **Evers, A., Zelnio, E. G., & Jackson, J. A. (2019).** "A
   generalized phase gradient autofocus algorithm." *IEEE Trans.
   Computational Imaging*, **5**(4), 606–619. — The classical-
   statistical state-of-the-art that Deep Autofocus outperforms
   (ref [12]).
3. **Gerg, I. D., & Monga, V. (2024).** "Deep Adaptive Phase
   Learning: Enhancing Synthetic Aperture Sonar Imagery Through
   Coherent Autofocus." *IEEE Trans. Geoscience and Remote
   Sensing*, DOI 10.1109/TGRS.2024.3385010. — The direct journal
   extension of this work; adds coherent (complex-valued) output
   and improved k-space handling.
4. **Williams, D. P. (2018).** "Fast target detection in synthetic
   aperture sonar imagery: A new algorithm and large-scale
   performance analysis." *IEEE JOE*, **44**(1), 71–92. — The
   target-detection front-end that could supply `TargetHint`s to
   Deep Autofocus, analogous to the sonobuoy spatial-GNN detector.
5. **Fienup, J. R. (2000).** "Synthetic aperture radar autofocus
   by maximizing sharpness." *Optics Letters*, **25**(4), 221–223.
   — The classical-sharpness reference (ref [4]) against which
   Deep Autofocus is measured; worth reading as the "why does
   sharpness-maximization work" primer.

---

*This analysis is Paper 5.3 of Round 2 (SAS) of the sonobuoy
literature survey. It establishes the modern ML baseline for SAS
autofocus and is the closest methodological analog to what the
`weftos-sonobuoy-active` crate should implement. Its self-supervised
training recipe (Eq. 8) and k-space phase-application pattern (Eq. 7)
are the two highest-value portable artifacts.*
