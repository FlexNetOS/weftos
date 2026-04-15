# Cornuelle-Worcester-Dushaw 1999 + Modern ML SSP Inversion — SSP from Ranging Residuals

## Citation

### Primary (ATOC Acoustic Engineering Test)

- **Authors**: Bruce D. Cornuelle, Peter F. Worcester, Brian D.
  Dushaw, Matthew A. Dzieciuch, Bruce M. Howe, James A. Mercer,
  Robert C. Spindel, and the ATOC Group
- **Title**: "Comparisons of measured and predicted acoustic
  fluctuations for a 3250-km propagation experiment in the eastern
  North Pacific Ocean"
- **Venue**: *JASA* 105(6):3202-3218 (June 1999)
- **DOI**: https://doi.org/10.1121/1.424646
- **Publisher**:
  https://pubs.aip.org/asa/jasa/article-abstract/105/6/3202/553422/

### Companion (ATOC basin-scale test, same issue)

- **Worcester, P.F., Cornuelle, B.D., Dzieciuch, M.A., et al.**
  (1999). "A test of basin-scale acoustic thermometry using a
  large-aperture vertical array at 3250-km range in the eastern
  North Pacific Ocean." *JASA* 105(6):3185-3201. DOI:
  10.1121/1.424649.

### Modern ML-based SSP inversion

- **Choo, Y., Seong, W.** (2018). "Compressive sound speed profile
  inversion using beamforming results." *Remote Sensing* 10(5):704.
  DOI: 10.3390/rs10050704.
- **Bianco, M., Gerstoft, P., Traer, J., et al.** (2019). "Machine
  learning in acoustics: theory and applications." *JASA* 146(5):
  3590. DOI: 10.1121/1.5133944. Section 4.3 surveys SSP inversion.
- **Xu, L., Yang, K., Yang, Q., Guo, Y.** (2025). "Passive inversion
  of sound speed profile based on normal mode extraction of
  monochromatic signals." *Intelligent Marine Technology and
  Systems* 3:7. DOI: 10.1007/s44295-025-00083-2.
- **Sun, Y. et al.** (2024). "Automatically Differentiable
  Higher-Order Parabolic Equation for Real-Time Underwater Sound
  Speed Profile Sensing." *J. Marine Sci. Eng.* 12(11):1925. DOI:
  10.3390/jmse12111925.

## Status

**Verified.** Cornuelle et al. 1999 is indexed at AIP Publishing
under DOI 10.1121/1.424646 (JASA 105, issue 6, pp. 3202-3218).
Worcester et al. 1999 companion under DOI 10.1121/1.424649 (pp.
3185-3201). Bianco et al. 2019 is verified at DOI 10.1121/1.5133944.
Xu et al. 2025 is verified via Springer DOI 10.1007/s44295-025-00083-2.
Sun et al. 2024 is verified at MDPI DOI 10.3390/jmse12111925. These
five references together span 25+ years of the same core idea:
**invert measured travel-time or arrival-angle residuals for the
underlying sound-speed profile**.

## Historical context

Cornuelle-Worcester-Dushaw 1999 was the Acoustic Engineering Test
(AET) of the basin-scale ATOC program — a 3250 km acoustic range
from Kauai to an NE-Pacific vertical line array, broadcasting at
75 Hz with M-sequence encoding. The paper demonstrated that measured
arrival patterns matched Navy-climatology-predicted patterns to
within ~5 ms RMS after accounting for mesoscale and tidal
variability, and critically that the **residuals were informative
about the real SSP along the path**. This validated OAT (Munk-Wunsch
1979) at basin scale and established travel-time-to-SSP inversion as
an operational technique.

Twenty-five years later, a cluster of ML papers (Choo 2018, Bianco
2019, Xu 2025, Sun 2024) reformulates the same inverse as a
**learnable operator** — a neural network (dictionary learning,
deep PINN, differentiable PE, tensor decomposition) that maps
observations (arrivals, beamformer outputs, mode extractions) to
an SSP estimate in ms rather than the minutes required by classical
matched-field inversion.

For a sonobuoy field with dense inter-buoy ranging, the relevant
scale is ~100 m - 10 km between pairs, not 3250 km, but the
mathematics is identical up to a range-dependent weighting.

## Core content

### The travel-time → SSP map

From any pair of nodes `(i, j)` at positions `p_i, p_j`, the
measured travel time is

    τ_ij = ∫_{Γ_ij} ds / c(s)

where `Γ_ij` is the acoustic ray (or mode, or path) connecting
them. Linearizing around a reference SSP `c₀(z)` gives

    δτ_ij = -∫_{Γ_ij^{ref}} (δc / c₀²) · ds

which is a **line integral of SSP perturbation** along the reference
path — the foundational OAT forward model (Munk-Wunsch 1979,
covered in `munk-worcester-tomography.md`). For `K` pairs × `M`
resolved arrivals per pair, stacking gives

    δτ ∈ R^{KM}  =  G · δc_vec ∈ R^{L}  +  ε

with `L` the discretized-SSP dimension.

### The EOF reduction

SSP variability is low-rank; leading ~3-5 EOFs of a climatological
covariance (Levitus, WOA18, Argo) capture >90% of seasonal variance.
Project `δc_vec` onto EOF basis:

    δc_vec = Φ · α          Φ = [φ_1 | φ_2 | ... | φ_K],  α ∈ R^K

Substituting:

    δτ ≈ (G Φ) · α + ε

which is a small `KM × K` linear system — solvable in closed form
by Gauss-Markov with EOF covariance `C_α = diag(λ_1, ..., λ_K)`
(EOF eigenvalues).

### Cornuelle-Worcester-Dushaw 1999 operational numbers

- **Range**: 3252 km
- **Source carrier**: 75 Hz, 30-s M-sequence pulse
- **Arrival pattern**: 6-12 resolved rays per 24-hour transmission
- **Travel-time precision**: ~1 ms RMS over 6-day observation
- **SSP posterior uncertainty**: ~0.5 m/s at mesoscale wavelengths,
  ~0.1 m/s at 500 m² resolution
- **Temperature-equivalent**: ~0.05 °C basin-averaged

### Bianco et al. 2019 JASA review — ML for SSP inversion

Bianco et al. reviews the 2013-2019 ML work in ocean acoustics. Key
takeaways for SSP inversion:
- **Dictionary learning** (Bianco 2017) encodes SSP in ~64-atom
  dictionaries; sparse coding recovers `δc(z)` from arrival times
  at 10× the speed of classical MFI.
- **Deep learning** (Choi 2018, Huang 2019) maps received pressure
  field directly to SSP without an explicit forward model — risky
  without physics regularization.
- **PINN** (Du 2023, Xu 2025) trains a Helmholtz-solving NN that
  takes SSP as a conditioning variable; inverting is gradient
  descent through the PINN.

### Xu 2025 passive SSP inversion

Passive (source-of-opportunity) mode extraction from monochromatic
signals → modal travel-time differences → SSP inversion via a
neural normal-mode extractor. Reports:
- Input: continuous shipping noise at ~150-300 Hz
- Output: SSP at 2 m depth resolution
- Accuracy: ~1 m/s RMS against in-situ CTD
- Time-to-solution: ~100 ms on modern GPU

### Sun 2024 differentiable-PE SSP sensing

An **autodiff Padé-approximant parabolic-equation solver** —
effectively BELLHOP's cousin RAM made gradient-traceable. Gradient
of observation residual w.r.t. SSP flows back through the full
wave-propagation kernel. Reports:
- Convergence: ~50 iterations to sub-1 m/s SSP from synthetic data
- Speedup vs non-differentiable PE + finite-difference gradient:
  ~100× for same accuracy

## Portable details — the SSP inversion for the sonobuoy field

### The reduced forward model

Buoys at known surface positions (from GNSS or LBL self-
localization). Each pair `(i, j)` gives multiple arrivals: direct
ray `τ_dir`, first surface bounce `τ_sb`, first bottom bounce
`τ_bb`. For each pair compute

    Δτ_ij^{sb-dir} = τ_sb - τ_dir        (sensitive to upper SSP)
    Δτ_ij^{bb-dir} = τ_bb - τ_dir        (sensitive to deep SSP)

These travel-time differences are **insensitive to clock bias**
(both arrivals share the clock) and to buoy-position noise (both
arrivals share the geometry). They're pure SSP-inversion data.

### Rust skeleton

```rust
/// Multi-path arrival measurement between two nodes.
#[derive(Debug, Clone)]
pub struct MultipathArrivals {
    pub pair: (u16, u16),
    pub tau_direct_s: f64,
    pub tau_surface_bounce_s: f64,
    pub tau_bottom_bounce_s: Option<f64>,
    pub sigma_s: f32,
}

/// Reduced-basis SSP, 3-5 EOFs.
#[derive(Debug, Clone)]
pub struct SspReducedBasis {
    pub depths_m: Vec<f64>,
    pub c_ref: Vec<f64>,
    pub phi: Vec<Vec<f64>>,           // L×K EOF matrix
    pub alpha: Vec<f64>,              // K-dim coefficients
    pub cov: Vec<Vec<f64>>,           // K×K
}

/// Ray-trace Jacobian: ∂τ/∂α_k per multipath arrival.
pub fn ssp_jacobian(
    pair: (u16, u16),
    positions: &[[f64; 3]],
    ssp: &SspReducedBasis,
) -> Vec<Vec<f64>> { /* ... */ todo!() }

/// Gauss-Markov SSP update from a batch of multipath arrivals.
pub fn ssp_update(
    ssp: &mut SspReducedBasis,
    arrivals: &[MultipathArrivals],
    positions: &[[f64; 3]],
) { /* ... */ todo!() }
```

### The tomography-on-a-sonobuoy cadence

- Ranging pings happen at TDMA cadence, 0.1-1 Hz per pair.
- Multipath arrivals (direct + surface-bounce ± bottom-bounce) per
  ping are extracted by matched-filter with peak detection.
- Stack 60-600 s of measurements → ~100-1000 arrival observations
  across the field.
- Solve reduced-basis inversion every ~60 s → SSP at ~5 EOF
  coefficients per field.
- State dim K=5, observation dim ~1000, Gauss-Markov is closed-form
  and trivial to run on a Tier-3 Cortex-M7 at 50 mW.

## Integration with the sonobuoy stack

This paper cluster closes the SSP loop for the K-STEMIT physics-prior
branch. The existing physics-prior branch (ADR-059, ADR-060) conditions
on a climatological or ship-of-opportunity-CTD SSP; neither is
in-situ nor time-varying at the buoy field's native cadence. The
ranging subsystem's multipath arrivals, processed by the Xu-2025
normal-mode extractor or the Sun-2024 differentiable-PE inverter,
produce **in-situ SSP at 5 EOF coefficients per minute** — the exact
input the Du-2023 Helmholtz-PINN and Zheng-2025 FNO want. Specifically:
`eml_core::operators::helmholtz_residual` gets a time-varying
`c(z, t)` rather than a static environment vector; the FiLM
conditioning (ADR-060) now has live values for thermocline depth
and mixed-layer gradient instead of 8-dim climatology. A new EML-
core operator `eml_core::operators::ssp_inverter` exposes the
Gauss-Markov update as a trainable wrapper; it caches the ray-trace
Jacobian keyed on `(geometry_hash, ssp_ref_hash)` and publishes SSP-
change events to the `Impulse` queue whenever the EOF coefficients
drift beyond threshold. Downstream, Bucker-1976 matched-field
processing (ADR-067 baseline) now operates on ground-truth geometry
+ ground-truth SSP, closing the last remaining modelling gap in the
classical baselines.

## Strengths

1. **Zero-marginal-cost SSP estimate** — the same pings that localize
   the buoys and detect targets also measure SSP. No extra hardware
   or deployment cost.
2. **Robust to clock and position bias** — travel-time-*differences*
   (surface-bounce minus direct, etc.) are immune to both.
3. **Low-dim reduced basis** — EOF expansion keeps the inverse small
   (`K=3-5`), runnable on-buoy in real time.
4. **Calibrated uncertainty** — Gauss-Markov posterior covariance
   propagates into downstream FiLM-conditioning uncertainty, closing
   the loop on physics-prior confidence.
5. **Long operational pedigree** — ATOC (1996-2006), NPAL (2004),
   PhilSea09 (2009). Basin-scale deployments worked. Drifting-buoy
   scale should work a-fortiori.

## Limitations

1. **Requires resolved multipath** — direct + surface-bounce minimum.
   In very shallow water or at very short inter-buoy ranges the
   arrivals merge and can't be separated by matched filter alone.
2. **Linearization assumes small SSP perturbation** — strong
   thermocline events (e.g., internal-wave passage) can exceed the
   5% linearization threshold; iterative re-linearization required.
3. **EOF basis is climatology-derived** — the system won't detect
   SSP structures not represented in WOA/Levitus/Argo.
4. **Ray identification can fail** — matched-filter peaks must be
   correctly labeled as "direct" vs "bounce". Low-SNR regimes flip
   identification; need arrival-angle gating or statistical
   association.
5. **SSP is path-averaged, not 3-D** — for sparse buoy fields the
   inversion recovers an SSP that best-explains the observed arrivals,
   not a true 3-D c(x, y, z) field. Dense fields (ADR-057 Grinstein
   Relation-Network regime, ≥8 buoys) recover 2-D range-azimuth
   structure; ≥16 buoys and careful path coverage are needed for
   3-D. v4 territory.

## Follow-up references

1. **Worcester, P.F., Cornuelle, B.D., Dzieciuch, M.A., et al.**
   (1999). "A test of basin-scale acoustic thermometry using a
   large-aperture vertical array at 3250-km range in the eastern
   North Pacific Ocean." *JASA* 105(6):3185. DOI: 10.1121/1.424649.
   ATOC basin-scale companion to Cornuelle-1999.
2. **Bianco, M., Gerstoft, P., Traer, J., et al.** (2019). "Machine
   learning in acoustics: theory and applications." *JASA*
   146(5):3590. DOI: 10.1121/1.5133944. The definitive review;
   §4.3 covers ML SSP inversion.
3. **Gemba, K.L., Hodgkiss, W.S., Gerstoft, P.** (2017). "Adaptive
   and compressive matched field processing." *JASA* 141(1):92.
   DOI: 10.1121/1.4973528. Modern MFP that benefits directly from
   ranging-derived SSP.
4. **Munk, W., Wunsch, C.** (1979). "Ocean acoustic tomography: a
   scheme for large-scale monitoring." *Deep-Sea Res.* 26(2):123.
   DOI: 10.1016/0198-0149(79)90073-6. The foundational reference
   (covered in `munk-worcester-tomography.md`).
5. **Huang, C.-F., Gerstoft, P., Hodgkiss, W.S.** (2008). "Effect
   of ocean sound speed uncertainty on matched-field geoacoustic
   inversion." *JASA* 123(6):EL162. DOI: 10.1121/1.2908406. Why
   getting SSP right matters for downstream acoustic processing.
