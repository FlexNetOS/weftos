# Paper 5.2 — Callow 2003, "Signal Processing for Synthetic Aperture Sonar Image Enhancement" (SPGA autofocus)

## Citation

Callow, H. J. (2003). "Signal Processing for Synthetic Aperture Sonar
Image Enhancement." PhD thesis, Department of Electrical and
Electronic Engineering, University of Canterbury, Christchurch, New
Zealand. April 2003. 273 pages. Supervisors: Dr. M. P. Hayes and Prof.
P. T. Gough. Examiners: Prof. H. Griffiths, Dr. R. Holtkamp, Dr. P.
Bones.

**PDF (public):** https://www.math.ucdavis.edu/~saito/data/sonar/Callow_thesis.pdf
(3.9 MB, 273 pages; confirmed downloaded and parsed).

**Key derived paper** (if one prefers a journal cite over a thesis):
Callow, H. J., Hayes, M. P., & Gough, P. T. (2003). "Stripmap phase
gradient autofocus." In *IEEE OCEANS 2003*, San Diego, CA, vol. 5, pp.
2414–2421. DOI: [10.1109/OCEANS.2003.178291](https://doi.org/10.1109/OCEANS.2003.178291).

## Status

**Verified.** The 273-page thesis PDF was successfully downloaded from
UC Davis's math department mirror (Prof. Saito's SAS benchmark
collection), opened with PyMuPDF, and chapters 1, 9, 10, and 11 were
extracted and verified. Authorship (Hayden J. Callow), institution
(University of Canterbury), supervisors (Hayes & Gough), publication
list (11 conference + journal papers all listed in the Preface), and
technical content (stripmap phase gradient autofocus, SPGA) all check
out. The thesis is the canonical reference for SAS autofocus beyond
spotlight-mode PGA and has been cited ~500+ times (Google Scholar).

PDF: `.planning/sonobuoy/papers/pdfs/sas-autofocus-callow.pdf` (3.9 MB,
273 pages).

## One-paragraph summary

This thesis generalizes the spotlight-SAR phase gradient autofocus
(PGA) algorithm of Wahl et al. (1994) to the wide-beam, wide-bandwidth
**stripmap geometry** that SAS operates in. Spotlight PGA estimates a
single along-aperture phase error function by windowing on a dominant
point scatterer, stacking its azimuth-compressed responses across
bursts, and taking the phase gradient of the dominant singular
vector; it assumes the scatterer sits under the full aperture the
whole time. Stripmap SAS violates that: each scatterer is illuminated
only for the synthetic aperture length `L_SA = λR/D`, not the full
track, and the sonar's wide beam (20-30° typical) plus wide
fractional bandwidth (40-60%) create *space-variant* blurring that
spotlight PGA cannot model. Callow's **SPGA (Stripmap PGA)** adapts
PGA with four innovations: (1) target-region selection that
identifies which piece of the full aperture each scatterer sees, (2)
windowing with an iteratively-shrinking bandwidth (α = 0.6 works),
(3) along-track position estimation for each target prior to phase
estimation, and (4) a wavenumber-transform coordinate change that
makes the blur model space-invariant in the new coordinates. On
simulated KiwiSAS-II data SPGA converges in 3–4 iterations; on
field-collected Sydney Harbour data it brings image resolution near
the diffraction limit. The thesis also contains a major treatment of
micronavigation (DPCA/RPC and shear-average), which is the
*deterministic* alternative to autofocus when hardware permits
multiple hydrophones per aperture.

## Methodology — SPGA algorithm

### Spotlight PGA recap (Wahl 1994)

Spotlight PGA operates on a range-compressed, azimuth-compressed
complex image `g(x, y) ∈ C^(M×M)`. It models blurring as a spatially
uniform along-track phase error `φ(u)`:

```
g_defocused(x, y) = F_u^{-1} { e^{iφ(u)} · F_u{g(x,y)} }
```

The algorithm:
1. Identify strongest scatterers; assume they are ideal point
   reflectors that would be delta functions if correctly focused.
2. Circularly shift each windowed target to origin (centre-shifting).
3. Apply wide-enough window in along-track frequency.
4. The dominant singular vector of the stacked windowed responses
   is an estimate of `e^{iφ(u)}`; take its phase gradient, integrate,
   apply as correction in k-space, repeat.

PGA requires **3-6 iterations** and works well when (a) the aperture
is spotlight (scatterers seen by the whole aperture), and (b) the
fractional bandwidth is small (<10%). Neither holds for SAS.

### Why spotlight PGA fails for SAS (thesis § 10.1.4)

Four reasons identified by Callow:

1. **Stripmap ≠ spotlight.** Each scatterer is illuminated only for
   `L_SA = λR/D`, not the full pass. Stacking across the whole
   aperture averages over scatterers *not seen* in the target's aperture.
2. **Wide-beam blurring.** With 20-30° beams, the phase-history
   quadratic coefficient varies across the beam, so the "uniform
   phase error" model breaks.
3. **Wide-band blurring.** 40-60% fractional bandwidth means different
   frequencies see different effective beamwidths. One phase
   correction fits no frequency perfectly.
4. **Along-track position of target unknown.** Spotlight PGA assumes
   target-at-origin. In stripmap, each scatterer is at a different
   along-track position; naive centering destroys the phase history.

### SPGA core steps (thesis § 10.3)

```
SPGA(image g, iterations K=3-5):
  for k = 1..K:
    1. TARGET SELECTION
       - Identify M dominant scatterers by local magnitude + isolation
       - For each target, estimate its along-track position u_m
         (subpixel, via matched-filter centroid or correlation peak)
    2. ALONG-TRACK POSITION ESTIMATION (§10.3.3)
       - For each target m, circularly shift in along-track by −u_m
         to bring each target to origin
    3. WAVENUMBER-TRANSFORM COORDINATE CHANGE (§10.3.4)
       - Apply wavenumber (Stolt) remap: maps the space-variant
         stripmap blur model to a space-invariant form
       - In the wavenumber domain, each scatterer's contribution to
         phase error is the same function of k_u
    4. WINDOWING (§10.3.2)
       - Apply along-track frequency window width W_k to each target
       - W_k shrinks geometrically: W_{k+1} = α · W_k, α ≈ 0.6
       - Shrinking window trades off noise immunity (wider) against
         high-order phase capture (narrower)
    5. PHASE ESTIMATION (§10.3.5)
       - Stack windowed targets → matrix H ∈ C^{N_targets × N_k}
       - Dominant left singular vector gives phase error estimate
       - Alternative: phase-gradient maximum-likelihood (MLE) estimator
         (Jakowatz 1996): ĝ(u) = arg Σ_m g_m(u+1) · conj(g_m(u))
    6. APPLY CORRECTION
       - Multiply SLC in k-space by e^{-iφ̂}
       - Check sharpness metric; if not converged, iterate
```

The thesis goes further and discusses the **range-variant SPGA**
extension (§ 10.3.6) where the blur model is allowed to vary with
range, useful for wide-swath applications.

### Micronavigation track (thesis Ch 9) — DPCA / RPC / Shear-Average

SPGA is an **autofocus** (image-based, blind) method. The thesis
devotes Chapter 9 to the **micronavigation** alternative, where the
sonar has multiple receiver elements and the phase error is estimated
from the raw data before imaging:

- **Displaced Phase Centre Antenna (DPCA).** With N receivers
  separated by Δ physical aperture, consecutive pings at different
  positions produce pairs of pings that share a phase centre. The
  phase difference between the shared-centre pair is a direct
  measurement of platform sway, ping by ping.
- **Redundant Phase Centre (RPC).** Generalization of DPCA with
  multiple overlapping phase centres; averages out noise.
- **Shear-Average.** Callow's contribution: an image-domain variant
  that pairs successive pings at the phase-centre positions and
  averages complex shears. Provides 3-4× better high-contrast image
  micronavigation than classical DPCA.

DPCA/RPC needs hardware (multi-receiver array) but gives
deterministic, per-ping phase estimates. SPGA needs no hardware but
needs good image content and iterates.

## Key results — the numbers

### Simulated data (KiwiSAS-II parameters)

Parameters: 30 kHz carrier, 20 kHz bandwidth, 0.3 m element.
Injected sway: 0.2 m peak-to-peak random walk, correlation length
~1 m (10-15 pings).

| Metric | Before SPGA | After SPGA (3 iter) |
|---|---|---|
| Peak-to-sidelobe ratio | 7 dB | 22 dB |
| Azimuth resolution | 3 m blurred | 0.15 m (D/2) |
| Sway RMSE | n/a | <1 cm |
| Iterations to converge | — | 3–4 |

**SPGA converges to near diffraction-limited resolution (0.15 m =
D/2) in 3-4 iterations** for point-target scenes without clutter.

### Field data (Sydney Harbour, July 2001)

KiwiSAS-II data collected with DSTO Australia support. 20 m sonar
calibration rail + 3 m pipe + bland clutter background. Along-track
sampling D/3 (AASR ≈ −15 dB). No INS micronavigation available.

- SPGA brings calibration rail retro-reflectors from ~2 m blurs to
  near diffraction-limited response.
- The 3 m pipe (32, -11) to (32, -14) becomes resolvable as a
  extended target with front-wall and back-wall echoes visible.
- Bland clutter regions unchanged (no features to autofocus on).

### Failure modes documented (§ 10.6.3)

- **Undersampling (Δu = D/2, AASR −8 dB).** SPGA estimates
  contaminated by grating-lobe aliasing; linear and higher-order sway
  estimated incorrectly.
- **Bland clutter.** No dominant scatterers → no autofocus reference →
  no image improvement. Fundamental limit of image-based autofocus.
- **Extended targets.** Large, strong-scattering objects (calibration
  rail) are not point-like; stripmap PGA assumption breaks; SPGA
  needs to window onto a *single* retro-reflector within the rail.

## Strengths

- **Generalizes PGA to the regime SAS actually lives in.** Spotlight
  PGA was invented for SAR; before SPGA, SAS autofocus was a
  pastiche of spotlight PGA hacks (windowing, bandwidth
  pre-filtering) that degraded performance. SPGA is the first
  principled stripmap PGA.
- **Wavenumber-transform coordinate change** (§ 10.3.4) is the key
  insight: in the remapped coordinates, the stripmap blur model
  becomes space-invariant and spotlight-PGA-like phase estimation
  kernels apply. This is reusable beyond autofocus — it's the same
  coordinate change used in omega-k reconstruction.
- **Thesis-depth treatment.** 273 pages means every tradeoff and
  failure mode is documented with simulation + field data. The
  analysis of undersampling (§ 10.5) and bandwidth selection (§
  10.3.2) are particularly useful for operational tuning.
- **Micronavigation (Ch 9) bridges both worlds.** Autofocus (image
  domain) and micronavigation (signal domain) are presented as the
  two sides of the same coin, which is the right pedagogical framing.
- **Reproducibility.** The KiwiSAS-II parameters (Table 10.1) are
  the de-facto SAS simulation benchmark and have been used by many
  follow-up papers (Gerg-Monga 2021 uses KiwiSAS-style data).

## Limitations

- **Requires dominant scatterers.** SPGA (and any PGA variant) fails
  on bland scenes. Deep learning alternatives (Gerg-Monga 2021,
  Paper 5.3) explicitly address this limitation.
- **Iterative, not real-time.** 3-5 iterations @ ~0.3 s/iter on
  2003-era hardware; modern implementations hit ~0.1 s per iter on
  GPU but still iterative. Deep Autofocus (Paper 5.3) is
  single-iteration.
- **Sway-only model.** Yaw, heave, and medium fluctuation require
  extensions (range-variant SPGA § 10.3.6 handles yaw; heave
  requires different model; medium fluctuation is largely untreated).
- **Assumes stripmap geometry.** Spotlight, circular, and multistatic
  SAS need different derivations (though SPGA's coordinate-change
  idea partially generalizes).
- **Pre-ML era.** 2003 predates CNN, transformer, and GNN approaches
  to autofocus. Paper 5.3 (Gerg-Monga) shows deep nets beat SPGA on
  perceptual quality and 1-shot convergence — at the cost of
  needing a training dataset.

## Portable details — the math we will reuse

### Stripmap blurring model (§ 10.1)

The stripmap blurring kernel for a single scatterer at (x₀, y₀) with
along-track platform trajectory deviation s(u) (sway):

```
R(u; x₀, y₀) = √(y₀² + (u−x₀)²) + s(u) · cos θ(u)
             ≈ R_0(u; x₀, y₀) + s(u) · cos θ(u)
```

where θ(u) is the grazing angle, and to first order in s:

```
φ_err(u) = (4π/λ) · s(u) · cos θ(u)           [small-sway approx]
```

Under the small-sway approximation, the phase error becomes **range-
invariant** (to first order in beam angle) after the wavenumber
coordinate change — the whole point of SPGA.

### SPGA objective (§ 10.3.5)

Given M windowed targets `g_m(u) ∈ C^{N}` stacked as `H ∈ C^{M×N}`,
find the maximum-likelihood phase estimator:

```
φ̂(u) = arg max_{φ}  Σ_m  | Σ_{u'}  e^{-iφ(u')} · g_m(u') |²
```

Equivalently, the dominant left singular vector of `H` gives
`u_1 = e^{iφ̂} / √N`. Phase-gradient kernel (lower-variance):

```
∂φ̂/∂u (u) = arg (  Σ_m  g_m(u+1) · g_m*(u)  )
φ̂(u)      = cumulative sum of phase gradient
```

### Window shrinkage schedule

```
W_1 = full azimuth bandwidth B_a
W_{k+1} = α · W_k,       α = 0.6
Stop when  |Δφ̂|  <  threshold (e.g., λ/20)
```

### DPCA phase-error measurement (§ 9.1)

For two consecutive pings at positions u_n and u_{n+1} = u_n + Δu,
with matched phase centres c_1 and c_2 (identical physical aperture
positions), the returns `e_1(t)` and `e_2(t)` at those phase centres
should be identical under perfect motion. The measured difference:

```
Δφ̂_n = arg ( e_1(t) · e_2*(t) )
```

is a direct measurement of the sway Δs between ping `n` and `n+1`,
accurate to within the carrier wavelength λ.

### Sampling / AASR (§ 10.5)

Azimuth ambiguity-to-signal ratio for along-track sample spacing Δu:

```
AASR ≈ 10 log₁₀ ( sinc⁴(π · D / (2·Δu) )   — approximate
Δu = D/4  → AASR ≈ −21 dB     (fully-sampled, standard)
Δu = D/3  → AASR ≈ −15 dB     (typical KiwiSAS-II)
Δu = D/2  → AASR ≈  −8 dB     (undersampled, SPGA fails)
```

## Sonobuoy integration plan

### Where SPGA-style autofocus fits in a drifting-buoy SAS pipeline

A distributed sonobuoy aperture-synthesis stack has three natural
autofocus opportunities:

1. **Per-buoy micronavigation from GPS + IMU.** Each buoy has
   GPS-level position error ~2 m std and an IMU for finer
   interpolation. This is analogous to the INS/DVL micronavigation
   in an AUV-mounted SAS — coarse but metrology-based. Good for the
   outer-loop trajectory.
2. **Inter-buoy coherence checks (DPCA analog).** When two buoys
   happen to drift such that their receive positions overlap (to
   within λ/4), the returns from a common scatterer must be
   phase-coherent. Phase difference → measurement of residual
   clock skew, SSP error, or GPS bias. This is a
   **DPCA-across-buoys** and is the distributed analog of Callow
   Ch 9's redundant-phase-centre technique.
3. **Image-domain autofocus (SPGA).** After a first-pass
   reconstruction, apply SPGA (or Deep Autofocus, Paper 5.3) to the
   SLC to clean up residual phase error. The prerequisites — dominant
   scatterers — map cleanly onto the sonobuoy detection problem: we
   already identify DEMON-band propeller tonals as point sources, so
   the "find dominant scatterer" step is essentially the propeller
   localization the spatial GNN already does.

### What to port into the sonobuoy `weftos-sonobuoy-active` crate

A concrete SPGA-style autofocus module:

```rust
pub struct SpgaAutofocus {
    max_iterations: usize,         // default 5
    window_shrink_rate: f32,       // default 0.6
    convergence_threshold: f32,    // default λ/20 radians
    phase_estimator: PhaseEstimator,  // MLE or SVD or GradientKernel
}

impl SpgaAutofocus {
    pub fn autofocus(&self, slc: &mut Slc, targets: &[TargetHint])
        -> AutofocusReport { ... }
}

pub struct TargetHint {
    pub u_along_track: f32,   // initial along-track position
    pub y_range: f32,         // initial range
    pub confidence: f32,      // from GNN detector upstream
}
```

The `TargetHint` feed can come from the spatial GNN (Tzirakis /
Grinstein lineage in round 1) — unifying the active-imaging branch
with the passive-sonar localization branch.

### Expected performance bound for drifting-buoy SAS

With GPS position error 2 m std and 10 m synthesized aperture
(60 s of drift), the first-pass reconstruction has phase error
~4π · 2 / λ = 4π · 2 / 0.3 m = **~84 rad** at 5 kHz — enormously
defocused. Without autofocus, the image is unusable. SPGA with
target hints from the GNN detector should reduce residual to
~λ/10 ≈ 0.3 rad, recovering ~15 m along-track resolution (the
theoretical bound set by aperture length). **Autofocus is
mission-critical, not optional, for buoy SAS.**

## Follow-up references (second-degree citations)

1. **Wahl, D. E., Eichel, P. H., Ghiglia, D. C., & Jakowatz, C. V.
   (1994).** "Phase gradient autofocus — a robust tool for high
   resolution SAR phase correction." *IEEE Trans. Aerospace and
   Electronic Systems*, **30**(3), 827–835. DOI:
   10.1109/7.303752. — The seminal PGA paper that SPGA generalizes.
2. **Bellettini, A., & Pinto, M. (2002).** "Theoretical accuracy of
   synthetic aperture sonar micronavigation using a displaced phase
   center antenna." *IEEE JOE*, **27**(4), 780–789. — The DPCA
   foundational paper that Callow Ch 9 extends.
3. **Fortune, S. A. (2005).** "Phase error estimation for synthetic
   aperture imagery." PhD thesis, University of Canterbury. —
   Callow's group-mate's follow-up thesis on statistical
   autofocus, which feeds MNS (mean normalized stddev) sharpness
   metric that Gerg-Monga 2021 adopts.
4. **Marston, T. M., & Kennedy, J. L. (2014).**
   "Semiparametric statistical stripmap synthetic aperture
   autofocusing." *IEEE TGRS*, **53**(4), 2086–2095. DOI:
   10.1109/TGRS.2014.2353814. — NSWC-PC's refinement of SPGA with
   statistical scatterer model; cited in Gerg-Monga 2021 ref [11].
5. **Evers, A., Zelnio, E. G., & Jackson, J. A. (2019).** "A
   generalized phase gradient autofocus algorithm." *IEEE Trans.
   Computational Imaging*, **5**(4), 606–619. DOI:
   10.1109/TCI.2019.2913762. — Modernizes SPGA with a generalized
   likelihood-ratio framework, cited in Gerg-Monga ref [12].

---

*This analysis is Paper 5.2 of Round 2 (SAS) of the sonobuoy
literature survey. It provides the classical autofocus baseline that
Paper 5.3 (Gerg-Monga Deep Autofocus) is measured against, and
supplies the DPCA/RPC formalism that generalizes to inter-buoy
coherence checking in the distributed-sonobuoy setting.*
