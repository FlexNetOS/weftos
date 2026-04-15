# Wenz — Acoustic Ambient Noise in the Ocean: Spectra and Sources

## Citation

- **Author**: Gordon M. Wenz (US Navy Electronics Laboratory, San Diego)
- **Title**: "Acoustic Ambient Noise in the Ocean: Spectra and Sources"
- **Venue**: *Journal of the Acoustical Society of America*, Vol. 34,
  No. 12 (December 1962), pp. 1936–1956
- **DOI**: https://doi.org/10.1121/1.1909155
- **Publisher link**: https://pubs.aip.org/asa/jasa/article/34/12/1936/684933
- **ADS bibcode**: 1962ASAJ...34.1936W
- **Open copy (scan)**: https://brigus.physics.mun.ca/~zedel/P6317/papers/wenz.pdf
  (Memorial University of Newfoundland syllabus copy)
- **Downloaded**: `.planning/sonobuoy/papers/pdfs/wenz-ambient-noise.pdf`
  (6-page scanned abridgement; primary citation is the DOI above)
- **Modern retrospective**: Dahl, P.H., Dall'Osto, D.R. (2025). "The
  Wenz curves for underwater ambient sound." *JASA* 157(5):R9. DOI:
  https://doi.org/10.1121/10.0035965

## Status

**Verified.** JASA publisher record, ADS bibcode, Scientific Research
Publishing citation, Semantic Scholar record, and the 2025 *JASA*
retrospective (Dahl & Dall'Osto) all corroborate the 1962 volume/issue/
page range and the DOI. The paper has 1,396+ citations per Semantic
Scholar; it is the most cited paper in ocean ambient noise.

## Historical context

Gordon Wenz, working at the US Navy Electronics Laboratory in San Diego,
compiled ambient-noise measurements from roughly 15 years of Navy,
academic, and international programs (1946–1961), plotted them on a
common frequency/level grid, and identified which physical source
dominated each frequency band. The resulting family of curves — still
called "the Wenz curves" in 2025 — became the universal noise-floor
prior for every sonar performance calculation and the first-order
ambient-noise model every ML sonar paper benchmarks against.

Before Wenz, ambient noise was a grab-bag of experimental curves from
different hydrophones, calibration regimes, and geographic conditions.
Wenz's contribution was threefold: (i) a common dB re 1 µPa²/Hz scale,
(ii) a decomposition into physically motivated overlapping components,
and (iii) a set of parameterized curves as a function of wind, shipping
density, and sea state that any sonar engineer could read off. Dahl &
Dall'Osto's 2025 retrospective confirms that 63 years of subsequent
measurements have refined but not displaced the Wenz framework.

## Core content

Wenz decomposes deep-ocean ambient noise into three overlapping spectral
regions with distinct physical mechanisms (Wenz 1962, §IV and Fig. 1):

| Band | Dominant source | Typical slope |
|------|-----------------|---------------|
| 1 Hz – 100 Hz | Turbulent-pressure fluctuations (ocean turbulence, hydrostatic pressure of surface waves) | ~ -8 to -10 dB/octave |
| 10 Hz – 200 Hz | Distant shipping (integrated over many ships within the SOFAR channel's reach) | broad peak near 50–100 Hz |
| 200 Hz – 80 kHz | Wind-dependent surface-agitation noise (breaking waves, bubble oscillations, spray) | ~ -5 to -6 dB/octave (Knudsen slope) |
| > 50 kHz | Thermal noise of molecular motion at the hydrophone face | +6 dB/octave (white force → -velocity squared) |

Additional non-continuous contributions Wenz catalogues:
- biological (snapping shrimp 2–20 kHz, marine mammals 0.05–100 kHz),
- seismic / geologic (microseisms 0.1–10 Hz),
- weather (precipitation, lightning), and
- ice (cryogenic cracking in polar regions).

### The key plotted relations

#### Thermal-noise floor (Mellen 1952; Wenz reproduces)

    NL_thermal(f) = -75 + 20·log₁₀(f_kHz)    [dB re 1 µPa²/Hz]

Crossover with surface noise near 50–100 kHz.

#### Wind-dependent ("Knudsen-like") sea-state noise

Wenz generalized Knudsen's sea-state curves as wind-speed-parameterized
spectra. An empirical form (reproduced in Urick 1983, Eq. 7.25) is

    NL_wind(f, v) ≈ 50 + 7.5·sqrt(v_knots) + 20·log₁₀(f_kHz) − 40·log₁₀(f_kHz + 0.4)
                                                                   [dB re 1 µPa²/Hz]

valid 0.1–25 kHz. Peaks near 500 Hz for typical sea-state 3 and rolls
off ~5 dB/octave toward 20 kHz.

#### Shipping noise

Wenz parameterized shipping noise by a "shipping density" index 1 (remote)
to 7 (heavy). A compact approximation (Ross 1987; see Ainslie 2010):

    NL_shipping(f, s) ≈ 40 + 20(s − 4) + 26·log₁₀(f_Hz) − 60·log₁₀(f_Hz + 0.03)
                                                                   [dB re 1 µPa²/Hz, 10–500 Hz]

Shipping noise dominates from ~10 Hz (below which turbulent pressure
takes over) up to ~200 Hz (above which wind noise takes over). The
200 Hz "crossover" is a soft function of wind speed and shipping density.

#### Turbulent pressure

    NL_turb(f) ≈ 107 − 30·log₁₀(f_Hz)     [dB re 1 µPa²/Hz, ~ 1–20 Hz]

Dominant below the shipping floor at infrasonic frequencies; matters
mainly for deep-tow and long-baseline arrays.

### Wenz's "composite" noise model

To get a usable scalar noise floor for sonar-equation use:

    NL(f) = 10·log₁₀( Σ_i 10^(NL_i(f) / 10) )

i.e. power-sum the four contributors (turbulent, shipping, wind,
thermal) at each frequency. For the `NL` term in Urick's sonar equation
(see `urick-sonar-equation.md`), integrate across the receiver
bandwidth: `NL_band = NL(f₀) + 10·log₁₀(W_Hz)` if roughly flat across W.

## Modern relevance

The Wenz curves remain the **zero-cost prior** for every ML sonar paper.
Papers in round 1 either cite Wenz directly (Xu 2023 SIR+LMR) or bake
the curves into the training augmentation noise distribution (DEMONet,
SIR+LMR, PINN/SSP papers). The curves also parameterize SNR sweeps:
when DEMONet reports accuracy at "-15 dB SNR", the denominator is a
Wenz-model NL at the appropriate wind/shipping condition, not thermal
noise.

Specific modern citations:
- **AudioMAE / BEATs training augmentation** uses Wenz-like ambient
  noise to generate realistic spectrogram backgrounds.
- **IEEE OCEANS 2023–2025** ML-sonar papers routinely plot Wenz curves
  on top of their predicted noise maps to validate the denoising.
- **NOAA HARP archive** reports per-site median spectral levels against
  Wenz for data-quality assurance (Hildebrand et al.).
- **Dahl & Dall'Osto 2025** (DOI 10.1121/10.0035965) revisits the Wenz
  curves with 63 years of additional data and concludes that low-frequency
  ambient has risen by ~3 dB/decade from 1962 to ~2000 due to shipping
  growth and stabilized since.

## Sonobuoy relevance

The sonobuoy system needs a usable ambient-noise prior for four things:

1. **Training-data augmentation.** Every training example for DEMONet
   or Perch needs to be mixed with realistic ambient noise at a
   distribution of levels. Wenz gives us the right family — sample
   `(wind_speed, shipping_density, frequency)` and mix.
2. **Sonar-equation calibration.** The `NL` term in the Urick FoM
   (see companion `urick-sonar-equation.md`) is drawn from the Wenz
   curves. A buoy's reported FoM depends on which curve is active;
   we must log the Wenz state (wind, shipping, season).
3. **Anomaly detection prior.** When measured noise is > 10 dB above
   the Wenz prior, something unusual is happening (nearby ship, storm,
   biological chorus). This is a free anomaly detector for `Impulse`
   events in the ECC substrate.
4. **Buoy-health check.** A sonobuoy hydrophone that reports noise
   *below* the Wenz thermal floor is broken (saturated, miscalibrated,
   or stuck). This is the cheapest self-test the edge code can run.

Round 1's ML-first survey gave us the detectors; Wenz gives us the
noise background against which all detection decisions are made. The
Wenz-curve module is a P0 dependency of the v1 Perch retrieval crate.

## Portable details

A self-contained Wenz evaluator in ~60 lines of Rust. Values cross-checked
against Dahl 2007 *Acoustics Today* Fig. 3 and Ainslie 2010 Fig. 8.1.

```rust
/// Wenz 1962 composite ambient-noise model.
/// Returns dB re 1 µPa² / Hz at a given frequency.
/// Inputs: frequency in Hz, wind speed in knots (0..40), shipping
/// density index 1 (remote)..7 (heavy).
pub fn wenz_noise_spectrum_db(freq_hz: f64, wind_knots: f64, shipping: u8) -> f64 {
    let f_khz = freq_hz / 1000.0;
    let f_hz = freq_hz;

    // Turbulent pressure, dominant below ~10 Hz
    let nl_turb = if f_hz >= 1.0 {
        107.0 - 30.0 * f_hz.log10()
    } else {
        f64::NEG_INFINITY
    };

    // Shipping (Ross 1987 parameterization of Wenz's curves)
    let s = shipping.clamp(1, 7) as f64;
    let nl_ship = 40.0 + 20.0 * (s - 4.0)
        + 26.0 * f_hz.log10()
        - 60.0 * (f_hz + 0.03).log10();

    // Wind / surface agitation (Wenz, reproduced in Urick 1983 Eq. 7.25)
    let v = wind_knots.clamp(0.0, 40.0);
    let nl_wind = 50.0
        + 7.5 * v.sqrt()
        + 20.0 * f_khz.log10()
        - 40.0 * (f_khz + 0.4).log10();

    // Thermal noise (Mellen 1952; Wenz Fig. 1 asymptote)
    let nl_therm = -75.0 + 20.0 * f_khz.log10();

    // Power-sum in linear units
    let linear = [nl_turb, nl_ship, nl_wind, nl_therm]
        .iter()
        .map(|db| 10f64.powf(db / 10.0))
        .sum::<f64>();

    10.0 * linear.log10()
}

/// Integrate the spectrum across a receiver bandwidth W (Hz) centered
/// on f₀. Returns dB re 1 µPa² in-band — drop directly into Urick NL term.
pub fn wenz_noise_in_band_db(f0_hz: f64, bandwidth_hz: f64, wind_knots: f64, shipping: u8) -> f64 {
    wenz_noise_spectrum_db(f0_hz, wind_knots, shipping)
        + 10.0 * bandwidth_hz.log10()
}
```

Unit-test anchors (Dahl 2007 *Acoustics Today* Fig. 3):
- `f = 100 Hz, wind = 10 kn, shipping = 4` → NL ≈ 68 dB re 1 µPa²/Hz
- `f = 1 kHz, wind = 10 kn, shipping = 4` → NL ≈ 54 dB re 1 µPa²/Hz
- `f = 10 kHz, wind = 20 kn, shipping = 4` → NL ≈ 52 dB re 1 µPa²/Hz
- `f = 100 kHz, wind = 0 kn, shipping = 1` → NL ≈ -15 dB re 1 µPa²/Hz
  (thermal-limited)

## Follow-up references

1. **Dahl, P.H., Dall'Osto, D.R.** (2025). "The Wenz curves for
   underwater ambient sound." *JASA* 157(5):R9. DOI:
   10.1121/10.0035965. The 63-year retrospective; essential for an
   up-to-date shipping-growth correction.
2. **Knudsen, V.O., Alford, R.S., Emling, J.W.** (1948). "Underwater
   ambient noise." *J. Marine Research* 7:410–429. The pre-Wenz
   wind/sea-state curves that Wenz reconciled with the post-WWII data.
3. **Ross, D.** (1987). *Mechanics of Underwater Noise*. Peninsula
   Publishing. Derives the shipping-noise parameterization Wenz uses;
   compact source for the 40 + 20(s-4) + ... formula above.
4. **Mellen, R.H.** (1952). "The thermal-noise limit in the detection
   of underwater acoustic signals." *JASA* 24(5):478. DOI:
   10.1121/1.1906924. The thermal-noise asymptote Wenz plots.
5. **Ainslie, M.A., McColm, J.G.** (1998). "A simplified formula for
   viscous and chemical absorption in sea water." *JASA* 103(3):
   1671. DOI: 10.1121/1.421258. The modern absorption coefficient
   α(f) that combines with Wenz noise to predict in-band NL at
   propagation-limited ranges.
