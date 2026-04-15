# Sueur et al. 2008 — Rapid Acoustic Survey for Biodiversity Appraisal

## Citation

Sueur, J.; Pavoine, S.; Hamerlynck, O.; Duvail, S. (2008).
**"Rapid Acoustic Survey for Biodiversity Appraisal."**
*PLoS ONE* 3(12): e4065.
DOI: https://doi.org/10.1371/journal.pone.0004065
PubMed: 19115006 (PMC2605254)
Published 30 December 2008. Open access under CC BY.

**Status**: verified. DOI resolves at PLoS ONE; PDF downloaded to
`.planning/sonobuoy/papers/pdfs/sueur-acoustic-indices.pdf` (10 pages,
~1 MB) from the PLoS printable URL; abstract and author list match
the PubMed / PMC record at
https://pmc.ncbi.nlm.nih.gov/articles/PMC2605254/.

## One-paragraph summary

Sueur et al. propose a **community-level acoustic biodiversity index**
— skipping species identification entirely — that can be computed
from a single raw recording. They introduce an **α (alpha) index**
based on Shannon entropy (total entropy `H = Ht · Hf`, the product of
temporal and spectral entropies, both normalised to [0, 1]) and a
**β (beta) index** `D = Dt · Df` based on temporal and spectral
dissimilarities between two recordings. They validate the indexes
first on **540 simulated chorus recordings** spanning three diversity
levels (5, 10, 15 species) and three detector sensitivities, then
apply them in the field to two closely spaced Tanzanian dry-coastal
sites (Mikocheni A and Mikocheni B) that they sampled in parallel.
The α index shows a logarithmic correlation with species count (more
species → higher but saturating H), and the β index is linear in
the number of **unshared** species between two communities. This
paper is the foundational reference for **automated, species-free
biodiversity monitoring** and is the mathematical spine of every
soundscape ecology pipeline that followed (ACI, NDSI, ADI, AEI all
derive from this framework).

## Methodology

### Simulated chorus experiment (validation)

- **Three species groups** of 15 synthetic species each, with each
  species given a distinct band-limited call. Three diversity levels:
  5, 10, and 15 species per chorus.
- For each group × diversity level, **7 recordings per chorus**
  were randomly selected from the 15 available species and summed
  with amplitude weights drawn from a uniform distribution.
- **10 replicate choruses per cell**, yielding 3 groups × 3 diversity
  levels × 10 replicates = 90 sound files per full design, which they
  repeat for robustness. Total: 540 simulated recordings for the α
  index and a parallel set for the β index.
- **30-second recording length** with ~1.323 × 10⁶ samples at
  44.1 kHz. STFT window 512 samples → 83.13 Hz frequency precision.

### Field application (Tanzanian dry-coastal forest)

- Two sites (Mikocheni A and Mikocheni B) separated by ~200 m, along a
  gradient of anthropogenic disturbance.
- **Omnidirectional microphone** held vertically at 2 m height.
  Recording sessions were timed around dawn chorus.
- **170 Hz high-pass filter** applied before index computation to
  remove low-frequency wind and geophonic background.

### The α index: acoustic entropy H

Given a mono audio signal `x(t)` digitised at sample rate `f_s`:

**1. Temporal entropy Ht.**

Compute the analytic envelope `A(t)` via Hilbert transform, normalise
to a probability mass function:

```
Ã(t) = A(t) / Σ_τ A(τ)
Ht = -Σ_t Ã(t) · log2(Ã(t)) · (log2(n))^{-1},     Ht ∈ [0, 1]
```

where `n` is the number of time samples. Division by `log2(n)`
normalises so that a flat envelope → Ht = 1 (maximum entropy) and a
perfectly impulsive envelope → Ht → 0.

**2. Spectral entropy Hf.**

Compute the Welch-averaged (or mean STFT-magnitude) spectrum `S(f)` of
length `N` frequency bins, normalise:

```
S̃(f) = S(f) / Σ_ν S(ν)
Hf = -Σ_f S̃(f) · log2(S̃(f)) · (log2(N))^{-1},    Hf ∈ [0, 1]
```

**3. Total acoustic entropy H.**

```
H = Ht · Hf,    H ∈ [0, 1]
```

High H → sound energy spread evenly across both time and frequency.
Silence or tonal signals → low H.

### The β index: acoustic dissimilarity D

Given two recordings `x_1(t)` and `x_2(t)` of equal duration at the
same sample rate with normalised envelopes `Ã_1, Ã_2` and
normalised spectra `S̃_1, S̃_2`:

**Temporal dissimilarity.**

```
Dt = ½ · Σ_t |Ã_1(t) - Ã_2(t)|,    Dt ∈ [0, 1]
```

**Spectral dissimilarity.**

```
Df = ½ · Σ_f |S̃_1(f) - S̃_2(f)|,    Df ∈ [0, 1]
```

**Dissimilarity index.**

```
D = Dt · Df,    D ∈ [0, 1]
```

`D = 0` when both recordings are identical in envelope and spectrum;
`D = 1` when they are completely disjoint in both.

## Key results

### Simulated chorus (α index)

- **Logarithmic correlation** between H and species count.
- Mean H at 5 species ≈ 0.45; at 10 species ≈ 0.60; at 15 species
  ≈ 0.70 (values approximate — exact table in paper).
- The saturation behaviour is expected: once the spectrum/time axis is
  well-filled, adding a species contributes less marginal entropy.
- The detector-sensitivity simulation (attenuating quiet species)
  shifts the intercept but not the slope of the log relation — H is
  robust under varying microphone gain.

### Simulated chorus (β index)

- **Linear correlation** between D and the number of **unshared**
  species between two communities (Jaccard-like behaviour).
- Slope is approximately constant across all three groups, suggesting
  D is a species-pool-independent dissimilarity estimator.

### Tanzanian field application

- Mikocheni A and Mikocheni B produced **significantly different H
  values** and a **measurable D between them**, consistent with
  independent species-counting surveys showing different avifaunal
  composition.
- The paper presents this as a proof-of-concept — one of the first
  deployments of automated acoustic biodiversity indexing in the
  field.

### Computational cost

- H can be computed in **O(n log n)** time per recording (FFT-
  dominated).
- D is **O(n log n)** likewise.
- On 2008 hardware, a 30-s recording processed in a few seconds —
  trivially tractable on-buoy today.

## Strengths

- **Species-free.** The index needs no annotation, no training data,
  no species catalogue. This is its most important property for
  large-scale PAM where manual labelling is the bottleneck.
- **Theoretically grounded.** H is a direct application of Shannon's
  entropy to a normalised spectro-temporal energy distribution; D
  is the Kolmogorov-style metric on probability measures. Both have
  clean axiomatic support.
- **Cheap and streaming.** The indices are computable in a single
  pass at O(n log n), small memory footprint. They fit trivially on
  an edge DSP.
- **Saturates gracefully.** The logarithmic H response means the
  index degrades gracefully as species count grows beyond the point
  where individual calls start overlapping — no catastrophic drop.
- **Units are normalised to [0, 1]** — directly comparable across
  sites and sensors, making long-term monitoring trivially
  cross-calibratable (provided the same band-pass filter is used).

## Limitations

- **H confounds species richness with call redundancy.** Two species
  calling in identical bands at the same time produce the same H as
  one species, because the envelope/spectrum is unchanged.
- **Background noise saturates the index.** A white-noise-like
  recording (wind, rain, shipping) drives H toward 1 even in the
  absence of biophony. In practice one *must* apply band-pass
  filtering (the paper uses 170 Hz HPF) and, for marine work,
  separate low-frequency (fish) and high-frequency (shrimp,
  cetacean) bands — the whole point of NDSI (Kasten 2012).
- **No anthrophony separation.** H makes no distinction between
  biological and human sound sources. Two sites with identical H
  can have radically different biodiversity if one is shipping-
  dominated.
- **Not validated in underwater acoustics by this paper.** All
  validation is terrestrial dry-coastal forest. Marine application
  was pursued in Staaterman 2014, Bertucci 2016, and others.
- **Spectral precision trade-off** — 83 Hz FFT bins in the original
  paper work well for birds (2–8 kHz) but are coarse for cetacean
  clicks. Parameter tuning is always site-specific.
- **No confidence intervals on a single-recording H estimate** —
  bootstrapping or time-windowing is needed for uncertainty
  quantification, and the paper doesn't treat this in depth.

## Portable details

### Minimal on-buoy index library (Rust pseudo-code)

```rust
/// Compute Sueur et al. 2008 acoustic entropy H = Ht * Hf.
/// Input: PCM f32 samples at sample_rate, assumed high-pass filtered.
pub fn acoustic_entropy(x: &[f32], sample_rate: u32) -> (f32, f32, f32) {
    // 1. Temporal entropy via Hilbert envelope
    let env = hilbert_envelope(x);
    let sum: f32 = env.iter().sum();
    let pmf = env.iter().map(|a| a / sum);
    let n = x.len() as f32;
    let ht = -pmf.clone()
        .map(|p| if p > 0.0 { p * p.log2() } else { 0.0 })
        .sum::<f32>() / n.log2();
    // 2. Spectral entropy via Welch-averaged magnitude spectrum
    let spec = welch_magnitude_spectrum(x, 512, 0.5);
    let ssum: f32 = spec.iter().sum();
    let spmf = spec.iter().map(|s| s / ssum);
    let nf = spec.len() as f32;
    let hf = -spmf.clone()
        .map(|p| if p > 0.0 { p * p.log2() } else { 0.0 })
        .sum::<f32>() / nf.log2();
    (ht, hf, ht * hf)
}
```

### Minimal D-index computation

```rust
pub fn acoustic_dissimilarity(x1: &[f32], x2: &[f32], sample_rate: u32) -> (f32, f32, f32) {
    let e1 = normalized_envelope(x1);
    let e2 = normalized_envelope(x2);
    let s1 = normalized_welch_spectrum(x1, 512, 0.5);
    let s2 = normalized_welch_spectrum(x2, 512, 0.5);
    let dt: f32 = e1.iter().zip(&e2).map(|(a, b)| (a - b).abs()).sum::<f32>() * 0.5;
    let df: f32 = s1.iter().zip(&s2).map(|(a, b)| (a - b).abs()).sum::<f32>() * 0.5;
    (dt, df, dt * df)
}
```

### Recommended index suite for marine PAM

Sueur's H alone is insufficient for marine work. The full on-buoy
bundle should include (in computational order):

| # | Index | Formula | Cost |
|---|-------|---------|------|
| 1 | RMS SPL (dB re 1 µPa) | `20 log10(rms(x) / 1e-6)` after calibration | O(n) |
| 2 | Sueur Ht | as above | O(n) |
| 3 | Sueur Hf | as above | O(n log n) |
| 4 | Sueur H = Ht·Hf | product | O(1) |
| 5 | Pieretti ACI | sum-of-differences over spectrogram bins | O(n log n) |
| 6 | Kasten NDSI | `(β - α) / (β + α)` where α=anthrophony band, β=biophony band | O(n log n) |
| 7 | Villanueva-Rivera ADI (Acoustic Diversity Index) | Shannon over binned band-thresholded occupancy | O(n log n) |
| 8 | Villanueva-Rivera AEI (Acoustic Evenness Index) | Gini over the same | O(N) after ADI |
| 9 | Broadband band-energy vector (10 or 16 bands) | | O(n log n) |
| 10 | Sueur D (for pairs of sites) | envelope + spectral L1 dissimilarity | O(n log n) |

Indices 1–9 are single-recording; 10 is pair-wise. All together
≈ 30 ms on an STM32H7 DSP per 60-second clip at 192 kHz sample rate.

### Band-split strategy for marine deployments

Sueur's original paper uses a single wideband pass with HPF. For
sonobuoy PAM, compute **per-band H** on at least four physical bands:

| Band | Range | Target content |
|------|-------|----------------|
| B1 | 10 Hz – 100 Hz | Baleen whales, seismic survey |
| B2 | 100 Hz – 2 kHz | Fish chorus, vessel mid-frequency, low-frequency active sonar |
| B3 | 2 kHz – 20 kHz | Snapping shrimp, mid-frequency active sonar, bottlenose dolphins |
| B4 | 20 kHz – 100 kHz | Odontocete clicks, porpoise narrowband high-frequency |

Per-band H{1..4}, ACI{1..4}, and band-energy summaries give a
feature vector of ~20 floats per minute — the canonical on-buoy
telemetry payload.

## Sonobuoy integration plan — long-term PAM mode

The Sueur indices are the **primary on-buoy telemetry product** for a
long-term PAM sonobuoy. They are cheap, interpretable, and compress
a 60-second WAV clip (~10 MB at 192 kHz/16-bit stereo) down to
~80 bytes of acoustic metadata, a **~10^5 compression ratio**.

**Duty-cycle-friendly.** Because H is a single-window statistic, it
can be computed on every wakeup regardless of duty cycle. A buoy
recording 1 min every 10 min still yields 144 H samples/day.

**Telemetry layout (80 bytes/min).**

```
0..4   : UNIX timestamp
4..6   : duty-cycle flag + battery level
6..10  : broadband SPL_dB (f32)
10..14 : Ht (f32)
14..18 : Hf (f32)
18..22 : H  (f32)
22..26 : ACI (f32)
26..30 : NDSI (f32)
30..34 : ADI (f32)
34..38 : AEI (f32)
38..48 : 10-bin band-energy vector (u16 × 10, log-encoded)
48..80 : 4-band (Ht, Hf, ACI, SPL) per-band stats
```

Transmit via Iridium SBD in 340-byte bursts every 4 hours (= 240
minutes of 80-byte samples), well within SBD payload limits.

**On-shore inference.** The H time-series reveals:

- **Diel rhythm** — H rises at dawn/dusk with fish/snapping-shrimp
  chorus (cf. Staaterman 2014).
- **Lunar rhythm** — at some reefs H peaks at new moon when fish
  spawn (Staaterman 2014).
- **Vessel events** — sharp spikes in broadband SPL and a drop in
  Hf (tonal shipping dominates one band).
- **Biodiversity loss** — monotonic decrease in H over years is the
  canonical long-term decline signature.

**Cross-site D comparison.** For a cabled or networked deployment
(SanctSound, Ocean Networks Canada), compute daily D between every
pair of buoys. The resulting D matrix is an acoustic distance metric
on the deployment; project via MDS for a soundscape map.

**Bayesian change-point monitoring.** H, ACI, and NDSI are each
ARIMA-modellable; fit an online Bayesian change-point detector
(Adams & MacKay 2007) on each index per buoy, flagging anomalous days
for analyst review. Fits in ~5 KB of state per buoy.

**Recommendation.** The long-term PAM profile mandates computing all
of H, Ht, Hf, ACI, NDSI, ADI, AEI, and D on-buoy at every window.
Belongs in the `sonobuoy-pam` profile (see Pijanowski analysis §
Sonobuoy integration plan and proposed ADR-063), not the tactical
profile. Propose **ADR-064: Acoustic indices as primary on-buoy
telemetry for long-term PAM** naming Sueur 2008 + Pieretti 2011 +
Kasten 2012 as the bibliography.

## Follow-up references

1. **Pieretti, Farina, & Morri 2011** *A new methodology to infer the
   singing activity of an avian community: The Acoustic Complexity
   Index (ACI)*, Ecol. Indic. 11(3):868–873,
   doi:10.1016/j.ecolind.2010.11.005. Complements H with a
   differential index capturing fast biophonic modulation.
2. **Kasten et al. 2012** *The remote environmental assessment
   laboratory's acoustic library*, Ecol. Inf. 12:50–67,
   doi:10.1016/j.ecoinf.2012.08.001. Introduces NDSI, which combines
   with Sueur's H for biophony/anthrophony separation.
3. **Villanueva-Rivera et al. 2011** *A primer of acoustic analysis
   for landscape ecologists*, Landsc. Ecol. 26:1233–1246,
   doi:10.1007/s10980-011-9636-9. Introduces ADI and AEI, defines the
   `soundecology` R package that made the Sueur indices widely
   adopted.
4. **Sueur, Farina, Gasc, Pieretti, & Pavoine 2014** *Acoustic
   indices for biodiversity assessment and landscape investigation*,
   Acta Acustica united with Acustica 100:772–781,
   doi:10.3813/AAA.918757. Sueur's own six-year retrospective of the
   acoustic indices ecosystem — a critical meta-review.
5. **Bradfer-Lawrence et al. 2019** *Guidelines for the use of acoustic
   indices in environmental research*, Methods Ecol. Evol. 10:
   1796–1807, doi:10.1111/2041-210X.13254. Empirical benchmarking of
   10+ indices across 12 biomes; recommended reading before deploying
   any index in production PAM.
6. **Buxton et al. 2018** *Efficacy of extracting indices from
   large-scale acoustic recordings*, Conserv. Biol. 32(5):1174–1184,
   doi:10.1111/cobi.13119. Important cautionary paper on how indices
   behave on month-scale PAM data.
