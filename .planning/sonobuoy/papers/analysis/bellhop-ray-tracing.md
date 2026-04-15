# Porter & Bucker — Gaussian Beam Tracing / BELLHOP

## Citation

### Primary (seminal paper)

- **Authors**: Michael B. Porter and Homer P. Bucker
- **Title**: "Gaussian beam tracing for computing ocean acoustic fields"
- **Venue**: *Journal of the Acoustical Society of America*, Vol. 82,
  No. 4 (October 1987), pp. 1349–1359
- **DOI**: https://doi.org/10.1121/1.395269
- **Publisher link**:
  https://pubs.aip.org/asa/jasa/article/82/4/1349/786154/Gaussian-beam-tracing-for-computing-ocean-acoustic
- **Open preprint (Porter's lab page)**:
  https://www.hlsresearch.com/personnel/porter/papers/JASA/

### Companion (BELLHOP reference manual)

- **Author**: Michael B. Porter
- **Title**: *The BELLHOP Manual and User's Guide* (PRELIMINARY DRAFT)
- **Report**: HLS-2010-1, Heat, Light, and Sound Research, Inc., La
  Jolla CA, 31 January 2011
- **Open PDF**: http://oalib.hlsresearch.com/Rays/HLS-2010-1.pdf
- **Downloaded**: `.planning/sonobuoy/papers/pdfs/bellhop-ray-tracing.pdf`

### 3-D extension

- **Porter, M.B.** (2019). "Beam tracing for two- and three-dimensional
  problems in ocean acoustics." *JASA* 146(3):2016. DOI:
  https://doi.org/10.1121/1.5125262

## Status

**Verified.** The 1987 Porter-Bucker JASA paper is indexed at the AIP
publisher site with DOI 10.1121/1.395269 and is cross-referenced in
Semantic Scholar, the OALIB Acoustics Toolbox references, Porter's
lab page, and the NPL AC-12 propagation-models review (Wang 2014). The
2011 HLS-2010-1 manual is the official BELLHOP distribution
documentation on OALIB. The 2019 JASA 146:2016 paper is verified
independently via its DOI.

## Historical context

By the mid-1980s ray tracing in ocean acoustics was viable but plagued
by two artifacts inherited from geometrical optics: infinite-energy
caustics (where neighboring rays cross) and sharp-edged shadow zones.
Porter and Bucker's 1987 JASA paper ported to underwater acoustics the
**Gaussian-beam method** that Černý, Popov & Pšenčík had been
developing in seismology since the late 1970s. Rather than treating
each ray as a zero-width energy carrier, a Gaussian beam treats each
ray as the axis of a beam whose intensity falls off as a Gaussian
normal to the ray, with beam width and phase curvature governed by
auxiliary differential equations. This removes both artifacts at
almost no extra cost per ray.

Porter released BELLHOP as a C/Fortran implementation in the early 1990s;
it became the standard ray model in the OALIB Acoustics Toolbox (the
ray half; KRAKEN is the modal half). The 2011 HLS-2010-1 manual
consolidated 20 years of feature additions — bottom interaction,
directional sources, arrival-angle output — and is the document
currently distributed with BELLHOP. The 2019 JASA paper formalized the
BELLHOP3D extension to fully 3-D bottom-bathymetry problems that had
been accumulating in BELLHOP's head since roughly 2004.

BELLHOP is the workhorse propagation model for shallow-water,
high-frequency, and range-dependent problems where KRAKEN's modal
assumption fails — the regime most shallow-water sonobuoy deployments
actually occupy.

## Core content

### Classical ray tracing

A ray in an ocean with sound speed `c(x, z)` is a characteristic of
the eikonal equation `|∇τ|² = 1/c²`. In 2-D `(r, z)` with
`s =` arclength,

    dr/ds = c · ξ,         dξ/ds = -(1/c²) · ∂c/∂r
    dz/ds = c · ζ,         dζ/ds = -(1/c²) · ∂c/∂z

with `(ξ, ζ)` the ray slowness vector, `ξ² + ζ² = 1/c²`. Energy
propagates along rays; amplitude falls as `1/sqrt(J)` where `J` is the
geometrical-spreading Jacobian `J = ∂(r, z) / ∂(α₀, s)` with `α₀` the
launch angle. When `J → 0`, classical amplitude blows up (caustic);
when no ray reaches `(r, z)`, classical amplitude is zero (shadow).

### The Gaussian-beam fix (Porter-Bucker 1987)

Each ray is augmented with two auxiliary quantities — `p(s)` and
`q(s)` — obeying the **dynamic ray equations**:

    dp/ds = -(c_{nn} / c²) · q,     dq/ds = c · p

where `c_{nn}` is the second derivative of `c` normal to the ray.
The ratio `p/q` sets the complex beam curvature; `q` governs the
beam width `W(s) = Im(q·ε) / (k·Im(ε))` for a chosen Gaussian-beam
parameter `ε` (initial complex beam width).

The beam contribution at a field point at normal distance `n` from the
ray centerline is

    u_beam(s, n) = sqrt( c(s) / (c₀ · q(s)) )
                    · exp{ i·k · [ τ(s) + (p(s) / (2·q(s))) · n² ] }

and the total acoustic field is a sum over all launched beams:

    p(r, z) ≈ Σ_{rays} A_ray · u_beam(s, n)

Three consequences, from Porter-Bucker 1987 §III:

1. **Caustic smoothing**: when `q → 0` (caustic), the `1/sqrt(q)`
   amplitude remains finite via the `p/q` complex ratio. The result
   matches the `Airy-function` uniform-asymptotic correction for a
   simple caustic but at a fraction of the cost.
2. **Shadow-zone leakage**: beams have finite width, so they deposit
   energy into nominally-shadow regions. This matches measurement
   (where shadows are never truly silent due to scattering) much
   better than classical rays.
3. **No eigenray search**: one sums over launched beams rather than
   searching for rays that hit the receiver exactly. Big computational
   win for dense receiver grids (e.g., vertical-line arrays and
   distributed sonobuoy fields).

### BELLHOP features (HLS-2010-1 manual)

BELLHOP adds to Porter-Bucker 1987:

- Multiple beam types: `G` Gaussian, `B` hat-shaped beam, `C` Cerveny
  (complex source), `R` classical ray (diagnostic).
- Bottom and surface reflection with frequency-dependent reflection
  coefficients (halfspace, file-specified layered bottoms, elastic
  bottoms via KRAKEN mode-matching).
- Arrivals output: per-receiver `(time, amplitude, phase, launch angle,
  receive angle)` list; input to matched-filter replicas or to ping-
  arrival-angle estimators.
- Directional sources (source beam patterns as `*.sbp` files).
- Line- or point-source options.

### BELLHOP3D (Porter 2019)

Adds a third spatial dimension to everything above. The ray equations
become six coupled ODEs; `(p, q)` become 2x2 matrices. Bottom
interactions now involve a triangulated bathymetry mesh. This is
necessary whenever 3-D refraction matters — e.g., seamount shadow,
canyon focusing, littoral boundary refraction.

### When BELLHOP applies

BELLHOP is the correct solver when:
- Frequency is mid-to-high (say 500 Hz – 50 kHz), where modal count is
  large and ray methods are efficient,
- The environment is range- or fully-3-D-varying (bathymetry, SSP
  fronts, eddies),
- The application needs eigenrays, arrival-angle structure, or
  matched-filter replicas for ping ASW,
- Bottom interaction is geometric rather than deeply mode-coupled.

BELLHOP underpredicts below ~100 Hz in shallow water, where modal
interference dominates over ray arithmetic — use KRAKEN there.

## Modern relevance

- **Matched-field processing (MFP) and matched-field tracking**: BELLHOP
  provides the per-receiver arrival structure that MFP correlates
  against. Neilsen, Niu, Gemba 2021–2025 use BELLHOP replicas.
- **Training-data generators** for ML propagation: Zheng-2025 FNO
  surrogate (round 1 §2.3) is benchmarked against BELLHOP and RAM on
  range-dependent scenes. The 28.4% FNO speedup is vs BELLHOP at
  2 kHz and vs RAM at lower frequencies.
- **Differentiable ray tracing**: Bianco et al. 2020–2024 have ported
  BELLHOP's ray integrator into autodiff (JAX / PyTorch) for
  gradient-based inversion of SSP from measured arrival angles.
- **Acoustic localization**: Grinstein 2023 (round 1 §2.2) and similar
  GNN-based localizers use BELLHOP replicas as teacher data for
  unsupervised pretraining.
- **Simulation in the ML-sonar literature**: whenever a paper says
  "simulated ocean SSP + bathymetry", the simulator is almost certainly
  BELLHOP or RAM.

## Sonobuoy relevance

BELLHOP is the right-tool-for-the-job in the regime a sonobuoy field
most often occupies: shallow or mid-depth water, 500 Hz – 20 kHz,
variable bottom, drifting buoys at different depths. Four implications
for the clawft sonobuoy project:

1. **Mid-frequency propagation ground truth.** The physics-prior branch's
   ML surrogate (Zheng-2025 FNO) needs a BELLHOP reference in
   shallow water just as it needs a KRAKEN reference in range-independent
   deep water. The `eml_core::operators::` dispatch to the right solver
   based on SSP/bathymetry regime.
2. **Arrival-angle features.** BELLHOP's arrivals file
   `(time, amplitude, launch-angle, receive-angle)` per source-receiver
   pair is a high-value conditioning feature for the bearing head
   (Grinstein Relation-Net). It's cheaper than a full TL map and
   rotation/translation invariant within the array.
3. **Matched-filter replicas for active ping sonobuoys.** If the
   sonobuoy field is used with an active source (commandable or cooperating
   ship ping), the per-buoy matched-filter template is a BELLHOP
   arrival list convolved with the transmit waveform. This is the
   correct coherent-processing path to beat the incoherent classifier.
4. **3-D bottom refraction near littoral drops.** In shelf-break
   deployments, BELLHOP3D (2019) is the only way to capture the
   along-shelf refraction; the ML surrogate should include bathymetry
   gradient features to learn this regime.

Unlike KRAKEN, BELLHOP is relatively compact (~20k lines of Fortran)
and has been ported to Julia (AcousticsToolbox.jl), Python (pyat), and
Rust (acousticsx — partial). A native Rust integration is feasible in
a later sprint.

## Portable details

A minimal Rust binding around a BELLHOP binary. Per-ray state vector is
small (5 floats + 4 for dynamic equations) so BELLHOP replicas can be
generated on-buoy at ~1 kHz sample rate with modest memory.

```rust
/// Ray state for 2-D BELLHOP Gaussian-beam integration.
/// Integrated by 4th-order Runge-Kutta; see Porter-Bucker 1987 Eqs. 4-8.
#[derive(Debug, Clone, Copy)]
pub struct RayState {
    pub r_m: f64,           // range from source
    pub z_m: f64,           // depth
    pub xi: f64,            // horizontal slowness
    pub zeta: f64,          // vertical slowness
    pub tau_s: f64,         // travel time
    pub p: num::Complex<f64>,  // dynamic-ray p (beam curvature)
    pub q: num::Complex<f64>,  // dynamic-ray q (beam width)
}

/// SSP and bathymetry sampler. 2-D because BELLHOP2D; 3-D is a natural
/// extension.
pub trait Environment2D {
    fn sound_speed(&self, r_m: f64, z_m: f64) -> f64;
    fn sound_speed_gradient(&self, r_m: f64, z_m: f64) -> (f64, f64); // (∂c/∂r, ∂c/∂z)
    fn sound_speed_curvature_n(&self, r_m: f64, z_m: f64, launch_angle: f64) -> f64; // c_{nn}
    fn bathymetry(&self, r_m: f64) -> f64;
    fn sea_surface(&self, r_m: f64) -> f64 { 0.0 }
}

/// One Runge-Kutta step of the Porter-Bucker 1987 system.
pub fn step(state: RayState, ds: f64, env: &dyn Environment2D) -> RayState { /* ... */ todo!() }

/// Evaluate a Gaussian beam at a field point.
pub fn beam_contribution(
    ray: &[RayState],
    field_r_m: f64,
    field_z_m: f64,
    k: f64,
    epsilon: num::Complex<f64>, // initial beam parameter (imaginary part = initial width)
) -> num::Complex<f64> { /* ... */ todo!() }

/// Full BELLHOP: launch N rays, sum beam contributions at receiver grid.
pub fn bellhop(
    source: (f64, f64),            // (0, z_s)
    receivers: &[(f64, f64)],      // (r, z)
    env: &dyn Environment2D,
    freq_hz: f64,
    angle_range_deg: (f64, f64),   // launch-angle fan
    num_rays: usize,
) -> Vec<num::Complex<f64>> { /* ... */ todo!() }
```

Unit-test anchors (Porter-Bucker 1987 Fig. 6 — Munk SSP at 100 Hz):
- Canonical Munk profile, source at 1000 m, receiver at 1000 m, 100 Hz
- Convergence zone centered at ~60 km, width ~10 km
- Caustic smoothed to a ~3 dB bump, not an infinite spike
- TL at 30 km ≈ 85 dB (deep shadow, but non-zero via beam leakage)

## Follow-up references

1. **Červený, V., Popov, M.M., Pšenčík, I.** (1982). "Computation of
   wavefields in inhomogeneous media — Gaussian beam approach."
   *Geophys. J. Roy. Astron. Soc.* 70(1):109. DOI:
   10.1111/j.1365-246X.1982.tb06394.x. The seismology precursor
   Porter-Bucker adapted.
2. **Jensen, F.B., Kuperman, W.A., Porter, M.B., Schmidt, H.** (2011).
   *Computational Ocean Acoustics*, 2nd ed. Ch. 3 "Wavenumber
   Integration", Ch. 4 "Ray Methods". Springer. The textbook companion;
   the only consolidated reference that treats KRAKEN, BELLHOP, SCOOTER,
   and RAM on equal footing.
3. **Collins, M.D.** (1993). "A split-step Padé solution for the
   parabolic equation method." *JASA* 93(4):1736. DOI:
   10.1121/1.406739. RAM: the PE solver that complements BELLHOP for
   low-frequency range-dependent work.
4. **Thode, A.M.** (2019). "DIFAR azigrams for whale monitoring."
   *JASA* 145:3467. DOI: 10.1121/1.5110619. Uses BELLHOP replicas as
   the physics prior in DIFAR sonobuoy bearing estimation; direct
   clawft analog.
5. **Porter, M.B.** (2019). "Beam tracing for two- and three-dimensional
   problems in ocean acoustics." *JASA* 146(3):2016. DOI:
   10.1121/1.5125262. BELLHOP3D; needed whenever bathymetry-induced
   3-D refraction matters.
