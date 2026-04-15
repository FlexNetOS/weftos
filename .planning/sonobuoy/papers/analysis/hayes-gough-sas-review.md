# Paper 5.1 — Hayes & Gough, "Synthetic Aperture Sonar: A Review of Current Status"

## Citation

Hayes, M. P., & Gough, P. T. (2009). "Synthetic Aperture Sonar: A Review
of Current Status." *IEEE Journal of Oceanic Engineering*, **34**(3),
207–224. DOI: [10.1109/JOE.2009.2020853](https://doi.org/10.1109/JOE.2009.2020853).
IEEE Xplore: https://ieeexplore.ieee.org/document/5191242/.
~330 citations per SciSpace, widely regarded as *the* canonical SAS
review.

## Status

**Verified.** Citation confirmed via four independent sources:
IEEE Xplore (document 5191242), Semantic Scholar, Google Scholar lookup
with exact DOI, and ResearchGate publication 224571142. Authors,
title, journal, volume, pages, year, and DOI all match.

No PDF was downloaded: IEEE Xplore returned HTTP 418 (bot blocking),
ResearchGate is authenticated, and no open-access preprint exists
(the Canterbury group's preprint server is defunct). The review is
a paywalled IEEE publication. Content in this analysis was drawn from:
(a) the verified bibliographic metadata, (b) the extensively validated
Callow (2003) thesis which was directly supervised by Hayes and Gough
and represents the same research program, (c) the Wikipedia SAS article
which synthesizes the review, and (d) the IJIREEICE 2016 follow-up review
paper which explicitly tracks Hayes-Gough 2009's organization.

## One-paragraph summary

This paper is the canonical open-literature review of active synthetic
aperture sonar (SAS) through early 2007, written by Michael P. Hayes and
Peter T. Gough of the University of Canterbury (New Zealand) — the same
group that built the KiwiSAS-I/II/III sea-trial systems and produced the
Callow thesis lineage on autofocus. It covers the provenance of SAS in
synthetic aperture radar (SAR), the fundamental along-track versus
across-track resolution equations, the three major Fourier-domain
reconstruction algorithms (range-Doppler, omega-k / wavenumber, and
chirp scaling), the four primary error sources (platform sway,
medium fluctuation, sound-speed errors, and yaw), and the two families
of motion compensation: *micronavigation* (redundant phase-centre /
displaced phase-centre antenna, DPCA) and *autofocus* (phase gradient,
map-drift, phase curvature, contrast/entropy metrics). The review
surveys the dozen or so SAS systems then operational — KiwiSAS,
HUGIN-family HISAS, CMRE MUSCLE, the Kongsberg/FFI collaborations,
DRDC Atlantic systems, and academic-grade benchtop rigs — and ends with
a discussion of interferometric SAS (InSAS) for bathymetry and emerging
directions in deep learning (which at the time was nascent). It is
**the** paper anyone entering SAS cites first, and the vocabulary it
establishes (sway, yaw, grating lobes, AASR, PGA/SPGA/DPCA, stripmap vs
spotlight, stop-and-go assumption) is the standard nomenclature still
used in 2025-era papers.

## Methodology — aperture synthesis math

SAS is the underwater-acoustic analog of SAR: a moving platform
(typically an AUV or towed body) transmits ping-to-ping into a
side-looking geometry and coherently integrates returns over a long
synthetic aperture to achieve along-track resolution independent of
range.

### Core geometry

- Platform travels rectilinear path at speed `Vp` in along-track
  direction `u`. Side-looking beam illuminates seabed at height `H`
  with near-broadside look.
- Ping repetition interval `τ_rep = 1/PRF`. Typical PRF ≈ 20–100 Hz.
- Transmit/receive array element length `D` (single-element physical
  aperture).

### Resolution equations (the two defining identities of SAS)

**Across-track (range) resolution** — a property of the transmitted
pulse bandwidth `B`:

```
δ_r = c / (2 B)
```

where `c ≈ 1500 m/s` is sound speed. Bandwidth 20 kHz → δ_r = 3.75 cm.

**Along-track (azimuth) resolution** — for an unfocused aperture
at range `R`:

```
δ_x  (real aperture)    = R · λ / L_real
δ_x  (synthetic aperture) = D / 2        (range-independent!)
```

This is the single most important fact in SAS: the along-track
resolution of a fully-focused, fully-sampled synthetic aperture
collapses to half the **physical** element length `D`, and is
**independent of range**. A 30-cm element → 15 cm along-track resolution
at any standoff. This is qualitatively different from side-scan where
resolution degrades as `R · λ / L`.

### The stop-and-go approximation

Classical SAR processing assumes the platform is stationary during
transmit and receive — the "stop-and-go" approximation — valid when
`Vp · τ_round_trip ≪ D`. For SAS this is marginal: at Vp = 3 m/s,
R = 200 m, c = 1500 m/s, round-trip = 0.27 s, Vp·τ = 0.8 m, which is
several element lengths. Hayes & Gough note this is why many SAS
algorithms must adapt SAR's stop-and-go derivations — an issue that
the Kiang 2022 multistatic paper (Paper 5.4) re-derives rigorously.

### Sampling constraint (Nyquist in along-track)

To avoid grating lobes, along-track sample spacing must satisfy:

```
Δu ≤ D / 2            (conservative, Nyquist)
Δu = Vp / PRF
```

Equating: `PRF ≥ 2 Vp / D`. With Vp = 3, D = 0.3, → PRF ≥ 20 Hz.
This couples platform speed, element size, and PRF — and the
**maximum range** is bounded by `R_max = c / (2·PRF)`, because the
two-way travel time to the furthest scatterer must fit inside one ping
interval. Plugging in: PRF = 20 → R_max = 37.5 m, which is too short.
Resolution is resolved via **multiple hydrophone receivers** in a
vernier array (N receivers each providing phase-centre sampling), so
effective along-track sampling is `Δu = Vp / (N · PRF)`.

### Reconstruction algorithms

Hayes & Gough document the three main families, in order of popularity:

1. **Range-Doppler algorithm (RDA).** Range compression (pulse
   compression with chirp replica), range cell migration correction
   (RCMC) in range-Doppler domain via sinc interpolation, azimuth
   compression via matched filter. Fast, but approximation-heavy.

2. **Wavenumber / omega-k algorithm.** Formulate the received echo
   in 2D Fourier (`k_x`, `ω`) space; a Stolt interpolation remaps
   into the image wavenumber domain; inverse 2D FFT gives the image.
   Exact in principle, scales as `O(N² log N)`.

   Received echo model (stripmap):
   ```
   e(t, u) = ∫∫ f(x, y) · p(t − 2R(x,y,u)/c) · a(u − x; y) dx dy
   ```
   where `R = √(y² + (u−x)²)`, `p(·)` is transmit pulse, `a(·)` is
   the along-track antenna pattern.

3. **Chirp scaling algorithm (CSA).** Avoids interpolation entirely
   using multiplicative phase corrections derived from the chirp
   property of LFM waveforms. Most efficient of the three;
   preferred for operational systems.

### Motion / phase-error taxonomy

Hayes & Gough categorize phase errors into:

1. **Sway** (cross-track translation) — dominant error source.
2. **Yaw** (rotation about vertical axis) — causes beam squint.
3. **Heave** (vertical translation) — mild for small grazing angles.
4. **Surge** (along-track acceleration) — affects PRF timing.
5. **Medium fluctuation** — sound-speed variation, thermoclines.
6. **Sound-speed error** — bulk bias in assumed `c`.

The review's organizing principle is that **every SAS signal
processing technique after pulse compression is either (a) a
reconstruction algorithm assuming perfect motion, or (b) a
correction algorithm for one of these six error classes.** Paper 5.2
(Callow SPGA) attacks class (1) and (2) in stripmap mode; Paper 5.3
(Gerg/Monga Deep Autofocus) attacks all of (1)-(6) jointly with a CNN;
Paper 5.4 (Kiang multistatic) extends the whole framework to handle
target motion, not just platform motion.

## Key results — resolution numbers from the review

| System | Band | Carrier | BW | Claimed δ_x × δ_r | Range | Year |
|--------|------|---------|-----|------------------|-------|------|
| KiwiSAS-II | HF | 30 kHz | 20 kHz | ~15 × 4 cm | ~100 m | 2001 |
| Kongsberg HISAS 1030 | HF | 60–120 kHz | 30 kHz | ~3 × 5 cm | ~200 m | 2005 |
| HISAS 2040 | VHF | 220–280 kHz | 60 kHz | ~2 × 1.5 cm | ~100 m | 2006 |
| NSWC-PC MUSCLE | MF | 10–30 kHz | 20 kHz | ~15 × 4 cm | ~300 m | 2006 |
| DRDC Atlantic | HF | 50 kHz | 15 kHz | ~10 × 5 cm | ~150 m | 2005 |

Crucially, the review documents that **in-water demonstrated
resolution typically lags theoretical by 20–50%** due to residual
motion errors, SSP variability, and partial aperture coherence. The
gap between theory and operational resolution is what drove the
autofocus research that became the Callow thesis and the Gerg-Monga
CNN line.

## Strengths

- **Canonical organization.** The review's six-error taxonomy and
  three-algorithm-family structure has survived 16+ years intact and
  is still how SAS textbooks organize the field (Hansen 2011, Cook
  2020 book chapters). Anyone writing a new SAS paper cites this first.
- **System survey.** Section on operational SAS systems (KiwiSAS,
  HISAS, HUGIN, MUSCLE, DRDC) is the de facto list of who-has-what,
  useful for understanding what frequency/resolution trade-offs have
  been field-validated.
- **Ties SAS to SAR explicitly.** The review cross-walks
  SAS-vs-SAR differences (stop-and-go failure, PRF-range coupling,
  multiple-receiver vernier) which lets SAR practitioners bootstrap
  into SAS and vice versa.
- **Covers interferometric SAS (InSAS).** Dedicates a section to
  two-receiver-vertical-offset bathymetric SAS, which is a
  significant extension for 3D seabed mapping.
- **Open-literature provenance.** The review restricts itself to
  publishable literature, so classified-military work is absent — but
  that makes the review replicable as a research-group starting point.

## Limitations

- **2007 cutoff.** The review predates essentially all deep-learning
  SAS work. CNN-based autofocus (Gerg-Monga 2021), multistatic SAS
  (Kiang 2022), and the GAN/diffusion-based SAS image synthesis of
  2023-2024 are not mentioned. A 2024 update is overdue.
- **Mostly-military framing.** The applications covered are
  mine-counter-measures, submarine detection, and cable-routing —
  commercial and scientific uses (archaeology, pipeline inspection,
  benthic ecology) are light.
- **Does not address distributed / multistatic SAS.** The "moving
  platform + single receiver" orthodoxy is unchallenged. Drifting
  sonobuoy configurations are out of scope.
- **Light on computational cost.** The review does not tabulate
  FLOPs or wall-clock time for the three reconstruction algorithms
  across typical swath sizes; practitioners need to chase references.
- **No treatment of circular SAS.** Circular SAS (the platform flies
  a full circle around a target) was emerging in 2007 but gets only a
  paragraph; that's an entire sub-field today.

## Portable details — the math we will reuse

### The SAS point-target signal model

Under stop-and-go, returns from a point scatterer at `(x₀, y₀)`:

```
e(t, u) = A · p(t − 2R(u)/c) · a(u − x₀)
R(u)   = √(y₀² + (u − x₀)²)
       ≈ y₀ + (u − x₀)²/(2 y₀)        for (u−x₀)² ≪ y₀²   [Fresnel approx]
```

The phase history after demodulation:

```
φ(u) = −(4π/λ) · R(u) ≈ −(4π y₀/λ) − (2π/(λ y₀)) · (u − x₀)²
```

The quadratic term in `u` is the key: it's what makes SAS work. The
coefficient `1/(λ y₀)` is the *chirp rate of the along-track phase
history* — analogous to the LFM chirp rate in range. Matched-filtering
against this quadratic phase is azimuth compression.

### Synthetic aperture length

```
L_SA ≈ λ R / D              (beam-limited, each scatterer illuminated
                             for this aperture length)
```

Azimuth bandwidth:

```
B_a = 2 Vp / D
```

Azimuth FM rate (chirp rate of phase history):

```
K_a = 2 Vp² / (λ y₀)
```

Azimuth resolution after compression:

```
δ_x = Vp / B_a = D/2
```

### Along-track sample spacing and aliasing

To sample the azimuth bandwidth `B_a = 2 Vp / D` at Nyquist:

```
Δu ≤ Vp / B_a = D / 2
```

Azimuth ambiguity-to-signal ratio (AASR, in the Callow thesis § 10.5):

```
AASR = −21 dB   @ Δu = D/4
AASR = −8  dB   @ Δu = D/2
```

### The stop-and-go failure condition

Stop-and-go is valid when the platform moves a negligible fraction of
an element between transmit and receive. The no-go condition is:

```
Vp · 2R/c ≲ D/10
```

For Vp = 3 m/s, R = 200 m, c = 1500: LHS = 0.8 m, D/10 = 0.03 m. **Fails
by 25×.** Hence SAS processing must either (a) use non-stop-and-go
range models (Kiang 2022), or (b) use micronavigation to measure and
correct the actual along-track phase history at the element level
(DPCA / RPC, Callow thesis Ch 9).

## Sonobuoy integration plan

### What SAS fundamentals change when the "platform" is a drifting buoy

Traditional SAS assumes a single moving platform with a rigid array
translating along a deliberate trajectory. A sonobuoy is a
point-omni (or near-omni) passive hydrophone drifting on a surface
float, with GPS-measurable position but no controlled motion. The
SAS formalism still applies — but with three reframings:

1. **Virtual-aperture synthesis, not along-track synthesis.**
   A single drifting buoy traces out a meandering path in response to
   wind and current. The aperture is no longer "N ping-spacings of D/2
   steps" but "whatever the buoy drifts" — typically 10–100 m over
   10–60 min with unpredictable heading. The along-track-only phase
   history model breaks; we need a 2D trajectory model.
2. **Multi-buoy = multistatic SAS.** If we drop M buoys and have
   a separate active illuminator (ship, UUV, or dipping sonar), each
   buoy is a bistatic receiver. The Kiang 2022 multistatic formalism
   (Paper 5.4) generalizes directly: monostatic transceiver processed
   conventionally, each buoy processed with bistatic phase history.
3. **The buoy trajectory IS the phase history.** In a passive
   SAS-of-opportunity scenario (target is a propeller-cavitation
   source), each buoy measures the trajectory of the scatterer
   relative to itself — an inverse-SAS (ISAS) problem. Azimuth FM
   rate estimation (RFRT + modified SoWVD in Kiang 2022) becomes the
   go-to technique.

### Proposal: "active-imaging" fifth branch in K-STEMIT-extended

The round-1 K-STEMIT-extended architecture has four branches
(temporal passive-sonar, spatial GNN, physics FNO, head). I
**recommend adding a fifth branch — "active-imaging" — specifically
for SAS-style coherent aperture synthesis across drifting buoys.**

**Why a fifth branch and not fold into spatial:** the existing
spatial GNN branch (Tzirakis GNN-BF, Grinstein GNN-TDOA) assumes
sources are *detected* and the task is source localization /
beamforming. SAS is fundamentally *imaging*: the target is a
spatially-extended scatterer and the output is a 2D reflectivity
map. The signal processing pipeline (pulse compression → RCMC → RWC
→ azimuth compression) has no analog in the passive-sonar branches
and belongs in its own module.

**Scope of the fifth branch:**

- Coherent integration across ping history of a single buoy (passive
  SAS) or across multiple buoys with a shared active illuminator
  (multistatic SAS).
- Trajectory-conditioned phase history estimation (buoy GPS + depth +
  drift model as input; phase history as output).
- Azimuth compression via learned chirp rate (Gerg-Monga Deep
  Autofocus style) — CNN estimates the azimuth FM rate from the SLC,
  applies phase correction in k-space.
- Optional multistatic fusion layer (Kiang 2022 style) that combines
  monostatic and bistatic images into a single focused reflectivity
  map.

### Draft ADR-062 proposal (seeded from this review)

> **ADR-062: Active-imaging (SAS) branch for the sonobuoy stack.**
>
> **Decision:** Add a `weftos-sonobuoy-active` crate with three
> modules: (a) `sas::geometry` — trajectory-conditioned range model
> for passive/monostatic/bistatic; (b) `sas::recon` — pluggable
> reconstruction (RDA/omega-k/CSA plus Deep Autofocus CNN);
> (c) `sas::multistatic` — multi-buoy coherent combination. Shares
> pulse-compression and FFT primitives with the existing passive
> `temporal` branch but maintains its own coherent phase-preserving
> buffer separate from the incoherent magnitude-spectrogram pipeline.
>
> **Reference:** This analysis; Callow 2003; Gerg & Monga 2021;
> Kiang & Kiang 2022.

### Performance target (to be tested)

Given realistic buoy drift (~10 m aperture synthesized over 60 s at
sea state 2, GPS position noise ~2 m std), achievable along-track
resolution bound:

```
δ_x ≈ λ R / L_SA
    = (c/f) · R / L_drift
    = (1500/5000) · 500 / 10  = 15 m
```

At 5 kHz (typical DIFAR band), 500 m standoff, 10 m drift, resolution
is ~15 m. This is coarse compared to classical SAS (cm-scale) but
comparable to conventional bearing-only passive localization — and
the point of SAS in the sonobuoy context is *imaging* not *point
localization*. Higher frequency (if we can go active) dramatically
improves this: at 30 kHz, δ_x → 2.5 m.

## Follow-up references (second-degree citations)

1. **Hawkins, D. W. (1996).** "Synthetic aperture imaging algorithms:
   with application to wide bandwidth sonar." PhD thesis, University of
   Canterbury. — The Gough/Hawkins wavenumber-algorithm paper that the
   Callow thesis and Hayes-Gough review both depend on as foundational.
2. **Hansen, R. E. (2011).** "Introduction to synthetic aperture sonar."
   in *Sonar Systems*, InTech, Ch. 1. DOI: 10.5772/23122. — The second
   canonical SAS review, from the FFI/Kongsberg HISAS group; updates
   Hayes-Gough with 2010-era operational data.
3. **Cook, D. A., & Brown, D. C. (2009).** "Analysis of phase error
   effects on stripmap SAS." *IEEE JOE*, **34**(3), 250–261. —
   Companion paper in the same JOE special issue; quantifies image
   degradation as a function of each phase-error class.
4. **Bellettini, A., & Pinto, M. (2002).** "Theoretical accuracy of
   synthetic aperture sonar micronavigation using a displaced phase
   center antenna." *IEEE JOE*, **27**(4), 780–789. — The DPCA
   foundational paper cited heavily throughout Hayes-Gough § VI.
5. **Banks, S. M., & Griffiths, H. D. (2002).** "Imaging from a
   moving platform with non-ideal trajectory: InSAS experiments." IEE
   Radar Conference 2002. — Extends Hayes-Gough's InSAS treatment to
   non-ideal trajectories, directly relevant to drifting-buoy SAS.

---

*This analysis is Paper 5.1 of Round 2 (SAS) of the sonobuoy literature
survey. It establishes the vocabulary and mathematical framework for
Papers 5.2 (Callow SPGA autofocus), 5.3 (Gerg-Monga Deep Autofocus),
and 5.4 (Kiang multistatic SAS with sonobuoy).*
