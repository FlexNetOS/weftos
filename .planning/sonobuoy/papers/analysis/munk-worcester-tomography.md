# Munk & Wunsch 1979 — Ocean Acoustic Tomography

## Citation

### Primary (seminal paper)

- **Authors**: Walter H. Munk, Carl Wunsch
- **Title**: "Ocean acoustic tomography: a scheme for large-scale
  monitoring"
- **Venue**: *Deep-Sea Research Part A*, Vol. 26, No. 2 (1979),
  pp. 123–161
- **DOI**: https://doi.org/10.1016/0198-0149(79)90073-6
- **Open PDF (MIT Wunsch lab mirror)**:
  http://ocean.mit.edu/~cwunsch/papersonline/munk_wunsch_ocean_acoustic.pdf

### Textbook companion

- **Authors**: Walter Munk, Peter Worcester, Carl Wunsch
- **Title**: *Ocean Acoustic Tomography* (Cambridge Monographs on
  Mechanics)
- **Publisher**: Cambridge University Press, 1995 (2009 paperback)
- **ISBN**: 978-0-521-11536-0
- **Publisher link**:
  https://www.cambridge.org/core/books/ocean-acoustic-tomography/BF8D40998E590967FBF9BDD82ACD4E88

### Demonstration paper (1982)

- **Authors**: Peter F. Worcester, Walter H. Munk, Robert C. Spindel,
  and others
- **Title**: "A demonstration of ocean acoustic tomography"
- **Venue**: *Nature* 299:121-125 (1982)
- **DOI**: https://doi.org/10.1038/299121a0
- **Publisher**: https://www.nature.com/articles/299121a0

## Status

**Verified.** The 1979 *Deep-Sea Research* paper is indexed at
ScienceDirect under DOI 10.1016/0198-0149(79)90073-6 with the canonical
volume/issue/pages metadata; the open-access mirror at MIT is Wunsch's
own lab page. The 1982 *Nature* demonstration paper is verified via
DOI 10.1038/299121a0. The 1995 Cambridge monograph is verified via
ISBN 978-0-521-11536-0 (Cambridge catalog). All three are canonical
references in every subsequent OAT paper.

## Historical context

Munk and Wunsch's 1979 paper opened a genre: **inverting measured
travel times between a network of acoustic sources and receivers to
infer the interior sound-speed field (hence temperature and current)
of the ocean between them**, by direct analogy with X-ray computed
tomography. The idea was radical at the time — oceanography had been
dominated by point-measurement CTD casts that were spatially sparse
and expensive. OAT proposed that a modest number of sources and
receivers spread across an ocean basin would yield integral
measurements whose combinations constrain an interior 3-D field. The
1982 *Nature* demonstration (Worcester, Munk, Spindel) showed it
worked on a 300 km scale. By 1988 the reciprocal-transmission
experiments in the Greenland Sea, and later the basin-scale ATOC
(Acoustic Thermometry of Ocean Climate) experiments 1996-2006, made
OAT an operational climate-monitoring tool.

For the sonobuoy ranging project, OAT matters for two reasons.
(1) **Travel-time measurements are the native output of any LBL
ranging system**, so an OAT-style back-end can be bolted on for
free to recover c(z) along every path. (2) **OAT formalizes the
duality between positioning and SSP estimation** — if you know the
geometry exactly, you learn SSP; if you know SSP exactly, you learn
geometry; jointly estimating both is the natural regime for a
drifting buoy field.

## Core content

### The forward model

A single source-receiver pair produces, at the receiver, a
multipath arrival pattern. Each resolvable arrival `k` corresponds
to a ray path `Γ_k` from source to receiver with travel time

    τ_k = ∫_{Γ_k} ds / c(r, θ, z)

where `c(r, θ, z)` is the 3-D sound-speed field and `ds` is arc
length. If the sound-speed field differs from a reference `c₀(z)` by
a small perturbation `δc(r, θ, z)`, then (linearizing)

    δτ_k = τ_k - τ_k^{ref} ≈ -∫_{Γ_k^{ref}} (δc / c₀²) · ds

which is a **line integral of the perturbation along the reference
ray**. Stacking `K` source-receiver pairs × `M` resolved arrivals
per pair gives a vector `δτ ∈ R^{KM}` and

    δτ = G · m + ε

with `m` the discretized field (dim `P`, say on a grid), `G` the
`KM × P` path-integration matrix, and `ε` measurement noise. This
is the **tomographic forward operator** — formally identical to
X-ray CT, with paths replacing straight lines and sound speed
replacing X-ray attenuation.

### The inverse problem

With `KM < P` (typical: the field is richer than the measurements)
the problem is underdetermined. Munk and Wunsch solve it with
**Gauss-Markov inversion** (a.k.a. regularized least squares, a.k.a.
the EOF-Wiener filter):

    m̂ = C_m · G^T · (G · C_m · G^T + C_ε)^{-1} · δτ

where `C_m` is the a-priori field covariance (often EOF-based from
climatology) and `C_ε` is the measurement-noise covariance. The
posterior covariance

    Ĉ_m = C_m - C_m · G^T · (G · C_m · G^T + C_ε)^{-1} · G · C_m

gives calibrated uncertainties per grid cell. This is the
**Bayesian cornerstone** of all OAT work.

### Multipath as signal (not noise)

The big conceptual move of Munk-Wunsch 1979: the multipath arrivals,
which classical sonar processing treats as interference, are
**independent tomographic samples of c(r, z)**. A single
source-receiver pair might resolve 5-15 paths (direct + surface-
bounce + bottom-bounce + refracted-surface + refracted-bottom +
convergence-zone paths). Each path samples a different depth range
and launch angle; together they constrain the vertical SSP
structure. The trick is accurate ray-identification — each measured
arrival must be matched to a predicted reference path, typically
via ray-tracing in the reference SSP (BELLHOP or equivalent).

### What OAT measures

Acoustic travel times are sensitive primarily to temperature (~4
m/s per °C) and secondarily to current (∆τ_reciprocal gives line-
integrated current along the path). OAT therefore yields (a)
temperature field `T(r, θ, z, t)`, (b) ocean-current field
`v(r, θ, z, t)` via reciprocal transmissions, and (c) mixed-layer
depth / thermocline structure. The 1982 *Nature* demonstration
showed ~5 mK sensitivity on 300 km paths.

### Operational numbers from the 1982 demonstration

- **Range**: 300 km central path
- **Network**: 4 sources, 5 receivers (20 paths)
- **Carrier**: 225 Hz (low for range-dependent refraction minimization)
- **Bandwidth**: ~20 Hz (hence ~50 ms pulse compression)
- **Travel-time resolution**: ~5 ms per arrival
- **Temperature recovery**: ~5 mK RMS in 500 km² cells
- **Temporal cadence**: 1 transmission / 4 hr

### Operational numbers from ATOC 1996-2006

- **Range**: 3250 km (Kauai → Vertical Line Array in NE Pacific,
  Cornuelle-Worcester-Dushaw 1999 JASA 105:3202)
- **Source**: 75 Hz
- **Pulse**: 30 s M-sequence with ~200 ms effective resolution
- **Travel-time stability**: ~1 ms over 6-day observations
- **Temperature recovery**: ~5 mK over 1000-km² cells, updating
  continuously

## Portable details — the OAT inversion, applied to the inter-buoy
ranging problem

### Joint geometry-and-SSP estimation

For a sonobuoy field of `N` buoys, the measurement model includes
both travel times `τ_ij` (pair `i`→`j`, direct path) and buoy
positions `p_i`. The joint estimation state is

    x = (p_1, ..., p_N,   c(z) discretized at L depths)
           3N                  L

and measurements are the inter-buoy RTTs (or OWTT after clock sync).
The path-integral Jacobian requires a reference ray-trace per
pair (BELLHOP call), but for inter-buoy geometry — both ends near
the surface — the dominant path is the direct ray and one surface-
bounce, both computable analytically in a linearly-varying SSP.

### Reduced-dimension SSP parameterization

In practice OAT does not estimate `c(z)` at every grid point.
Munk-Wunsch recommend EOF expansion:

    c(z) = c₀(z) + Σ_{k=1..K} α_k · φ_k(z)

with `φ_k(z)` the leading EOFs from climatology (Levitus, WOA) and
`α_k` the unknowns. Typically `K = 3-5` captures >95% of seasonal
variability. The state dimension drops from L (~100) to K (~5),
making real-time estimation tractable on a sonobuoy.

### Rust skeleton

```rust
/// EOF-parameterized SSP.
#[derive(Debug, Clone)]
pub struct SspEof {
    pub depths_m: Vec<f64>,
    pub c_ref_mps: Vec<f64>,           // c₀(z)
    pub eofs: Vec<Vec<f64>>,           // K eigenfunctions φ_k(z)
    pub alpha: Vec<f64>,               // K coefficients
}

impl SspEof {
    pub fn c_at(&self, z_m: f64) -> f64 { /* interp */ todo!() }
    pub fn path_average(&self, path: &RayPath) -> f64 { /* ∫ds/c */ todo!() }
}

/// Joint buoy-position + SSP estimator. Extended Kalman filter
/// over the state x = (p_1..p_N, alpha_1..alpha_K).
pub struct GeometryAndSspEKF {
    pub positions: Vec<[f64; 3]>,
    pub ssp: SspEof,
    pub cov: Vec<Vec<f64>>,            // (3N+K) × (3N+K)
}

impl GeometryAndSspEKF {
    /// Ingest one RTT measurement between buoys i and j.
    pub fn update_rtt(&mut self, i: usize, j: usize, tau_ij: f64, sigma_tau: f64) {
        // 1. Ray-trace in current SSP to get reference τ_ij^ref and
        //    path-integration Jacobian ∂τ/∂α_k.
        // 2. Compute geometric Jacobian ∂τ/∂p_i, ∂τ/∂p_j.
        // 3. Standard EKF update.
        todo!()
    }
}
```

### What falls out for free

Because the same RTT measurements update both positions and SSP,
the sonobuoy field ends up producing an **in-situ SSP as a
by-product of ranging**, at ~the network cadence (0.1-1 Hz). This
SSP feeds directly into:

- Du-2023 Helmholtz-PINN (the physics-prior branch conditions on
  c(z)).
- Zheng-2025 FNO (6-channel input includes `k = ω/c`).
- KRAKEN / BELLHOP replicas for matched-field processing (Bucker
  1976, Capon 1969) — now the replicas use ground-truth in-situ
  SSP, not climatology.
- Active-imaging branch (Kiang-2022 multistatic SAS) phase
  reconstruction, which is c(z)-sensitive.

## Integration with the sonobuoy stack

OAT is the **second major gift** of active inter-buoy ranging. Where
LBL (Hunt-1974) gives meter-scale buoy positions, OAT gives an
in-situ sound-speed profile at the same cadence with no extra
hardware — the same RTT measurements, processed by a different
inverse. In the K-STEMIT-extended architecture this closes the loop
between the ranging subsystem and the physics-prior branch (Du-2023
Helmholtz-PINN, Zheng-2025 FNO): both solvers require c(z), and OAT
provides it directly rather than via climatological priors (ADR-059)
or ship-of-opportunity CTD casts. The EOF parameterization above
reduces the state dimension to ~5 coefficients per field, small
enough that the joint EKF over buoy positions + SSP coefficients
runs on-buoy in real time. The recovered SSP is fed back into the
BELLHOP/KRAKEN replica generation for the MFP baselines (Bucker-1976,
Capon-1969) — those classical methods now operate on ground-truth
geometry and ground-truth SSP, at which point their performance
should approach theoretical limits. A new EML-core operator
`eml_core::operators::oat_inversion` (Gauss-Markov posterior update)
becomes the trainable wrapper; it caches the path-integral
Jacobian keyed on `(ssp_hash, buoy_positions)`.

## Strengths

1. **Dual-use measurements** — the same pings that localize the
   buoys also estimate the SSP. Zero marginal cost.
2. **Calibrated uncertainty** — Gauss-Markov posterior covariance
   gives per-cell error bars, composable with the ECC ADR-059
   EML-core physics-prior.
3. **Well-posed under EOF reduction** — `K=3-5` EOF coefficients
   per field make the inverse small and stable.
4. **Long proven track record** — basin-scale (ATOC 1996-2006) and
   sub-basin (TRANSMIT 1991) demonstrations establish operational
   confidence.

## Limitations

1. **Linearization around a reference** — the Gauss-Markov step
   assumes small perturbations `δc ≪ c₀`. For sonobuoy fields in
   high-gradient thermoclines this can fail; iterative
   re-linearization helps.
2. **Ray identification is fragile** — each measured arrival must
   be matched to a predicted path. In shallow water with many
   surface/bottom bounces, ambiguity grows fast. Modern solutions
   use full-field matched processing (not implemented in
   Munk-Wunsch 1979).
3. **Requires broadband pulses** — 5 ms arrival resolution needs
   ~200 Hz bandwidth. Narrowband JANUS-style 80 Hz BFSK won't
   resolve adjacent multipaths. Sonobuoy ranging pings need LFM
   chirp at ≥200 Hz bandwidth to enable OAT as a side product.
4. **Reciprocal transmissions for currents** — recovering v(r, z)
   requires reciprocal (bidirectional) transmissions between every
   pair, doubling the ping budget. OWTT-only systems lose this.
5. **Temporal sampling** — OAT is traditionally slow (hours). For
   fast-varying thermocline structure a sonobuoy field would need
   sub-minute cadence, which is feasible but requires careful
   ping scheduling.

## Follow-up references

1. **Worcester, P.F., Munk, W.H., Spindel, R.C.** (1982). "A
   demonstration of ocean acoustic tomography." *Nature* 299:121.
   DOI: 10.1038/299121a0. The first at-sea OAT experiment.
2. **Munk, W., Worcester, P., Wunsch, C.** (1995). *Ocean Acoustic
   Tomography*. Cambridge Monographs on Mechanics. ISBN
   978-0-521-11536-0. The definitive textbook; Ch. 4 covers
   inversion, Ch. 7 covers ray identification.
3. **Cornuelle, B.D., Worcester, P.F., Dushaw, B.D., et al.** (1999).
   "Comparisons of measured and predicted acoustic fluctuations for
   a 3250-km propagation experiment in the eastern North Pacific
   Ocean." *JASA* 105(6):3202. DOI: 10.1121/1.424646. The ATOC
   Acoustic Engineering Test; modern operational reference.
4. **Dushaw, B.D., Worcester, P.F., Munk, W.H., et al.** (2009). "A
   decade of acoustic thermometry in the North Pacific Ocean." *J.
   Geophys. Res.* 114:C07021. DOI: 10.1029/2008JC005124.
5. **Wu, X. et al.** (2023). "Flow current field observation with
   underwater moving acoustic tomography." *Frontiers in Marine
   Science* 10:1111176. DOI: 10.3389/fmars.2023.1111176. Modern
   moving-node OAT; directly maps to a drifting sonobuoy field.
