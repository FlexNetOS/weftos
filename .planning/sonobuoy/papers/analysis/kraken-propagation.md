# Porter — The KRAKEN Normal Mode Program

## Citation

- **Author**: Michael B. Porter
- **Title**: "The KRAKEN Normal Mode Program"
- **Venue**: SACLANTCEN Memorandum SM-245 (later re-issued as SR-221),
  SACLANT Undersea Research Centre, La Spezia, Italy / Naval Research
  Laboratory, Washington DC, 1991–1992
- **DTIC accession**: AD-A252 409 (Defense Technical Information Center)
- **DTIC PDF**: https://apps.dtic.mil/sti/tr/pdf/ADA252409.pdf
- **DTIC archive (Internet Archive mirror)**: https://archive.org/details/DTIC_ADA252409
- **CMRE Open Library**: https://openlibrary.cmre.nato.int/handle/20.500.12489/201
- **OALIB current distribution (living draft)**: http://oalib.hlsresearch.com/Modes/kraken.pdf
- **Downloaded**: `.planning/sonobuoy/papers/pdfs/kraken-propagation.pdf`
  (207-page living OALIB version)
- **Software home**: Ocean Acoustics Library (OALIB),
  https://oalib-acoustics.org/

## Status

**Verified.** SACLANTCEN memorandum SM-245 is indexed in DTIC with
accession AD-A252 409; the report is also mirrored on the CMRE
(former SACLANTCEN) open library, on OALIB, and on the Internet
Archive. The Acoustics Toolbox references page at OALIB confirms the
1991/1992 SACLANTCEN report as the canonical KRAKEN citation. The
*Computational Ocean Acoustics* textbook (Jensen et al. 2011) cites
this report as the reference implementation of modal propagation.

## Historical context

KRAKEN was Porter's answer to the numerical pathologies of earlier
normal-mode programs (PROLOS, NORMOD, PE-based hybrids). By the late
1970s the Navy had several normal-mode codes, each failing on a
different class of sound-speed profiles: some missed modes, some
produced spurious modes, some numerically diverged for soft-bottom
profiles. Porter, then at SACLANT Undersea Research Centre (La Spezia)
and concurrently NRL Washington, built KRAKEN to (i) always find a
complete set of trapped and leaky modes, (ii) handle both fluid and
elastic bottoms, and (iii) run cheaply enough that ASW operators could
use it in fleet tools.

The design is now 35 years old, and it is still the Navy's reference
modal solver. KRAKEN, KRAKENC (the complex-eigenvalue / leaky-mode
extension), and SCOOTER (the companion wavenumber-integration code)
ship as the modal half of the OALIB Acoustics Toolbox; the ray half is
Bellhop (see companion `bellhop-ray-tracing.md`). Every post-2000
textbook (Jensen-Kuperman-Porter-Schmidt *Computational Ocean
Acoustics* 2011, Etter 2018) uses KRAKEN as the normal-mode reference
implementation, and every ML paper that trains on synthetic TL maps
generates the training set with KRAKEN, RAM, or Bellhop.

## Core content

### The modal model

For a range-independent waveguide with depth-dependent sound speed
`c(z)` and density `ρ(z)`, the Helmholtz equation

    ∇²ψ + (ω² / c²(z)) ψ = -s(ω)·δ(r)·δ(z - z_s) / r     (cylindrical)

separates under `ψ(r, z) = Σ_m Φ_m(z)·H₀^(1)(k_{r,m}·r)` into a
Sturm–Liouville eigenproblem in depth:

    d/dz [ ρ⁻¹(z) · dΦ_m/dz ] + [ ω²/(ρ(z)·c²(z)) - k_{r,m}² / ρ(z) ] · Φ_m(z) = 0

subject to surface BC `Φ_m(0) = 0` (pressure release) and bottom BC
(rigid, soft, fluid halfspace, or elastic-elastic) that fixes the
modal problem. The eigenvalue `k_{r,m}` is the horizontal wavenumber
of mode `m`; `Φ_m(z)` is its depth shape. The far-field pressure at
`(r, z_r)` from a source at `(0, z_s)` is

    p(r, z_r, ω) ≈ ( i·e^{-iπ/4} / (ρ(z_s)·sqrt(8π·r)) )
                   · Σ_m Φ_m(z_s)·Φ_m(z_r)·e^{i·k_{r,m}·r} / sqrt(k_{r,m})

and the transmission loss is `TL(r, z_r) = -20·log₁₀(|p(r, z_r)|)`
relative to a 1-m reference.

### The KRAKEN algorithm

Porter's innovations, relative to pre-1991 modal codes (Porter 1991,
§2):

1. **Galerkin finite-element discretization** of the depth operator on
   a non-uniform mesh. Handles arbitrary `c(z)` and density jumps
   without operator splitting.
2. **Sturm-sequence bracketing + inverse iteration** for eigenvalues.
   Guarantees that all modes in a specified wavenumber interval are
   found; no silent mode-dropping (the core failure of PROLOS).
3. **Richardson extrapolation** across two mesh refinements. Gives
   4th-order convergence of `k_{r,m}` for 2nd-order elements; ~1%
   eigenvalue accuracy at mesh spacings of `λ/10`.
4. **KRAKENC extension** — complex rotation of the vertical axis
   captures leaky modes with `Im(k_{r,m}) < 0`. Needed whenever the
   bottom radiates energy away (soft-bottom litoral, sediment-coupling
   cases).
5. **Range-dependence via adiabatic or one-way coupled modes**. For
   range-varying environments, eigenfunctions are recomputed at each
   range step and mode amplitudes coupled via range-derivatives. The
   range-dependent companion code C-SNAP (Ferla, Porter, Jensen 1993,
   SACLANTCEN SM-274) builds on KRAKEN.

### The KRAKEN input / output contract

- Input: an `*.env` file (depth layers, `c(z)`, `ρ(z)`, bottom type,
  frequency, source and receiver depths).
- Output: `MODFIL` binary with eigenvalues and mode shapes; `FIELD`
  postprocessor produces TL(r, z) grids.
- Typical runtime: milliseconds to seconds for a 1 kHz, 5 km range,
  500 m water column problem — orders of magnitude faster than the
  parabolic-equation RAM solver for the same scene when KRAKEN is
  applicable (range-independent or slowly range-varying).

### When KRAKEN applies

KRAKEN is the correct solver when:
- Frequency is low-to-mid (modal count < ~100; above this, ray methods
  are cheaper and just as accurate),
- The environment is range-independent or adiabatically range-varying,
- The bottom is a stack of fluid and elastic layers with known `ρ, c_p, c_s`,
- Sound-speed gradients are not so extreme that mode density explodes.

For the thermocline-bending cases that sonobuoys care about, KRAKEN is
still usable when run with sufficient vertical resolution (mesh spacing
`< λ/10`); for sharp fronts or eddies, one must switch to PE (RAM,
Collins 1993) or ray (Bellhop) methods. The "which solver" decision is
itself a subject of Porter's writing (Jensen, Kuperman, Porter, Schmidt
2011, Ch. 5).

## Modern relevance

ML-PDE papers in the sonobuoy space (round 1 §2.3) compare against
KRAKEN-generated ground truth:

- **Du-2023 Helmholtz-PINN** (Frontiers Mar. Sci.) trains a PINN to
  match KRAKEN TL maps on a held-out SSP library. The PINN reaches
  R²=0.99 in 2D — that R² is against KRAKEN, not against measurement.
- **Zheng-2025 FNO surrogate** (Frontiers) uses RAM as ground truth for
  fronts and KRAKEN for range-independent pretrain. The 28.4% speedup
  figure is FNO vs RAM, but the KRAKEN-generated pretrain data is a
  significant fraction of the training set.
- **Matched-field processing (MFP) with ML** (Neilsen, Niu, Gemba, etc.)
  uses KRAKEN to generate the "replica field" library that MFP
  correlates measured pressures against.

Outside ML, KRAKEN is the reference solver in NATO/Navy fleet tools:
ASTRAL, OAML/ESPS, and the Hybrid Prediction System all have a KRAKEN
(or KRAKENC) kernel.

## Sonobuoy relevance

KRAKEN is the classical propagation solver the sonobuoy physics-prior
branch's ML surrogates (Du-2023 PINN, Zheng-2025 FNO) *replace*. Several
consequences for the project:

1. **Ground-truth generator.** For any closed-form benchmark we run, the
   TL reference must be a KRAKEN or RAM output, not the ML surrogate's
   self-assessment. This is the "what must the ML beat?" anchor for
   ADR-059.
2. **Fallback solver.** The ML surrogate is fast but brittle
   (`Helmholtz-PINN 3D → R²=0.48`). When the surrogate's confidence
   drops below threshold, the edge node should fall back to a cached
   KRAKEN-precomputed TL library or, if connected, a live KRAKEN
   query. This is the classical-ML hybrid pattern.
3. **Training data pipeline.** To train or fine-tune the physics branch
   for a new theater (Arctic, Mediterranean, shallow shelf), we need a
   KRAKEN-based sweep over `(SSP, bathymetry, sediment)` producing
   ~10⁴–10⁵ TL maps. This is a standard Navy workflow; we inherit it.
4. **Trust story.** When we tell an ASW operator "our ML TL map is
   within 0.1 dB of the Navy reference", the Navy reference is
   KRAKEN/RAM/Bellhop. We must match by default.

Unlike FNO or PINN, KRAKEN is not trainable; it is a deterministic
numerical method. In the ECC/EML architecture this means KRAKEN lives
as an **external solver** called from `eml_core::operators::` rather
than a parameterized learned operator. One Rust wrapper around the
OALIB KRAKEN binary is enough for v2.

## Portable details

KRAKEN itself is 15k lines of Fortran 77/90 and not reimplementable in
a reasonable budget. The portable contract is the **environment file
format** (`*.env`) and the **output binary** (`MODFIL`). A Rust sonobuoy
crate should ship:

```rust
/// Minimal KRAKEN env-file writer. Matches the format documented in
/// Porter 1991 §3 and reproduced in Jensen-Kuperman-Porter-Schmidt
/// 2011 Appendix B.
pub struct KrakenEnv {
    pub title: String,
    pub frequency_hz: f64,
    pub num_media: u32,           // water + sediment layers
    pub top_option: String,       // e.g. "NVW" = vacuum above, dB/λ atten
    pub media: Vec<Medium>,
    pub bottom_option: String,    // e.g. "A" = acoustic halfspace
    pub bottom_halfspace: Halfspace,
    pub cmin_mps: f64,            // phase-velocity search interval lower
    pub cmax_mps: f64,            //                               upper
    pub max_range_km: f64,
    pub num_sd: u32,              // source depths
    pub source_depths_m: Vec<f64>,
    pub num_rd: u32,              // receiver depths
    pub receiver_depths_m: Vec<f64>,
}

pub struct Medium {
    pub mesh_points: u32,
    pub sigma_m: f64,             // rms interface roughness
    pub depth_m: f64,
    pub profile: Vec<(f64, f64, f64, f64, f64, f64)>,
    //              (z, cp, cs, rho, alpha_p, alpha_s)
}

pub struct Halfspace {
    pub cp_mps: f64,
    pub cs_mps: f64,
    pub rho_gcc: f64,
    pub alpha_p_db_per_lambda: f64,
    pub alpha_s_db_per_lambda: f64,
}

impl KrakenEnv {
    /// Serialize to the KRAKEN `*.env` text format.
    pub fn write(&self, path: &std::path::Path) -> std::io::Result<()> { /* ... */ }
}

/// Shell-out wrapper around an installed KRAKEN binary.
pub fn run_kraken(env: &KrakenEnv, workdir: &std::path::Path) -> Result<ModeOutput, KrakenError> {
    // 1. Write env to workdir/run.env
    // 2. Spawn `kraken.exe run` (or `kraken` on Unix)
    // 3. Parse MODFIL binary → eigenvalues, mode shapes
    // 4. Return ModeOutput { k_r: Vec<Complex64>, phi: Array2<f64>, z: Vec<f64> }
    todo!()
}

pub struct ModeOutput {
    pub k_r: Vec<Complex64>,        // horizontal wavenumbers (m⁻¹)
    pub phi: ndarray::Array2<f64>,  // mode shapes, rows = modes, cols = depths
    pub depths_m: Vec<f64>,
}

impl ModeOutput {
    /// Evaluate pressure p(r, z_r) by modal summation given source depth.
    pub fn pressure(&self, r_m: f64, z_r_m: f64, z_s_m: f64) -> num::Complex<f64> { /* ... */ }

    /// Transmission loss in dB re 1 m.
    pub fn tl_db(&self, r_m: f64, z_r_m: f64, z_s_m: f64) -> f64 {
        -20.0 * self.pressure(r_m, z_r_m, z_s_m).norm().log10()
    }
}
```

Unit-test anchors (Jensen-Kuperman-Porter-Schmidt 2011 Fig. 5.11 —
Pekeris waveguide):
- 100 m water, c=1500 m/s, ρ=1.0 g/cc; bottom c=1800 m/s, ρ=1.8 g/cc
- 20 Hz source at z_s = 36 m, receiver at z_r = 46 m
- Expect 2 trapped modes, `k_{r,1}` ≈ 0.0836 m⁻¹, `k_{r,2}` ≈ 0.0717 m⁻¹
- TL at 10 km ≈ 60 dB

## Follow-up references

1. **Porter, M.B., Reiss, E.L.** (1984). "A numerical method for
   ocean-acoustic normal modes." *JASA* 76(1):244–252. DOI:
   10.1121/1.391101. The finite-element eigenvalue algorithm that
   KRAKEN productionized.
2. **Jensen, F.B., Kuperman, W.A., Porter, M.B., Schmidt, H.** (2011).
   *Computational Ocean Acoustics*, 2nd ed. Ch. 5 "Normal Modes".
   Springer. DOI: 10.1007/978-1-4419-8678-8. The textbook companion;
   derives KRAKEN from first principles.
3. **Collins, M.D.** (1993). "A split-step Padé solution for the
   parabolic equation method." *JASA* 93(4):1736–1742. DOI:
   10.1121/1.406739. The RAM PE solver; the companion range-dependent
   method to KRAKEN, and the one the FNO papers replace.
4. **Ferla, C.M., Porter, M.B., Jensen, F.B.** (1993). "C-SNAP: Coupled
   SACLANTCEN Normal mode propagation loss model." SACLANTCEN SM-274,
   DTIC AD-A266 189. Range-dependent coupled-mode companion to KRAKEN.
5. **Pekeris, C.L.** (1948). "Theory of propagation of explosive sound
   in shallow water." *Geol. Soc. Am. Mem.* 27. DOI:
   10.1130/MEM27-2-p1. The 1948 analytical foundation of shallow-water
   modal theory; the unit-test case every modal code is benchmarked
   against.
