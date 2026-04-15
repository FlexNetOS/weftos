# Paper R2.1 — Bucker 1976 (Foundational Matched-Field Processing)

## Citation

Bucker, H. P. (1976). "Use of calculated sound fields and matched-field
detection to locate sound sources in shallow water." *Journal of the
Acoustical Society of America*, **59**(2), 368–373.
DOI: [10.1121/1.380872](https://doi.org/10.1121/1.380872).
Author affiliation: Code 503, Naval Undersea Center, San Diego, CA.

## Status

**Verified.** DOI confirmed via ADS abstract record
[1976ASAJ...59..368B](https://ui.adsabs.harvard.edu/abs/1976ASAJ...59..368B/abstract)
and AIP/JASA publisher record (publisher paywall). This paper is
universally cited as the origin of matched-field processing (MFP) in
underwater acoustics. Baggeroer, Kuperman & Mikhalevsky's 1993 IEEE JOE
review names Bucker 1976 as the first matched-field localization
method. PDF not obtained — JASA / AIP paywall (403 on institutional
mirrors).

## Historical context

Before 1976, underwater passive source localization relied on
**plane-wave beamforming**: assume a planar wavefront crossing the
array, steer a delay-and-sum (or Capon 1969 MVDR) beamformer over
elevation/azimuth, read off bearing. The plane-wave assumption fails
in shallow water for three reasons:

1. **Multipath.** In shallow water (roughly D < 200 m, frequency
   30–500 Hz) the acoustic field is not a single ray; it is a
   superposition of bottom-reflected, surface-reflected, and direct
   rays. Within a few wavelengths of the source, the field looks
   planar; beyond ~5–10 water depths it becomes a modal interference
   pattern, and a plane-wave model is simply wrong.
2. **Normal modes.** Shallow-water waveguide propagation is well-
   described by modal theory (Pekeris, Tolstoy, Kuperman): the field
   at range r and depth z decomposes into a discrete sum of modes
   whose horizontal wavenumbers k_m depend on frequency and the
   sound-speed profile (SSP). Classical bearing estimation cannot
   recover range *or* depth from modal interference — it can only
   estimate the horizontal arrival angle of the first mode, badly.
3. **Depth discrimination.** The Navy wanted to tell submarines
   (deep) from surface ships (shallow) from a bottom-mounted array.
   Plane-wave beamforming is fundamentally incapable of estimating
   source depth because depth does not appear in the plane-wave
   steering vector.

Bucker's insight, framed in 1976 JASA: if you **compute** the expected
complex pressure field **p(r, z; r_s, z_s, ω)** that a source at
(range r_s, depth z_s) would produce at each hydrophone in your array,
you can correlate that *predicted* field against the *measured* field
and declare a detection when the correlation peaks. The steering
vector is no longer a delay-and-sum plane wave; it is the **full
Green's function of the waveguide**, including modal structure. The
peak of the correlation surface directly gives range and depth — no
triangulation required, one vertical line array (VLA) suffices.

This was radical for three reasons. First, it made localization a
**model-based** problem: you need a reasonable acoustic propagation
model (normal-mode code, parabolic-equation code, or ray trace) and an
SSP estimate before you can localize. Second, it fused bearing, range,
and depth estimation into a single unified framework. Third, it
replaced *angle-of-arrival* with *(range, depth)* as the primary
output — which is what anti-submarine warfare (ASW) actually cares
about.

Bucker's original formulation was the **Bartlett (linear) matched-
field processor**. Capon's 1969 MVDR adaptation (MFP-MV, or adaptive
MFP) followed in the late 1980s after Fizell, Klemm, Bucker himself,
and others proposed it.

## Core math

### Signal model at one frequency ω

Consider a vertical line array of N hydrophones at depths
z_1, ..., z_N, horizontal range 0 (the array is the origin). A
narrowband point source at range r_s, depth z_s, radiating at ω,
produces a complex pressure at hydrophone i of

```
  p_i(ω; r_s, z_s) = G(r_s, z_s → 0, z_i; ω) · S(ω)
```

where G is the **acoustic Green's function** of the ocean waveguide
for the given SSP, bathymetry, bottom properties, and surface
conditions, and S(ω) is the source amplitude. In normal-mode form
(Bucker's chosen basis):

```
  G(r, z_s → 0, z; ω) ≈ (i/4ρ(z_s)√(8π r))
                         · Σ_m  U_m(z_s) U_m(z) · e^{i k_m r} / √k_m
```

where U_m(z) is the depth eigenfunction of mode m, k_m its horizontal
wavenumber, and ρ(z_s) the density at the source depth. The **replica
vector** (Bucker's term: "calculated sound field vector") stacking all
N hydrophone responses is

```
  d(r_s, z_s; ω) = [ G(r_s, z_s → 0, z_1; ω),
                     G(r_s, z_s → 0, z_2; ω),
                     ...,
                     G(r_s, z_s → 0, z_N; ω) ]^T  ∈ ℂ^N
```

Normalized: **w(r_s, z_s; ω) = d(r_s, z_s; ω) / ‖d(r_s, z_s; ω)‖**.
This is the steering vector for MFP. Unlike the plane-wave steering
vector a(θ) = [1, e^{jφ}, ..., e^{j(N−1)φ}]^T (which lives on a
2-parameter manifold of direction), the MFP steering vector lives on
a (≥)2-parameter manifold of source location (r_s, z_s) embedded in
ℂ^N via the waveguide physics.

### Measurement and sample covariance

At the array, we snapshot the hydrophone outputs K times:

```
  y(t_k; ω) = d(r_s, z_s; ω) S_k(ω) + n(t_k; ω),   k = 1, ..., K
```

The sample cross-spectral-density matrix (SCM) — the underwater
acoustician's name for the spatial covariance matrix — is

```
  R̂(ω) = (1/K) Σ_k  y(t_k; ω) y(t_k; ω)^H       ∈ ℂ^{N×N}
```

### Bucker's detection factor (Bartlett MFP)

Bucker proposed a **detection factor** DF that measures how well the
measured field matches the replica field at candidate location
(r, z). In modern notation:

```
  B_MFP(r, z; ω) = w(r, z; ω)^H  R̂(ω)  w(r, z; ω)
```

This is the **Bartlett (conventional) matched-field ambiguity
surface**. When the true source is at (r_s, z_s), w(r_s, z_s) aligns
with the dominant eigenvector of R̂ and B_MFP peaks. Candidate
locations far from the true source produce low values because their
replica vectors are (approximately) orthogonal to the measured field.

Bucker's 1976 paper demonstrated DF on simulated shallow-water cases
and showed that the ambiguity surface in (r, z) has a clear peak at
the true source location, with sidelobes at locations where the
waveguide modal structure accidentally matches. The **side-lobe
level** problem is central to MFP and motivates the adaptive MFP
variants (MFP-MV, MCM) that followed.

### Adaptive MFP (MVDR form, post-Bucker extension)

Although Bucker 1976 used Bartlett, the natural extension — invented
later — is to apply Capon's MVDR weights to the MFP steering vectors:

```
  w_MV(r, z; ω) = R̂(ω)^{-1} w(r, z; ω)
                  / ( w(r, z; ω)^H  R̂(ω)^{-1}  w(r, z; ω) )
```

```
  B_MVDR-MFP(r, z; ω) = 1 / ( w(r, z; ω)^H  R̂(ω)^{-1}  w(r, z; ω) )
```

This gives much sharper ambiguity peaks and rejects correlated
interference (e.g., surface shipping) by nulling in the covariance
inverse. The trade-off is severe **mismatch sensitivity**: if the
true Green's function differs from the replica (wrong SSP, wrong
bathymetry, wrong bottom), the MVDR null can accidentally cancel the
true target.

### Broadband incoherent MFP

For a broadband source over frequencies ω_1, ..., ω_L, Bucker's
framework generalizes by *incoherent* frequency averaging:

```
  B_MFP(r, z) = (1/L) Σ_l  w(r, z; ω_l)^H  R̂(ω_l)  w(r, z; ω_l)
```

Coherent broadband MFP (Michalopoulou, Collins, Baggeroer) adds phase
alignment across frequencies and gives better sidelobe suppression
but requires knowledge of the source spectrum.

## Strengths

1. **Unified bearing + range + depth estimation.** One algorithm, one
   vertical line array, full 3D source localization — a capability
   plane-wave beamforming cannot provide at any cost.
2. **Exploits multipath constructively.** The modal interference
   pattern that breaks plane-wave processing is precisely what MFP
   uses: the richer the waveguide, the more discriminative the
   ambiguity surface.
3. **Model-based, interpretable.** Ambiguity surfaces are physically
   meaningful — peaks, sidelobes, and mismatch signatures can be
   reasoned about by an acoustician.
4. **Extensible.** The framework absorbs MVDR (adaptive MFP), minimum
   cross-entropy, maximum likelihood, and sparse Bayesian learning
   variants. All share the same replica-vector structure.
5. **Dense training data unnecessary.** MFP is fundamentally a
   model-based detector; it needs no examples of real sources. This
   is the key contrast with deep-learning MFP — MFP works when you
   have only a propagation model and zero labeled data.

## Limitations

1. **Environmental mismatch is catastrophic.** SSP errors of
   ~1 m/s over the water column, bathymetry errors of ~10 m, or
   bottom sound-speed errors of ~50 m/s can degrade MFP from a
   useful detector to no better than random guessing. This is MFP's
   Achilles heel and the entire motivation for deep-learning MFP.
2. **Compute-heavy forward modeling.** Every candidate (r, z) pair
   requires a full normal-mode or PE field computation. For a
   1000×100 range-depth grid, that is 10^5 forward runs, repeated
   per frequency. Modern GPU-accelerated or neural-operator
   surrogates (FNO, PINN) address this.
3. **No uncertainty quantification.** Bucker's DF is a point
   statistic; there is no principled confidence interval on the
   (r̂, ẑ) estimate without extending to maximum-likelihood or
   Bayesian MFP.
4. **Assumes stationary environment.** The SSP changes on
   minute-to-hour timescales in shallow water (thermocline
   migration, internal waves). Classical MFP does not track this;
   adaptive / environmental-focalization MFP variants do but pay a
   severe compute cost.
5. **VLA assumption.** The original 1976 formulation assumes a
   **vertical** line array. Horizontal line arrays (HLAs) have range
   ambiguity and cone-of-uncertainty problems; distributed/drifting
   arrays (the sonobuoy case) are even harder because geometry is
   time-varying.

## Modern relevance: how GNN/DL beamformers build on or replace MFP

The three round-1 ML-beamforming analyses in this folder should be
read as *direct successors* to Bucker 1976:

- **Tzirakis 2021 GNN-BF** (`gnn-bf.md`). Replaces the hand-designed
  replica vector w(r, z; ω) with a **learned graph aggregation**:
  the edge weights A_ij between mics are learned from the data, not
  computed from a waveguide model. This *defers* the propagation
  physics into the network weights, trading interpretability for
  robustness to geometry. Beats MVDR (Capon) under reverberation
  because reverb is exactly the shallow-water multipath regime that
  broke plane-wave beamforming in 1976 and that Bucker's MFP was
  designed to exploit — GNN-BF learns to exploit it data-
  driven-style instead.
- **Grinstein 2023 Relation-Network SLF** (`gnn-tdoa-uncertain.md`).
  Compared explicitly against the Steered-Response-Power / GCC-PHAT
  classical baselines, which are the plane-wave beamformer
  cousins of MFP. Extends MFP-style likelihood fields to settings
  where sensor geometry is uncertain — directly addresses the
  sonobuoy drift problem.
- **Chen & Rao 2025 Grassmann-subspace DoA** (`neural-beamforming-
  sparse.md`). Generalizes MUSIC (1986), not MFP per se, but the
  subspace-learning framework absorbs MFP as a special case:
  replace plane-wave manifold with waveguide manifold, learn the
  manifold from data.
- **Sun et al. 2024** (MDPI Remote Sensing, DOI
  [10.3390/rs16081391](https://doi.org/10.3390/rs16081391); see
  `modern-ml-mfp.md`). Explicit deep-learning *MFP* replacement,
  trading replica-vector correlation for a CNN (IRSNet) that
  estimates depth and range from a multi-time pressure+eigenvector
  feature. Claims dramatically better robustness than Bartlett/MVDR
  MFP under environmental mismatch.

The common thread: **classical MFP is the optimal (maximum-
likelihood) detector if and only if the replica is correct; when the
replica is wrong, any learned approximation that saw enough mismatch
during training outperforms it.** Bucker 1976 is the "MFP is
optimal" half; the 2020s deep-learning papers fix the "replica is
wrong" half.

## Sonobuoy integration plan

MFP was designed for **fixed vertical line arrays** — ideally a VLA
moored to the seafloor with known hydrophone depths and a stable
local SSP measurement. Sonobuoys violate all three assumptions:

1. They **drift**. The array geometry changes over the deployment
   lifetime (minutes to hours).
2. They are **horizontal-plane sparse and distributed**. A sonobuoy
   field is a 2-D planform of (x_n, y_n) surface positions, each
   with a short hanging hydrophone at ~10–300 m depth.
3. Each buoy **does not know** its own position to better than the
   drift-corrected GPS update interval; relative timing requires
   inter-buoy clock synchronization (DIFAR buoys broadcast VHF with
   GPS-disciplined timestamps, but there is still 1–10 ms jitter).

### Adapting MFP to a drifting sonobuoy field

**Step 1 — Treat buoy network as a virtual VLA + HLA hybrid.**
Even though each sonobuoy has only one (or a few, in DIFAR) closely
spaced hydrophone(s), *across* the sonobuoy field you have a
horizontally distributed hydrophone set. The replica vector becomes:

```
  d(r_s, z_s; ω) = [ G(r_s, z_s → (x_n, y_n), z_n; ω) ]_{n=1..N}
```

where (x_n, y_n, z_n) are the live buoy positions at the
snapshot time and r_s = √((x_s − x_n)² + (y_s − y_n)²) per buoy.
The **(x_s, y_s, z_s)** search space replaces Bucker's (r_s, z_s).

**Step 2 — Model the array geometry uncertainty.** Let
(x_n, y_n) = (x̂_n, ŷ_n) + δ_n where δ_n ~ 𝒩(0, σ²I) is drift/GPS
error. The SCM conditional on δ_n, combined with the prior, gives a
**marginal likelihood** replica:

```
  d̄(r_s, z_s) = E_δ [ d(r_s, z_s; δ) ]
```

This is the geometry-uncertain MFP of Grinstein et al. (2023) in
MFP form.

**Step 3 — Neural replica.** Replace G(...) with a neural Green's
function (FNO-propagation or PINN-SSP-Helmholtz, already in our
library at `fno-propagation.md` and `pinn-ssp-helmholtz.md`). This
gives a differentiable forward model that is ~1000× faster than
normal-mode computation per replica.

**Step 4 — GNN-BF as the inner beamformer.** Rather than Bartlett or
MVDR, pass the buoy-network covariance R̂ through a learned GNN
(Tzirakis 2021 style) whose adjacency A_ij is conditioned on the
live (x_n, y_n, z_n, τ_n-clock) per buoy. The GNN outputs per-buoy
weights that *implicitly* account for both Bucker replica physics
**and** geometry drift.

**Step 5 — Incoherent broadband fusion.** Sum over frequency bins in
the source band (e.g., 100–2000 Hz for submarine tonals, ~50–500 Hz
for baleen whales). Incoherent is far more robust than coherent
under clock jitter.

**Step 6 — Track, don't detect.** Integrate the per-snapshot
ambiguity surfaces into a Bayesian tracker (IMM, JPDAF) using the
source-motion dynamics; this suppresses ambiguity-surface sidelobes
temporally. This is the "MFP track-before-detect" variant
referenced in the survey (Michalopoulou 2002 JASA).

## Portable details

### Full steering-vector math (VLA case)

Vertical line array at (0, 0) with depths z_1, ..., z_N. Source at
(r_s, z_s). For a stratified SSP c(z) giving M propagating modes,
each with eigenfunction U_m(z) and horizontal wavenumber k_m:

```
  G(r_s, z_s → 0, z_i; ω) = (i · e^{-iπ/4})/(ρ(z_s) √(8π r_s))
                             · Σ_{m=1..M}  U_m(z_s) U_m(z_i)
                                           · exp(i k_m r_s)
                                           / √k_m
```

The modes (U_m, k_m) are found by solving the depth-separated
Sturm-Liouville problem

```
  d²U_m/dz² + [(ω/c(z))² − k_m²] U_m(z) = 0
```

with boundary conditions U_m(0) = 0 (pressure-release surface) and
an impedance or finite-density condition at the bottom. The KRAKEN
or ORCA normal-mode codes are the standard implementations.

### Sample-covariance estimation

With K snapshots at frequency ω:

```
  R̂(ω) = (1/K) Σ_{k=1..K} y(t_k, ω) y(t_k, ω)^H
```

For K < N (undersampled regime, typical for vertical arrays with
only seconds of data), R̂ is rank-deficient and MVDR-MFP fails.
Remedies:

- **Diagonal loading**: R̂_DL = R̂ + ε I, ε ≈ 10 · σ²_noise
- **Subspace truncation**: project onto top-p eigenvectors of R̂
- **Shrinkage (Ledoit-Wolf)**: R̂_shrink = α R̂ + (1−α) σ̂² I
- **Multi-snapshot averaging** across time-overlapping FFT windows

### Bartlett (Bucker) weight and output

```
  w_B(r, z; ω) = d(r, z; ω) / ‖d(r, z; ω)‖
  B_B(r, z; ω) = w_B(r, z; ω)^H  R̂(ω)  w_B(r, z; ω)
```

### MVDR (adaptive) MFP weight and output

```
  w_MV(r, z; ω) = R̂(ω)^{-1} d(r, z; ω)
                   / ( d(r, z; ω)^H  R̂(ω)^{-1}  d(r, z; ω) )
  B_MV(r, z; ω) = 1 / ( d(r, z; ω)^H  R̂(ω)^{-1}  d(r, z; ω) )
```

### Localization

```
  (r̂_s, ẑ_s) = argmax_{(r, z)}  B(r, z; ω)
```

In practice: grid search on (r, z) × {ω_l}, incoherent-sum across
frequencies, detect peak, verify peak height exceeds a threshold
calibrated to ~1% false-alarm rate.

### Performance metric: MFP gain

Array gain of MFP over an omnidirectional single hydrophone for a
coherent source:

```
  AG_MFP = 10 log_{10}( N · SNR_out / SNR_in )
```

For a perfectly matched replica, AG_MFP → 10 log_{10} N (full array
gain). Under small mismatch, AG drops; under gross mismatch (SSP
error > 2 m/s, bathymetry > 20 m), AG can go negative.

## Follow-up references

1. **Baggeroer, A. B., Kuperman, W. A., & Mikhalevsky, P. N. (1993).**
   "An overview of matched field methods in ocean acoustics."
   *IEEE Journal of Oceanic Engineering*, 18(4), 401–424.
   DOI: [10.1109/48.262292](https://doi.org/10.1109/48.262292).
   The definitive MFP review — 24 pages, covers Bartlett, MVDR, MCM,
   sector-focused, environmental focalization, broadband coherent
   and incoherent variants.
2. **Collins, M. D., & Kuperman, W. A. (1991).** "Focalization:
   Environmental focusing and source localization." *JASA*, 90(3),
   1410–1422. DOI:
   [10.1121/1.401933](https://doi.org/10.1121/1.401933). The
   classical mismatch-mitigation paper: simultaneously estimate
   environment and source location.
3. **Tolstoy, A. (1993).** *Matched Field Processing for Underwater
   Acoustics.* World Scientific, Singapore. ISBN 978-981-02-1159-1.
   The only book-length MFP treatment.
4. **Michalopoulou, Z.-H. (2000).** "The effect of source amplitude
   and phase in matched field source localization." *JASA*, 107(5),
   2563–2575. DOI:
   [10.1121/1.428642](https://doi.org/10.1121/1.428642). Coherent
   broadband MFP.
5. **Wang, Y., et al. (2021).** "Deep-learning source localization
   using multi-frequency magnitude-only data." *JASA*, 149(5),
   3480–3489. DOI:
   [10.1121/10.0005127](https://doi.org/10.1121/10.0005127). Early
   neural-MFP that replaces the replica correlator with a CNN.
