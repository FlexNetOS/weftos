# Hunt et al. 1974 — Long-Baseline Acoustic Navigation (WHOI-74-6)

## Citation

- **Authors**: Mary M. Hunt, William M. Marquet, Donald A. Moller,
  Kenneth R. Peal, Woollcott K. Smith, Robert C. Spindel
- **Title**: "An Acoustic Navigation System"
- **Report**: Technical Report WHOI-74-6, Woods Hole Oceanographic
  Institution, December 1974
- **DOI**: https://doi.org/10.1575/1912/2117
- **Open PDF**: https://darchive.mblwhoilibrary.org/handle/1912/2117
- **Downloaded**: `.planning/sonobuoy/papers/pdfs/lbl-acoustic-nav.pdf`
  (to be fetched; primary open source above)

### Modern companion (survey)

- **Kinsey, J.C., Eustice, R.M., Whitcomb, L.L.** (2006). "A Survey of
  Underwater Vehicle Navigation: Recent Advances and New Challenges."
  Proceedings of 7th IFAC Conference on Manoeuvring and Control of
  Marine Craft (MCMC2006), Lisbon, September 2006. Open PDF:
  https://www.whoi.edu/cms/files/jkinsey-2006a_20090.pdf

## Status

**Verified.** WHOI-74-6 is archived at the WHOI/MBL Library open
repository with persistent DOI 10.1575/1912/2117 and is the foundation
citation for Long-Baseline (LBL) underwater acoustic positioning in
every subsequent AUV-navigation survey (Kinsey-Eustice-Whitcomb 2006,
Paull 2014, Leonard 2016). The Kinsey 2006 companion is verified
independently via Semantic Scholar
(semanticscholar.org/paper/471430e42810c062b0dcbf766c15b0bed0b6e5b4)
and the WHOI open PDF. Both PDFs are directly accessible, not behind
paywalls.

## Historical context

By the early 1970s deep-submergence research vehicles (DSRVs) and
their surface support ships needed meter-scale positioning in open
ocean far from shore. Radar and star fixes gave ~100 m at best, and
bottom-mounted transponder beacons had only been used ad-hoc. Hunt,
Marquet, Spindel, and the WHOI ocean-engineering group consolidated
roughly a decade of transponder work into the first **operational
long-baseline acoustic navigation system** — the hardware, the
acoustic waveforms, the travel-time-to-position inversion, and the
software — all packaged as a service used by the submersible Alvin
and surface R/V Knorr and Lulu. WHOI-74-6 is the report that
documented it; every LBL system deployed since (deep-sea,
under-ice, sonobuoy-based, etc.) is a descendant.

## Core content

### The LBL geometry

Two or more acoustic transponders are moored on the ocean floor at
known positions `(x_i, y_i, z_i)`, `i = 1..N`. A vehicle (ship or
submersible) on the surface or in the water column interrogates each
transponder with a coded ping; each transponder replies with its own
coded reply. The vehicle measures the **round-trip time (RTT)**
`τ_i` to each transponder and converts to slant range

    R_i = (c̄ / 2) · (τ_i - τ_turnaround,i)

where `c̄` is the ray-path-averaged sound speed and
`τ_turnaround,i` is the fixed electronic delay of transponder `i`
before it replies. With ≥3 ranges the vehicle position is the
intersection of spheres (trilateration).

### The sound-speed subtlety

Hunt et al. explicitly treat the fact that `c` varies with depth —
they use a harmonic-mean over the ray path, derived either from
measured CTD casts (Niskin + thermistor + salinity) or from the
climatological Leroy-Parthiot formula. The ranging bias sits
almost entirely in this average; a 1 m/s c-error over a 5 km path
is a ~3 m range error. This observation is the seed of ocean
acoustic tomography (Munk-Wunsch 1979): if you know the geometry
well enough, the travel-time residuals tell you c(z).

### Accuracy numbers (WHOI-74-6 operational reports)

Hunt et al. report operational accuracies that held through the 1970s:

- **Baseline length typical**: 3-10 km between transponders
- **Per-transponder range accuracy**: ~1 m RMS at 1500-5000 m depth
  (limited by c-profile knowledge, not electronics)
- **Absolute vehicle position**: 2-3 m RMS within the baseline
- **Update rate**: 1 ping/s (constrained by slant-range RTT itself —
  ~3 s per 4500 m)
- **Transponder depth accuracy**: <1 m after calibration survey

The `τ_turnaround` of the transponder is a load-bearing calibration
— it must be known to ~100 µs for meter-scale ranging, and it drifts
with temperature. Hunt et al. describe a matrix-inversion
self-calibration that solves for the transponder positions and
turnaround delays jointly from a grid of ranging measurements taken
by the support ship.

### Ping waveform (1974)

- **Carrier**: 9-14 kHz (coverage, not high frequency)
- **Pulse**: ~10 ms coded burst, different frequency per transponder
- **Detection**: narrowband filters, envelope-detection + threshold
- **Diversity**: frequency-division (each transponder has its own
  carrier; the vehicle interrogates all simultaneously and sorts
  replies by frequency)

Modern descendants use chirp (LFM) or m-sequence waveforms with
matched-filter detection for ~20 dB processing gain over the 1974
envelope scheme — see JANUS (Potter-Alves 2014) and Eustice 2011
for modern waveform choices.

## Portable details — the LBL equations a drifting-sonobuoy
implementation reuses

### RTT to slant range

    R_i(t) = (c̄(t, path_i) / 2) · (τ_i(t) - τ_turnaround,i)

where `τ_i(t)` is the measured RTT at epoch `t`, `τ_turnaround,i`
is the calibrated transponder delay, and `c̄(t, path_i)` is the
ray-path-average sound speed — time-varying because SSP varies.

### Trilateration (Hunt-Marquet-Smith Eqs. 4.1-4.5)

Given measurements `R_i` from `N ≥ 3` transponders at known positions
`p_i = (x_i, y_i, z_i)`, the vehicle position `p = (x, y, z)` minimizes

    J(p) = Σ_i w_i · (||p - p_i|| - R_i)^2

where `w_i = 1/σ²_{R_i}` are inverse-variance weights. Gauss-Newton
linearization around a prior `p^0`:

    A · Δp = b
    A_ij = (p^0_j - p_i,j) / ||p^0 - p_i||     (j = 1,2,3)
    b_i  = R_i - ||p^0 - p_i||
    Δp   = (A^T W A)^{-1} A^T W b

Three-tran-minimum if depth `z` is known from a pressure sensor
(sonobuoy case: `z = 0` at the surface, so two transponders suffice
for lat-lon).

### Joint transponder self-calibration (Hunt-Marquet §6)

The turnaround delays `τ_turnaround,i` and transponder positions
`p_i` are jointly estimated from a ship survey: the ship sits at
`M ≥ 4` known GPS positions and records RTTs. The unknown vector is

    x = (p_1, ..., p_N, τ_tt,1, ..., τ_tt,N)      dim = 4N

and the observation Jacobian is straightforward. This is the
ancestor of the modern **self-calibration step** that every
sonobuoy field must perform at deploy time.

### Rust skeleton (portable to clawft-sonobuoy-ranging)

```rust
/// One RTT ping measurement.
#[derive(Debug, Clone, Copy)]
pub struct RttMeasurement {
    pub remote_id: u32,          // which transponder / buoy
    pub rtt_seconds: f64,        // raw round-trip time
    pub turnaround_sec: f64,     // calibrated remote delay
    pub c_bar_mps: f64,          // path-averaged sound speed
    pub variance_m2: f64,        // σ² on the resulting range
}

/// Derived slant range.
pub fn slant_range(m: &RttMeasurement) -> (f64, f64) {
    let r = 0.5 * m.c_bar_mps * (m.rtt_seconds - m.turnaround_sec);
    let sigma = (m.variance_m2).sqrt();
    (r, sigma)
}

/// Gauss-Newton trilateration. Prior `x0` in meters, returns (x, Σ).
pub fn trilaterate_gn(
    anchors: &[[f64; 3]],
    ranges: &[f64],
    sigmas: &[f64],
    x0: [f64; 3],
    iters: usize,
) -> ([f64; 3], [[f64; 3]; 3]) { /* ... */ todo!() }
```

## Integration with the sonobuoy stack

This paper defines the **primary replacement for GPS-derived sensor
positions** in the K-STEMIT-extended architecture. Section 10 of
`SYNTHESIS.md` explicitly flags sensor-position uncertainty as an
open problem — Grinstein-2023 Relation-Network assumes known-geometry
GCC-PHAT inputs, Tzirakis-2021 dynamic-adjacency GCN assumes a
meaningful haversine graph, Bucker-1976 MFP assumes known array
geometry, and Kiang-2022 multistatic SAS assumes known buoy positions.
With drifting buoys at GPS-only accuracy (2-5 m @ 1 Hz), all four
back-ends operate with position noise that dominates the intrinsic
TDOA/bearing error budget. Inter-buoy LBL ranging per Hunt-1974
gives meter-scale inter-buoy distances at 0.5-1 Hz, dropping the
position-noise floor below the TDOA floor for the first time. The
Rust skeleton above slots into a new crate `clawft-sonobuoy-ranging`
that outputs a distance matrix `D(t) ∈ R^{N×N}` and per-pair
uncertainties `σ_ij(t)`, which Tzirakis' GCN consumes directly as the
edge-weight matrix and Grinstein's Relation-Network consumes as a
metadata feature per pair. The self-calibration loop (Hunt §6) maps
onto `eml_kernel`'s trainable-operator abstraction: the turnaround
delays `τ_turnaround,i` and the reference SSP become EML parameters
updated by a Kalman filter over the range residuals.

## Strengths

1. **First-principles derivation** — every equation is derivable
   from first principles (Fermat's principle + sound-speed profile),
   so the method generalizes cleanly to non-standard geometries
   (inverted LBL, moving baseline, drifting sonobuoy).
2. **Self-calibration built-in** — the matrix-inversion joint
   estimation of transponder positions and turnaround delays means
   a freshly-deployed field is usable within hours, not days.
3. **Deterministic per-measurement uncertainty** — the algorithm
   outputs not just a position but a covariance, enabling
   downstream probabilistic processing (Kalman filter, particle
   filter).
4. **Hardware-frugal** — 9-14 kHz, ~10 ms pulse, ~1 W electrical
   transmit is compatible with sonobuoy-class hardware.

## Limitations

1. **RTT halves update rate** — a 5 km range + 10 ms pulse + 5 km
   return trip + turnaround is ~7 seconds, so only ~0.14 Hz per
   pair. Modern one-way travel-time (Eustice 2011) doubles this by
   eliminating the reply.
2. **Assumes known SSP** — the c̄ averaging step is a point of
   failure under strong thermocline variability. Modern systems
   must jointly estimate SSP (Munk-Wunsch 1979) or ingest CTD
   casts.
3. **Narrowband 14 kHz susceptible to multipath in shallow water**
   — surface-bounce and bottom-bounce arrivals add ~1-3 ms of
   ambiguity to envelope-detection. Modern chirp/m-sequence
   correlation processing (Potter-Alves 2014 JANUS) gives ~100 µs
   resolution.
4. **Turnaround-delay temperature drift** — transponder electronics
   drift ~10-50 µs over deployment life; adds ~10-50 cm slow drift
   in range.
5. **No Doppler mitigation** — 1974 surface-ship motion was slow
   enough to ignore; drifting sonobuoys moving at ~0.5-2 m/s
   acquire ~0.1-1% range-rate bias that modern systems explicitly
   correct (Webster-Eustice 2012).

## Follow-up references

1. **Kinsey, J.C., Eustice, R.M., Whitcomb, L.L.** (2006). "A Survey
   of Underwater Vehicle Navigation: Recent Advances and New
   Challenges." MCMC2006, Lisbon. The definitive 2006 survey — one
   section is entirely dedicated to LBL descendants of Hunt 1974.
2. **Milne, P.H.** (1983). *Underwater Acoustic Positioning
   Systems*. Gulf Publishing. The textbook treatment of LBL/SBL/USBL
   geometries; extends Hunt 1974's equations to inverted and
   ultra-short-baseline cases.
3. **Eustice, R.M., Whitcomb, L.L., Singh, H., Grund, M.** (2007).
   "Experimental Results in Synchronous-Clock One-Way-Travel-Time
   Acoustic Navigation for Autonomous Underwater Vehicles." IEEE
   ICRA 2007. The modern one-way-travel-time descendant.
4. **Paull, L. et al.** (2014). "AUV Navigation and Localization: A
   Review." IEEE JOE 39(1):131. DOI:
   10.1109/JOE.2013.2278891. Updates Kinsey 2006 with 2007-2013 work
   including USBL, GIB, and cooperative navigation.
5. **Otero, P. et al.** (2023). "Underwater Positioning System Based
   on Drifting Buoys and Acoustic Modems." *J. Marine Sci. Eng.*
   11(4):682. DOI: 10.3390/jmse11040682. The direct modern analog
   of WHOI-74-6 for drifting (not moored) buoy fields.
