# Paper 5.4 — Kiang & Kiang 2022, "Imaging on Underwater Moving Targets With Multistatic Synthetic Aperture Sonar"

## Citation

Kiang, C.-W., & Kiang, J.-F. (2022). "Imaging on Underwater Moving
Targets With Multistatic Synthetic Aperture Sonar." *IEEE Transactions
on Geoscience and Remote Sensing*, **60**, art. no. 4211218, pp. 1–18.

- **DOI:** [10.1109/TGRS.2022.3220708](https://doi.org/10.1109/TGRS.2022.3220708)
- **IEEE Xplore:** https://ieeexplore.ieee.org/document/9941127/
- **Author's copy (open):** http://cc.ee.ntu.edu.tw/~jfkiang/selected_publications/TGRS_2022.pdf

**Affiliation:** Dept. of Electrical Engineering, National Taiwan
University, Taipei. Corresponding author: jfkiang@ntu.edu.tw (Jean-Fu
Kiang, Life Senior Member IEEE).

## Status

**Verified.** Full PDF (6 MB, 18 pages) downloaded from the first
author's institutional page at NTU and opened with PyMuPDF; all 18
pages parsed cleanly including the 8 simulation cases, equations,
and conclusions. IEEE Xplore URL confirms DOI and article number
4211218; the author, title, venue, and year all match. Submitted
1 July 2022, accepted 6 Nov 2022, published 7 Nov 2022, current
version 17 Nov 2022. **This paper is the single most relevant paper in
the entire SAS literature for the sonobuoy project — it explicitly
proposes a multistatic SAS configuration consisting of "an active
sonar, a towed receiver, and a sonobuoy."**

PDF: `.planning/sonobuoy/papers/pdfs/multistatic-sas.pdf` (6 MB, 18
pages).

## One-paragraph summary

This is the first paper to propose and rigorously develop a
**three-node multistatic synthetic aperture sonar (SAS)** configuration
that explicitly integrates a drifting sonobuoy as one of the receivers.
The three nodes are: (A) a cable-towed transceiver on a host ship
moving at Vp = 3 m/s, (B) a towed passive receiver trailing the
transceiver at baseline L₂ = 250 m, and (C) a **stationary sonobuoy**
at a fixed position separated by hundreds of metres from the tow track.
All three receive echoes from a moving submarine target modelled as
a 3D-mesh-tessellated USS Albacore hull at 2 km standoff. Conventional
SAS assumes a stationary scatterer and uses platform motion to
synthesize the aperture; Kiang & Kiang **drop both the
stop-and-go approximation and the stationary-target assumption** and
derive a full non-stop-and-go range model for each of the three
(monostatic, bistatic, bistatic) pairs. The key signal-processing
chain is (1) range compression, (2) range-frequency reversal
transform (RFRT) to concentrate the signal around τ=0, (3) modified
second-order Wigner-Ville distribution (SoWVD) to estimate the
azimuth FM rate K_a, (4) range cell migration correction (RCMC) using
the estimated K_a, (5) Radon transform on the RCMC output to estimate
the Doppler centroid f_dc, (6) range-walk compensation (RWC), and (7)
azimuth compression. The three Doppler centroids f_dc1, f_dc2, f_dc3
(one per receiver) are **solved jointly for the target velocity vector
(v_x, v_y, v_z) with <3% error, insensitive to noise down to SNR =
−17 dB**. Eight scenarios with different target velocities are
simulated; in all but one (target moving at the same velocity as the
platform) the imaging succeeds, and in the one blur-prone case PGA
autofocus fully recovers the image.

## Methodology — multistatic SAS with sonobuoy

### Geometry (Fig. 1 of paper)

Coordinate system: x = cross-track, y = along-track (direction of
platform motion), z = vertical.

| Node | Role | Position at slow-time η |
|---|---|---|
| A | Transceiver | `(0, Vp·η, −L₁ sin θ_b)` — towed underwater, moves +y |
| B | Passive receiver | `(0, Vp·η − L₂, −L₁ sin θ_b)` — towed behind A at baseline L₂ |
| C | Sonobuoy (passive) | `(x_C, y_C, z_C) = (−500, 0, −150)` m — **stationary** |

Default parameters (Table I of paper):

- **Carrier:** f₀ = 150 kHz
- **Range bandwidth:** B_r = 20 kHz
- **Pulse duration:** T_r = 5 ms
- **Aperture time:** T_a = 80 s
- **PRF:** 80 Hz
- **Platform speed:** V_p = 3 m/s
- **Tow cable length:** L₁ = 250 m
- **Transceiver–receiver baseline:** L₂ = 250 m
- **Depression angle:** θ_b = 10° → sonar depth ~43 m
- **Sound speed:** c_s ≈ 1500 m/s
- **Target:** scaled USS Albacore mesh, LOA = 10 m, depth = 350 m,
  standoff ≈ 2 km

### Non-stop-and-go range model (Eqs. 1-4)

For the nth scatterer at position (x_n0, y_n0, z_n0) at η = 0, moving
with velocity (v_x, v_y, v_z):

**Forward path** (transmitter A → target):

```
R'_1n(η) ≈ R'_10 + (c_s · f'_dc1)/(2 f_0) · η
               + (c_s · K'_a1)/(4 f_0) · η²        (Eq. 1)
```

**Backward path** (target → transceiver A after platform has moved):

```
R''_1n(η) ≈ R''_10 + (c_s · f''_dc1)/(2 f_0) · η
               + (c_s · K''_a1)/(4 f_0) · η²       (Eq. 2)
```

The Doppler centroids (2nd-order range model coefficients):

```
f'_dc1  = (2 f_0 / (R'_10 c_s)) · [v_x(x_n0 + v_x R_10/c_s)
        + (v_y − V_p)(y_n0 + v_y R_10/c_s)
        + v_z(z_n0 + v_z R_10/c_s + L₁ sin θ_b)]          (Eq. 3)

f''_dc1 = similar, but with (v_y − 2V_p) instead of (v_y − V_p)
          in the y-term                                   (Eq. 4)
```

The azimuth FM rates:

```
K'_a1 = (2 f_0 / (R'_10 c_s)) · [v_x² + (v_y − V_p)² + v_z²]    (Eq. 5)
K''_a1 = same but with R''_10                                    (Eq. 6)
```

Note these coefficients **depend on both platform motion (V_p) and
target motion (v_x, v_y, v_z)** — the conventional SAS case (stationary
target) collapses to v = 0 and gives the standard Doppler coefficients.

For the bistatic pair (B or C), the range is `R_tx + R_rx` with
separate Doppler terms; see Eqs. (21)-(25) for receiver B and
analogous equations for receiver C in § IV.

### The processing chain (Fig. 3 of paper)

```
raw echo s_rb1(τ, η)
    ↓  range FFT
S_11(f_τ, η)
    ↓  range compression filter H_rc(f_τ) = e^{iπ f_τ²/K_r}
S_12(f_τ, η)                   ← Eq. 8-9
    ↓  multiply by mirrored replica (RFRT)
S_rfrt(f_τ, η) = S_12 · S_12(−f_τ, η)
    ↓  range IFFT
s_rfrt(τ, η)                   ← Eq. 10-11
    ↓  modified SoWVD (Eq. 12) → 2D FFT → peak-find
estimated K_a (azimuth FM rate)
    ↓  build RCMC filter H_rcmc(f_τ, η) = e^{iπ f_τ K̂_a η²/f_0}
S_13(f_τ, η) = S_12 · H_rcmc
    ↓  range IFFT
s_rcmc1(τ, η)                  ← Eq. 17
    ↓  Radon transform → peak-find
estimated f_dc (Doppler centroid)
    ↓  build RW filter H_rw(f_τ, η) = e^{−i2π(f_0 + f_τ) tan θ̂_R · η}
S_14(f_τ, η) = S_13 · H_rw
    ↓  azimuth FFT → range IFFT
S_16(τ, f_η)                   ← Eq. 18
    ↓  azimuth compression H_ac(f_η) = e^{−iπ f_η²/K̂_a}
S_17(τ, f_η)
    ↓  azimuth IFFT
final image s_18(τ, η)         ← Eq. 20 — focused
```

### Velocity vector estimation (§ V)

This is the paper's most original contribution beyond the imaging
pipeline. Three Doppler centroids estimated from the three receivers:

```
f̂'_dc1  (from monostatic transceiver A)
f̂''_dc2 (from bistatic receiver B)
f̂''_dc3 (from bistatic sonobuoy C)
```

Each Doppler centroid is a **linear function of the three target
velocity components (v_x, v_y, v_z)** once geometry is substituted
(Eqs. 3-4, 23, and analogous for C). Stack as a 3×3 linear system:

```
| ∂f_dc1/∂v_x  ∂f_dc1/∂v_y  ∂f_dc1/∂v_z |   | v_x |   | f̂_dc1 − const₁ |
| ∂f_dc2/∂v_x  ∂f_dc2/∂v_y  ∂f_dc2/∂v_z | · | v_y | = | f̂_dc2 − const₂ |
| ∂f_dc3/∂v_x  ∂f_dc3/∂v_y  ∂f_dc3/∂v_z |   | v_z |   | f̂_dc3 − const₃ |
```

Solve by matrix inversion. Given SNR ≥ −17 dB, velocity vector is
recovered with <3% error, **insensitive to noise over a 17 dB SNR
range**.

## Key results

### Eight simulation cases (§ VI)

| Case | Target velocity | Outcome |
|---|---|---|
| 1 | (0, −4, 0) — opposite to platform | Clear image, v̂ = (0.002, −3.914, 0), error ≈ 2% |
| 2 | (4, 0, 0) — cross-track | Clear image |
| 3 | (0, 4, 0) — **same direction as platform** | Blurred (SAS malfunction), PGA helps |
| 4 | (−4, 0, 0) — opposite cross-track | Clear image |
| 5 | (2, −4, 0) — oblique | Mild blur, PGA fixes |
| 6 | (2, 4, 0) — oblique co-direction | Mild blur, PGA fixes |
| 7 | (0, 0, −1) — diving | Clear image |
| 8 | (4, 0, 0) at high target speed | Significant blur from hull occlusion; PGA recovers upper half |

Case 3 is the pathological scenario: target moving with exactly the
platform velocity → no relative motion → no azimuth frequency
modulation → SAS principle fails. The paper explicitly flags this as
a failure mode never previously documented in the literature.

### Verification against SOTA (§ VII-A)

Three state-of-the-art monostatic SAS algorithms tested on case 1:

| Algorithm | SSIM vs proposed |
|---|---|
| Conventional RDA (stop-and-go) | 0.9398 |
| Chirp scaling algorithm (CSA) | 0.9111 |
| ω-KA (omega-k) | 0.9082 |

All three SOTA SAS algorithms produce images that are ~1 s
azimuth-shifted from the non-stop-and-go result, demonstrating the
measurable impact of the stop-and-go approximation when the target
is moving. SSIM > 0.9 in all cases — the scenes are recognizably
similar but the proposed method has better motion-fidelity.

### Noise sensitivity (§ VII-C)

- SNR ≥ −15 dB: velocity error stays in 1.8–2.2%.
- SNR = −17 dB: velocity error ~2.5%, images still focused.
- SNR ≤ −20 dB: peak on S_wv(f_p, f_η) cannot be identified →
  method fails.

Remarkable noise robustness (down to −17 dB SNR) arises from the
coherent integration over 80 s aperture time and the peak-detection
robustness of the RFRT + SoWVD combination.

### Two independent targets (§ VII-B)

Extended to two submarines at the same depth, 50 m apart. Iterative
extraction (image → inverse-process → subtract → image the second
target) works; demonstrates method scales to multiple targets but
at linear cost per target.

### PGA autofocus application (§ VII-D)

For the three blurred cases (5, 6, 8), classical phase gradient
autofocus (PGA) is applied after the main processing chain. **PGA
fully recovers case 8** (the worst blur, from hull-occlusion-induced
phase errors); cases 5 and 6 also improve. This establishes PGA (and
by extension the Callow SPGA of Paper 5.2 and the Gerg-Monga Deep
Autofocus of Paper 5.3) as **composable with multistatic SAS** —
multistatic first, then autofocus.

## Strengths

- **Explicitly integrates a sonobuoy as a multistatic node.** This is
  the defining feature. The authors state "**this is the first
  work to propose the use of scan images projected from 3-D target
  models to help verify an acquired SAS image of a moving target**"
  and "**we explore the possibility of integrating a sonobuoy to
  increase the flexibility and versatility of SAS imaging and
  velocity estimation on underwater moving targets, which has never
  been discussed in the literature.**"
- **Non-stop-and-go range model.** Rigorous derivation of all three
  bistatic range equations without the stop-and-go approximation,
  which was needed for target motion and fundamentally required by
  the long aperture times of SAS.
- **Joint velocity vector estimation.** The 3×3 linear system for
  (v_x, v_y, v_z) from three Doppler centroids is elegant — and more
  importantly, the system is *well-conditioned* given the three
  receivers' geometric diversity (towed, towed+offset, stationary
  elsewhere).
- **Noise robustness down to −17 dB SNR.** That's operational-grade;
  field acoustic environments often have −10 to −20 dB SNR.
- **Composable with existing autofocus.** PGA applied after
  multistatic processing works, so classical and ML autofocus
  extensions plug in.
- **Comprehensive scenario coverage.** Eight cases cover the
  pathological co-directional motion scenario and the SAS failure
  mode — the paper does not hide the method's limits.
- **Open accessible PDF via NTU mirror.** Unlike most IEEE TGRS
  papers, this is readable without paywall.

## Limitations

- **Simulation only; no field data.** The "submarine" is a
  tessellated hull mesh with Lambertian scattering; real ocean
  acoustic effects (SSP layering, reverberation, multipath off
  surface/bottom) are not modelled. The authors acknowledge this
  explicitly (§ VII-G).
- **Constant-velocity target assumption.** Acceleration /
  maneuvering is not handled; would require a higher-order range
  model with more parameters and more receivers.
- **Sonobuoy is stationary.** The paper models the sonobuoy as a
  fixed point. Real sonobuoys drift at 0.1–1 m/s, so the range
  model for receiver C needs to be generalized to include C's own
  trajectory. This is a straightforward extension but not in the
  paper.
- **Single-sonobuoy analysis.** Real deployments use patterns of
  20-40 buoys. The 3×3 linear system generalizes to N×3
  overdetermined least-squares — covered by the geometry but not
  worked out.
- **150 kHz carrier.** High-frequency, short-range (2 km). Real
  submarine detection often runs at 1–10 kHz for longer range; the
  method scales but PRF, aperture time, and bandwidth would
  re-tune.
- **No uncertainty quantification.** The velocity vector estimate
  comes with no confidence interval; in a Bayesian framework we'd
  want posterior covariance for downstream tracking.
- **Computational load (Table III).** Dominant cost is the 2D
  FFT in SoWVD, O(N_r N_a log N_a). For large Na (80 s × 80 Hz =
  6400 bins), this is heavy. Edge deployment on a buoy is not
  feasible; must run on the collecting vessel or cloud.

## Portable details — the multistatic SAS math we need

### Generalized N-buoy range model

For N nodes (some active, some passive, some drifting), generalize
Eqs. 1-4 as:

```
R_tx,n(η)   = range from transmitter to scatterer n at time η
R_rx,k,n(η) = range from receiver k to scatterer n at time η
R_pair,k,n(η) = R_tx,n(η) + R_rx,k,n(η)      [bistatic]
```

For a *drifting* receiver k at position `p_k(η)`, we have
`R_rx,k,n(η) = ‖p_k(η) − (x_n0, y_n0, z_n0) − v_n · η‖` where
`p_k(η)` comes from GPS + drift model (linear, Lagrangian, or
HYCOM-informed).

### Doppler centroids as linear functions of velocity

```
f_dc,k = (2 f_0 / (R_k c_s)) · [ ∇_v R_pair,k,n ] · v   +  const_k
```

where ∇_v is the gradient of R_pair,k,n with respect to target
velocity v = (v_x, v_y, v_z). This is the core identity that lets
us estimate v from N Doppler centroid measurements.

### Joint velocity estimation (N ≥ 3 receivers)

```
[ ∇_v R_pair,1;  ∇_v R_pair,2;  ... ; ∇_v R_pair,N ]  · v
    = [ f_dc,1; f_dc,2; ... ; f_dc,N ]  −  const
```

N×3 overdetermined for N > 3 → ordinary least squares:

```
v̂ = (AᵀA)^{-1} Aᵀ (f_dc − const)
```

For our buoy field (typical N = 20-40), this is very overdetermined
and robust to individual buoy failures.

### RFRT key identity (Eq. 10)

Multiplying a range-compressed signal by its mirror replica about
`f_τ = 0` eliminates the `f_τ` dependence of phase terms:

```
S_rfrt(f_τ, η) = S_12(f_τ, η) · S_12(−f_τ, η)
            ~ rect(f_τ/B_r) · exp(−j4π f_0 (R_10 + R_11(η) + R_12(η))/c_s)
```

Simultaneously compensates **range cell migration and range walk**
— saving a step vs RCMC + RWC pipelines.

### SoWVD for K_a estimation (Eq. 12)

The modified second-order Wigner-Ville distribution:

```
s_wv(p, η) = s_rfrt(τ₀, η + p/2)   · s_rfrt*(τ₀, η − p/2)
           · [s_rfrt(τ₀, η + p/2 + p₀) · s_rfrt*(τ₀, η − p/2 − p₀)]*
```

Take 2D FFT; peak in (f_p, f_η) plane at `(0, 4 p₀ K_a)` gives K_a.
The tuning parameter `p₀` trades off resolution against focus.

## Sonobuoy integration plan

### This paper is the closest published analog to what we want to build

The distributed-sonobuoy SAS vision is **essentially a generalization
of Kiang & Kiang 2022 to N stationary/drifting buoys with no active
transceiver** (pure passive SAS) or **with cooperative active
illumination** (active multistatic). The paper's math is the
starting point.

### What ports directly into `weftos-sonobuoy-active`

1. **Non-stop-and-go range model.** Eqs. 1-6 generalize to any
   transmitter/receiver geometry. Implement as a trait:

   ```rust
   pub trait RangeModel {
       fn range(&self, scatterer: ScattererState, eta: f32) -> f32;
       fn range_derivative(&self, scatterer: ScattererState, eta: f32)
           -> (f32, f32);    // (f_dc, K_a)
   }

   impl RangeModel for MonostaticGeometry { ... }
   impl RangeModel for BistaticGeometry { ... }
   impl RangeModel for DriftingBuoyGeometry { ... }
   ```

2. **RFRT + SoWVD K_a estimator.** Fully derived in Eqs. 10-16;
   implementable as a stateless DSP function.
3. **Radon-transform f_dc estimator.** Standard Radon transform on
   |s_rcmc|; one peak-find.
4. **N-receiver velocity-vector estimator.** OLS on the N×3 Jacobian
   of Doppler centroids. Add UncertaintyQuantification layer returning
   covariance.
5. **PGA / Deep Autofocus composition.** The paper demonstrates that
   PGA runs cleanly after multistatic processing; our pipeline
   should do multistatic-recon → autofocus as a pipeline stage.

### What needs extending

1. **Drifting sonobuoys (not stationary).** Generalize the range
   model for receiver C to `p_C(η) = p_C(0) + v_drift · η`, with
   `v_drift` estimated from GPS time-series.
2. **Passive multistatic (no active transceiver).** The paper
   relies on an active transmitter at node A. A pure-passive
   version uses the target's own radiated noise as the "source"
   and each buoy as a receiver → this is **inverse SAS (ISAS)**
   and requires the k_x bandwidth to come from target-relative
   motion only. The Doppler centroid estimation is the same;
   the range compression step is replaced by cross-correlation
   against a template or blind-deconvolution.
3. **N-buoy instead of 3-node.** OLS system scales to N×3; need
   robust OLS (Huber) to handle buoy outliers from acoustic
   shadowing or weather.
4. **Uncertainty quantification.** Posterior covariance for
   (v_x, v_y, v_z) estimate — plug into downstream tracking
   (Kalman filter, particle filter).
5. **Edge-compute constraints.** The paper runs everything on a
   mothership. For buoys to preprocess locally (reduce uplink
   bandwidth), the K_a estimation and Radon transform need edge-
   deployable variants — probably CNN approximators to the SoWVD
   and Radon peaks.

### Proposed ADR-064

> **ADR-064: Multistatic SAS as the core imaging model for the
> sonobuoy active-imaging branch.**
>
> **Decision:** Adopt Kiang & Kiang 2022's non-stop-and-go
> multistatic SAS formalism as the reference model for the
> `weftos-sonobuoy-active::multistatic` module. Generalize to N
> drifting receivers with per-receiver range models and OLS
> velocity-vector estimation. Maintain PGA / Deep Autofocus as
> post-processing stages.
>
> **Rationale:** This is the only published SAS formalism that
> explicitly includes a sonobuoy node and covers the non-stop-
> and-go regime required for long aperture times.

### Expected performance for a 20-buoy pattern

With 20 buoys on a 1-km square grid, a 5 kHz active illumination
from a separate ship, 80 s aperture, and a moving target at 2 km
standoff:

- **Velocity error:** ~0.5-1% (scaling from 3% at N=3 to higher
  N by standard OLS SNR scaling √(N/3)).
- **Imaging resolution:** limited by aperture geometry. With
  buoys spread over 1 km, synthesizable aperture could exceed
  platform-only SAS by 10×, pushing theoretical δ_x to <1 m at
  5 kHz, 2 km.
- **SNR floor:** should hold ≤ −17 dB for each individual buoy's
  velocity estimate, improving N-fold for the joint estimate.

## Follow-up references (second-degree citations)

1. **Kiang, C.-W., & Kiang, J.-F. (2021).** "ISAS imaging of
   submerged targets using polynomial chirp and cubic chirplet
   decomposition." (Ref [29] in paper; authors' earlier work on
   inverse-SAS imaging of underwater vehicles.) — Establishes the
   ISAS framework that the current paper generalizes to multistatic.
2. **Yang, H.-S., et al. (2019).** "Fast imaging algorithm for
   multistatic synthetic aperture sonar." (Ref [33].) — The MSAS
   imaging algorithm with phase-center approximation that the
   current paper compares against.
3. **Soumekh, M. (1999).** *Synthetic Aperture Radar Signal
   Processing with MATLAB Algorithms*. Wiley, New York. (Ref [39].)
   — The ω-K / chirp-scaling / RDA reference implementation that
   Kiang & Kiang use for verification; canonical textbook for
   Fourier-domain SAR/SAS.
4. **Goldstein, R. M., Zebker, H. A., & Werner, C. L. (1988).**
   "Satellite radar interferometry: Two-dimensional phase
   unwrapping." *Radio Science*, **23**(4), 713–720. — Foundational
   phase-unwrapping for multistatic SAS interferometry (extension
   path beyond current paper).
5. **Zhang, Y., Wang, Z., & Krolik, J. L. (2011).** "Statistical
   performance analysis of maneuvering target tracking using
   multistatic sonar systems." *IEEE JOE*, **36**(4), 648–661.
   DOI: 10.1109/JOE.2011.2165198. — Statistical framework for
   multistatic tracking that dovetails with the velocity-vector
   estimate of the current paper.

---

*This analysis is Paper 5.4 of Round 2 (SAS) of the sonobuoy
literature survey. It is the **single most relevant** published work
for the distributed-sonobuoy active-imaging vision: the only paper to
explicitly propose a sonobuoy as a multistatic SAS node, with a
rigorous non-stop-and-go range model and joint velocity-vector
estimation. This paper is the seed for ADR-064 and for the
`weftos-sonobuoy-active::multistatic` module's entire math.*
