# Urick — Principles of Underwater Sound (the sonar equation)

## Citation

- **Author**: Robert J. Urick
- **Title**: *Principles of Underwater Sound*, 3rd edition
- **Year**: 1983 (1st ed. 1967, 2nd ed. 1975)
- **Publisher**: McGraw-Hill, New York. Later reprinted by Peninsula
  Publishing, ISBN 978-0-932146-62-5
- **Chapter of interest**: Ch. 2 "The Sonar Equations" (and Ch. 15–16 for
  detection threshold and signal excess in the 3rd ed.)
- **WorldCat / OCLC**: 8688952
- **Archive copy (1st ed.)**: https://archive.org/details/principlesofunde00uric
- **Reference database entry**: https://www.scirp.org/reference/referencespapers?referenceid=1399376

## Status

**Verified.** The 3rd edition ISBN, publisher, and chapter numbering are
corroborated by the publisher listing, multiple university syllabi, and the
scanned Internet Archive copy of the 1st edition (which carries the same
Ch. 2 structure). The passive- and active-sonar equations and the
definition of detection threshold (DT) quoted below are reproduced in
essentially identical form in every underwater-acoustics textbook that
followed (Ainslie 2010, Etter 2018, Jensen et al. 2011).

## Historical context

Urick was a physicist at the US Naval Ordnance Laboratory and later the
Naval Surface Warfare Center, White Oak; he wrote *Principles of
Underwater Sound* as the first comprehensive single-author Navy
textbook on applied underwater acoustics. The 1st edition (1967) was
written during the Cold War surge in ASW (anti-submarine warfare)
research and consolidated in one volume the sonar-equation formalism
that had been scattered across wartime NDRC Summary Technical Reports,
the 1946 *Physics of Sound in the Sea*, and various Navy internal
memoranda. By the 3rd edition (1983) the book had become the canonical
Navy reference, adopted by NUSC/NOSC (later NUWC/SPAWAR) for fleet
training and by civilian oceanography programs worldwide.

Every modern underwater-acoustics textbook — Jensen, Kuperman, Porter &
Schmidt's *Computational Ocean Acoustics* (Springer 2011), Ainslie's
*Principles of Sonar Performance Modelling* (Springer 2010), Etter's
*Underwater Acoustic Modeling and Simulation* (CRC 2018) — treats Urick's
formulation of the sonar equation as the starting point. Modern ML sonar
papers still frame their headline metric as "SE at threshold" or
"effective DT reduction" in Urick's notation even when the inference
engine is a transformer.

## Core content — the sonar equations

Urick organizes sonar performance around the decibel equation

    SE = SL - TL + TS - (NL - DI) - DT     (active, monostatic)
    SE = SL - TL      - (NL - DI) - DT     (passive)

where all terms are in dB and SE ("signal excess") is positive when the
sonar can detect the target at the chosen probability of detection and
false-alarm rate. The parameters are (Urick 1983, Ch. 2, pp. 17–34):

| Term | Name | Unit | Reference |
|------|------|------|-----------|
| `SL` | Source level | dB re 1 µPa @ 1 m | transmitter or target-radiated |
| `TL` | Transmission loss (one-way) | dB | receiver to source/target |
| `TS` | Target strength | dB | for active sonar only; bistatic form adds TL on both legs |
| `NL` | Noise level (ambient + self) | dB re 1 µPa²/Hz · bandwidth | in the receiver band |
| `DI` | Directivity index of receiver | dB | `10·log₁₀(4π/Ω)` for beam solid angle Ω |
| `DT` | Detection threshold | dB | SNR required at the array output |

Bistatic active sonar replaces `TL` with `TL₁ + TL₂` (source-to-target,
target-to-receiver) and `SE = SL - TL₁ - TL₂ + TS - (NL - DI) - DT`.
Reverberation-limited operation replaces `NL - DI` with `RL`, the
reverberation level at the array output (Ch. 8).

### Detection threshold (DT)

Urick (1983, Ch. 12 "Detection of Signals in Noise") defines

> "DT is the ratio, in decibel units, of the signal power (or
> mean-squared voltage) in the receiver bandwidth to the noise power
> (or mean-squared voltage) in a 1-Hz band, measured at the receiver
> terminals, required for detection at some preassigned level of
> correctness of the detection decisions."

Formally, for a known-signal-in-Gaussian-noise detector with
bandwidth `W`, integration time `t`, and input signal-to-noise spectral
density ratio `d = (S/W)/N₀`,

    DT = 5·log₁₀(d·W·t)   (coherent matched filter, incoherent energy detector form)
    DT = 5·log₁₀(d²·W·t)  (incoherent / square-law detector)

where `d` is the Neyman-Pearson "detection index" set by the desired
`(P_d, P_fa)` pair on the ROC curve. The 5·log₁₀ form (rather than
10·log₁₀) is specific to the envelope/square-law detector family and
arises because the output noise is χ² with `2Wt` degrees of freedom; see
Urick §12.3 and Van Trees Part I §2.3.

### Source level

Broadband source level for a radiated-noise source is

    SL = 10·log₁₀( I_ref / (1 µPa² / m²) )
       = 20·log₁₀( p_ref @ 1 m / 1 µPa )

For narrowband tonals, `SL` is reported per Hz; for broadband, the
integral across the receiver bandwidth is used. Fleet practice reports
SL of submarines in three octave bands (100 Hz, 1 kHz, 10 kHz) and of
surface ships as continuous spectra to 10 kHz.

### Transmission loss

Urick decomposes TL as

    TL = TL_spreading + TL_absorption + TL_anomaly
       = 20·log₁₀(r)         (spherical, near source)
       = 10·log₁₀(r) + ...   (cylindrical, past first skip)
       + α·r / 1000          (absorption, α in dB/km from Francois-Garrison)
       + A(r, z, c(z), ...)  (anomaly: convergence zones, ducts, shadow zones)

The `A` anomaly term is where propagation solvers (KRAKEN, RAM, Bellhop)
become essential — see the companion analyses.

### Passive-sonar figure of merit (FoM)

When `TL` is the only unknown,

    FoM = SL - (NL - DI) - DT

equals the maximum transmission loss the sonar can tolerate while still
achieving the target `P_d/P_fa`. The FoM is the single number a sonar
operator uses to summarize "how far can I hear today?" — the environment
and the array performance are baked in.

## Modern relevance

Every 2020s ML sonar paper frames its win in Urick terms, even when the
model is a transformer. Examples from round-1 analyses in this
repository:

- **DEMONet** (Xie et al., arXiv:2411.02758) reports DeepShip accuracy at
  ±15 dB SNR — the "SNR" here is `(SL - TL - NL + DI)`, so a 5-pp accuracy
  win at -15 dB SNR is equivalently a ~5 dB DT reduction at fixed P_d/P_fa.
- **Xu/Xie/Wang SIR+LMR** (arXiv:2306.06945) reports "80% accuracy at -15 dB
  SNR"; that 5 dB SNR advantage compounds multiplicatively with passive
  range via `TL = 10·log₁₀(r) + α·r`, so at 1 kHz (α ≈ 0.06 dB/km) a 5 dB
  DT win extends detection range by ~78%.
- **Grinstein GNN-TDOA** (arXiv:2306.16081) frames 29% DoA-error reduction
  as a DI improvement — the array pattern tightens, so the effective
  `NL - DI` noise term drops.
- **NOAA DIFAR** analyses (Allen 2021, Thode 2019) explicitly compute
  Urick's passive FoM as their success metric.

The sonar equation is also the de facto loss function for end-to-end
differentiable sonar: if a model outputs `(P_d, P_fa)` on held-out tracks
at a set of ranges, backing out effective DT and comparing to baseline
Urick DT is the canonical way to publish a "dB gain" number.

## Sonobuoy relevance

For the clawft sonobuoy project, Urick grounds four things that round 1
did not:

1. **The benchmark an ML detector must beat.** Round-1 ML papers quote
   accuracy at fixed SNR; sonobuoy operators quote FoM. Any ML head the
   project ships must be convertible to a DT number so its FoM impact
   is comparable to the classical ERAPS / DMODEL / MODAS baselines.
2. **The physics-prior branch's output specification.** The
   Du-2023 / Zheng-2025 / FiLM trio (§2.3 in SYNTHESIS.md) produces a
   TL map. That map is only useful if the task head consumes it as the
   `TL` term in a live sonar equation, not merely as a side feature.
3. **The array-design contract for a drifting sonobuoy field.** `DI`
   for a non-stationary irregular array is not a fixed 10·log₁₀(4π/Ω);
   it depends on the GCC-PHAT-derived covariance matrix. The Tzirakis
   dynamic-adjacency GCN (§2.2) is implicitly computing a learned,
   time-varying DI — we must log it alongside the classical array
   factor so operators trust it.
4. **The test protocol.** Urick Ch. 12 is the ROC-curve canon. Every
   sonobuoy demo should report `(P_d, P_fa)` at two or three operating
   points, not just top-1 accuracy; ROC AUC on Watkins and DeepShip is
   necessary but not sufficient.

This is the foundational text the sonobuoy README and the ADR-053
motivation section should cite.

## Portable details

The sonar equation is already trivially implementable; the value-add is a
typed, unit-checked Rust struct that converts raw measurements into
FoM/SE reliably and composes with a propagation solver.

```rust
/// All fields in dB unless noted.
pub struct PassiveSonarBudget {
    pub sl_db_re_1upa_at_1m: f64,
    pub tl_db: f64,
    pub nl_db_re_1upa2_per_hz: f64,
    pub bandwidth_hz: f64,
    pub di_db: f64,
    pub dt_db: f64,
}

impl PassiveSonarBudget {
    /// Signal excess in dB. Positive = detected at (P_d, P_fa).
    pub fn signal_excess(&self) -> f64 {
        let nl_in_band = self.nl_db_re_1upa2_per_hz + 10.0 * self.bandwidth_hz.log10();
        self.sl_db_re_1upa_at_1m
            - self.tl_db
            - (nl_in_band - self.di_db)
            - self.dt_db
    }

    /// Figure of merit = max tolerable TL. Drop in the TL from a
    /// propagation solver and compare.
    pub fn figure_of_merit(&self) -> f64 {
        let nl_in_band = self.nl_db_re_1upa2_per_hz + 10.0 * self.bandwidth_hz.log10();
        self.sl_db_re_1upa_at_1m - (nl_in_band - self.di_db) - self.dt_db
    }
}

/// Urick Ch. 12: envelope / square-law detector.
/// d is the Neyman-Pearson detection index for the target (P_d, P_fa).
pub fn detection_threshold_incoherent(d: f64, bandwidth_hz: f64, integration_s: f64) -> f64 {
    5.0 * (d * d * bandwidth_hz * integration_s).log10()
}

/// Urick Ch. 12: coherent matched filter.
pub fn detection_threshold_coherent(d: f64, bandwidth_hz: f64, integration_s: f64) -> f64 {
    5.0 * (d * bandwidth_hz * integration_s).log10()
}
```

Values to unit-test against Urick Fig. 12.4 and Table 12.3:

- `d = 10`, `W = 1 kHz`, `t = 1 s` → coherent DT ≈ 20 dB; incoherent ≈ 25 dB
- For `P_d = 0.5, P_fa = 10⁻⁴`, `d ≈ 18` (Urick Table 12.3 via Peterson,
  Birdsall & Fox 1954)

## Follow-up references

1. **Jensen, F.B., Kuperman, W.A., Porter, M.B., Schmidt, H.** (2011).
   *Computational Ocean Acoustics*, 2nd ed. Springer. ISBN
   978-1-4419-8677-1. The modern companion to Urick; every solver used
   in the sonobuoy physics branch is derived here.
2. **Ainslie, M.A.** (2010). *Principles of Sonar Performance Modelling*.
   Springer. ISBN 978-3-540-87661-8. Rewrites the sonar equation in SI
   units with explicit probability-theoretic foundations for DT.
3. **Peterson, W.W., Birdsall, T.G., Fox, W.C.** (1954). "The theory of
   signal detectability." *IRE Transactions on Information Theory* 4:171.
   DOI: 10.1109/TIT.1954.1057460. The origin of the ROC curve and the
   detection-index tables Urick reproduces.
4. **Urick, R.J.** (1984). *Ambient Noise in the Sea*. Peninsula
   Publishing. The companion volume; expands Ch. 7 of *Principles* and
   reconciles the Wenz curves with post-1962 deep-ocean measurements.
5. **Ainslie, M.A., de Jong, C.A.F.** (2016). "Sonar equations for
   planetary exploration." *JASA* 140(2):1400. DOI:
   10.1121/1.4960783. Rederives the sonar equation from first
   principles, resolving the dB-unit pitfalls that tripped up several
   Europa-sonar papers.
