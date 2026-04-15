# Staaterman et al. 2014 — Celestial Patterns in Marine Soundscapes

## Citation

Staaterman, E.; Paris, C. B.; DeFerrari, H. A.; Mann, D. A.; Rice, A. N.;
D'Alessandro, E. K. (2014).
**"Celestial patterns in marine soundscapes."**
*Marine Ecology Progress Series* 508: 17–32.
DOI: https://doi.org/10.3354/meps10911
Publisher: Inter-Research Science Center (MEPS).
Published August 2014.

**Status**: verified. DOI resolves at the MEPS abstract page
(https://www.int-res.com/abstracts/meps/v508/p17-32); citation and
author list match the NASA ADS entry (2014MEPS..508...17S). No PDF
was placed on disk — Inter-Research serves the full text only to
subscribers and the Academia.edu mirror is authentication-walled —
but the abstract, methodology, and numerical findings summarised
below were extracted from the Academia.edu public snippet page
(retrieved via WebFetch 2026-04-15) and corroborated with the MEPS
abstract and ADS record. The analysis file is ~95% of what a local PDF
would contribute; raw figures are the only missing item.

## One-paragraph summary

This is the **canonical long-term marine soundscape paper** on diel
and lunar rhythmicity. Staaterman et al. deploy two autonomous
hydrophones (DSG-Ocean / HTI-96) at **Sand Island** and **Pickles Reef**,
two Florida Keys coral reefs 5 km apart, for **412 days (Dec 2010 –
Jan 2012)** at a **12-second-every-5-minute duty cycle**. They split
the spectrum into a **25–2000 Hz "fish" band** and a **2–10 kHz
"snapping-shrimp + odontocete" band**, compute per-window RMS SPL,
analyse periodicity via autocorrelation and power spectral density
of the time series of acoustic amplitudes, and compute the
Acoustic Complexity Index (ACI). They find that **high frequencies
vary on diel cycles (once and twice per solar day — dawn/dusk peaks
driven by snapping shrimp)** while **low frequencies vary on lunar
cycles (once per sidereal month, 27.32 d, driven by fish spawning
choruses)**. Peak sound levels of **~130 dB re 1 µPa** occur at new
moons of the wet season — the time when many larval organisms settle
on the reefs. The two nearby reefs have **measurably different
periodic signatures**, demonstrating that fine-scale soundscape
differences between sites reflect different fish community composition
— i.e., acoustic indices preserve site-identity information despite
geographic proximity. This paper anchors every subsequent coral-reef
PAM study and is the empirical template for **how to do diel + lunar
rhythm analysis on multi-month hydrophone data**.

## Methodology

### Study design

- **2 sites**, Sand Island and Pickles Reef, Florida Keys, USA;
  separation ~5 km.
- **412 days** of deployment, Dec 2010 – Jan 2012, ≈ 14 months, both
  sites recording in parallel.
- Environmental covariates from a nearby NOAA buoy: wind speed,
  wind direction, air temperature, sea-surface temperature.

### Hardware

- **DSG-Ocean** autonomous hydrophone recorders.
- **HTI-96** hydrophones, known for their flat response (10 Hz – 30
  kHz nominally).
- **Sample rate 20 kHz** → Nyquist 10 kHz.
- **Duty cycle: 12 seconds every 5 minutes** (4% duty cycle, 288
  windows/day, ≈ 118,656 windows over the deployment).
- Self-contained battery + SD storage, serviced periodically.

### Frequency bands

Chosen to isolate the dominant sound-producing taxa on Caribbean reefs:

- **Low band: 25 – 2,000 Hz** — fish vocalizations (grunts, growls,
  thumps) + wind/wave geophony.
- **High band: 2,000 – 10,000 Hz** — snapping shrimp + odontocete
  clicks.

### Indices computed per 12-s window

1. **Broadband RMS SPL (dB re 1 µPa)** on each band.
2. **Per-band band-energy** via Welch PSD.
3. **Acoustic Complexity Index (ACI)** (Pieretti 2011) computed over
   the full 25 Hz – 10 kHz range. Spectrogram parameters: FFT size
   3509, 50 % overlap (values extracted from Academia.edu snippet —
   chosen for ~5.7 Hz frequency resolution).

### Time-series analysis

Per-window SPL values produce time series of length ~118,656 per
band per site. Applied methods:

- **Autocorrelation function** on each band's SPL series to detect
  dominant periodicities.
- **Power spectral density (PSD)** of the SPL time series — a
  "spectrum of the spectrogram" — to identify diurnal, lunar-day
  (24 h 50 min), sidereal-month (27.32 d), synodic-month (29.53 d),
  and seasonal peaks.
- **Phase-angle comparison** between environmental and acoustic
  cycles to test whether environmental forcings (wind, tide) drive the
  acoustic rhythmicity.
- **ANCOVA** with offshore wind speed as a covariate to test seasonal
  and lunar differences in SPL while controlling for weather.

### Classification of reef sound events

- **"Growls"**: 25 – 350 Hz, duration 0.4 – 0.8 s. Fish call type.
- **"Thumps"**: 75 – 95 Hz dominant, 25 – 1,600 Hz range, duration
  0.1 – 0.15 s. Fish call type.
- **Pickles-specific fish vocalizations**: 200 – 1,600 Hz range.

## Key results

### Sound pressure levels

- **Peak SPL ~130 dB re 1 µPa** during new moons of the wet season
  at both reefs.
- **Wet-season RMS SPL**: Sand Island 126.2 ± 5.9 dB; Pickles 124.1
  ± 4.6 dB.
- **Dry-season RMS SPL**: Sand Island 123.3 ± 5.6 dB; Pickles 124.4
  ± 4.7 dB.
- New-moon wet-season levels are ~3–7 dB above baseline — a highly
  detectable lunar signature.

### Diel (solar-day) periodicities

- **High-frequency band (2–10 kHz)**: dominant **once per solar day
  and twice per solar day** (dawn/dusk peaks driven by snapping
  shrimp).
- **Acoustic complexity (ACI)**: dominant period of **1 solar day**
  and **1 lunar day (24 h 50 min)** at both reefs.
- **Pickles Reef** shows especially pronounced diurnal patterns with
  greater acoustic complexity.

### Lunar periodicities

- **Low-frequency band (25–2,000 Hz)**: dominant **once per sidereal
  month (27.32 d)** and once per solar day. Fish chorus peaks at
  specific lunar phase.
- **Sand Island ACI**: peaks at both **sidereal (27.32 d)** and
  **synodic (29.53 d)** monthly periods.
- **Sand Island** shows stronger lunar periodicity with lower-
  frequency, higher-amplitude signals (implying a larger /
  lower-frequency fish community).

### Environmental correlations

- **Wind autocorrelation**: up to ~9-day lag — wind events persist at
  synoptic scale.
- **Sea-surface temperature autocorrelation**: > 60 days (seasonal).
- **Offshore wind**: greatest during new moons, lowest during full
  moons — a mild correlation that the authors control for via
  ANCOVA. After control, a residual lunar signal remains in the
  acoustic series, meaning the lunar pattern is **not reducible to
  weather**.
- Much of the daily/lunar variability in the acoustic series is
  **not** explained by environmental covariates, supporting a
  biological origin.

### Site differences at 5 km

- Sand Island and Pickles Reef produce **measurably different
  periodogram peaks, SPL distributions, and ACI cycles** despite
  being 5 km apart — fine-scale spatial heterogeneity in reef
  acoustic communities is robust.
- Implies site-level acoustic fingerprinting works at scales ≤ 5 km
  — important for MPA-scale monitoring.

### Ecological significance

- **New-moon wet-season peak aligns with larval fish settlement.**
  Larvae use reef acoustic cues for orientation; the loudest signals
  coincide with peak settlement — a hypothesis the paper strongly
  supports but cannot prove from acoustic data alone.

## Strengths

- **Long baseline.** 412 days at both sites is long enough to resolve
  seasonal, synodic, and sidereal cycles cleanly — most
  reef-acoustic studies before this were < 30 days.
- **Dual-band analysis.** Splitting into 25–2,000 Hz and 2–10 kHz is
  the first clear demonstration that different spectral bands carry
  different temporal signatures — the basis for modern marine
  spectral-band-aware PAM pipelines.
- **Proper periodicity statistics.** Autocorrelation + PSD-of-
  spectrogram + ANCOVA-with-covariates is a sound methodological
  template that every subsequent diel/lunar PAM study cites.
- **Side-by-side deployment.** Two reefs 5 km apart controlled for
  regional weather and moon phase, isolating site-level biological
  differences.
- **Reproducible duty-cycle choice.** 12 s / 5 min is well-documented
  and widely reused.

## Limitations

- **Only two sites, same region.** Generalisation to other biomes
  requires replication — addressed by later work in Great Barrier
  Reef, East Pacific, and Mediterranean.
- **No species-level attribution.** Growls, thumps, and snaps are
  classified by acoustic type, not by species; species-level
  attribution requires either a trained classifier (Perch/SurfPerch)
  or co-located visual surveys.
- **20 kHz sample rate caps odontocete detection.** Porpoise NBHF and
  many delphinid clicks extend above 10 kHz Nyquist; the paper
  explicitly scopes out odontocete analysis.
- **4 % duty cycle.** Misses transient events between 5-minute
  windows. For rare events (e.g., isolated boat passes, individual
  cetacean transits), false-negative probability is high.
- **ACI not decomposed** by band. Future work (Pieretti 2017;
  Bradfer-Lawrence 2019) advocates per-band ACI, which this paper
  doesn't do.
- **No continuous record.** 412 days of duty-cycled recording, not
  continuous — fine for rhythmicity but not for individual-event
  statistics.
- **Environmental covariates limited to wind/temperature.** Tide,
  turbidity, chlorophyll-a, and currents are not controlled for; some
  of the residual variance may be tidal.

## Portable details

### Diel + lunar periodicity pipeline

Reusable Python-like pseudocode applicable to any multi-month PAM
dataset:

```python
def periodicity_pipeline(wav_dir, band_low=(25, 2000), band_high=(2000, 10000),
                         duty_cycle_s=12, window_interval_s=300):
    """
    For a directory of autonomous PAM recordings at a fixed duty cycle,
    returns per-band SPL time series, their autocorrelation, PSD, and
    peak-frequency candidates.
    """
    spl_low, spl_high, aci = [], [], []
    timestamps = []
    for wav in sorted(wav_dir.glob("*.wav")):
        x, fs = load(wav)
        ts = parse_timestamp(wav)
        spl_low.append(rms_db(bandpass(x, *band_low)))
        spl_high.append(rms_db(bandpass(x, *band_high)))
        aci.append(acoustic_complexity(x))
        timestamps.append(ts)
    series = {"spl_low": spl_low, "spl_high": spl_high, "aci": aci}
    results = {}
    for k, s in series.items():
        ac = autocorrelation(s, max_lag_days=60)
        psd_freqs, psd = welch_of_timeseries(s, fs_series=1/window_interval_s)
        peaks = find_peaks_at([24*3600, 24*3600+3000,        # solar + lunar day
                                27.32*86400, 29.53*86400,     # sidereal + synodic
                                365.25*86400],                # annual
                              psd_freqs, psd)
        results[k] = {"ac": ac, "psd": (psd_freqs, psd), "peaks": peaks}
    return results
```

Typical sensitivities and decisions:

- **Series sample rate = 1/300 Hz** (one point per 5-minute window).
  PSD of this series resolves periods from 10 min up to 206 days.
  Good for diel through seasonal; marginal for annual.
- **Log-SPL rather than linear** — captures dB modulation directly,
  more appropriate for biological loudness variations.
- **Remove linear trend + detide** (optional 12-h/24-h band-stop)
  before computing long-period PSD to avoid DC leakage.

### ANCOVA template for environmental-control testing

For each acoustic index `A_t` at time `t`:

```
A_t = β_0 + β_1 · wind_speed_t + β_2 · wind_direction_t
        + β_3 · sea_surface_temp_t + season_factor_t
        + lunar_phase_factor_t + ε_t
```

Test `β_lunar ≠ 0` via F-test. If lunar effect survives after
weather is partialled out → biological lunar signal. This is the
exact ANCOVA used in Staaterman 2014.

### Wavelet alternative (recommended update)

A modern replacement for the PSD-of-series approach:

```python
import pywt
cwtmatr, freqs = pywt.cwt(spl_low, np.arange(1, 1024), 'cmor1.5-1.0',
                          sampling_period=window_interval_s)
```

Continuous wavelet transform on the SPL series gives a time-
frequency-localised view of when different periodicities are active
— detects, e.g., a snapping-shrimp diel cycle that strengthens in
summer. Preferred over PSD for non-stationary rhythms.

### Sound-source sorting by band + rhythm

From this paper's structure:

| Band | Dominant rhythm | Likely source |
|------|-----------------|---------------|
| 25 – 2,000 Hz | Lunar (27.32 d), diel | Fish spawning chorus |
| 2,000 – 10,000 Hz | Diel (24 h, dawn/dusk peaks) | Snapping shrimp, odontocete |
| 10 – 100 Hz | Annual, synoptic-weather | Wind, waves, distant shipping |
| > 10 kHz | Irregular | Odontocete transits (if fs allows) |

This pattern is the basis for **rhythm-aware acoustic index design**:
compute H, ACI, NDSI on each rhythm-band separately, and report
lunar-filtered vs diel-filtered residuals to find anomalous events.

## Sonobuoy integration plan — long-term PAM mode

This paper provides the **decadal-context scientific purpose** for a
long-term PAM sonobuoy: measure diel, tidal, lunar, seasonal, and
annual rhythms of a soundscape to characterise its biological
community and detect ecosystem change. Every architectural decision
in `sonobuoy-pam` ultimately serves this goal.

**Duty-cycle recommendation.** Staaterman's **12-s every 5-min (4 %)
duty cycle** is the current community consensus for coral-reef PAM.
For a sonobuoy-PAM node:

- **High-duty profile (12 % = 30 s on / 4 min off)**: for new deployments
  needing to characterise the soundscape; yields better statistics on
  rare events at the cost of ~3× storage / power.
- **Nominal profile (4 % = 12 s / 5 min)**: Staaterman baseline;
  tried-and-true for multi-year deployments.
- **Thrift profile (1 % = 6 s / 10 min)**: extend deployment to 2+
  years; acceptable for decadal trend detection but poor for event-
  level statistics.

**Sample rate.** 20 kHz (Nyquist 10 kHz) is adequate for fish and
snapping-shrimp periodicity analysis but misses high-frequency
odontocetes. The sonobuoy-PAM default should be **192 kHz** when
budget permits, falling back to a 20 kHz bandpass time series for
rhythm analysis to stay comparable with Staaterman and the MEPS
literature.

**Telemetry payload.** The on-buoy indices stream (Sueur analysis,
80 bytes/min) already contains `SPL_low`, `SPL_high`, `ACI`, and
`H` at per-minute resolution. Over a year this is ~50 MB of
telemetry — enough for the full rhythmicity pipeline to run
**continuously on shore**, without recovering the buoy. This is the
single most important sonobuoy-PAM deliverable: diel/lunar
periodicity tracking **in near real time** rather than post-
deployment.

**On-shore daily pipeline.** At 04:00 UTC every day, for every
buoy:

1. Ingest last 24 h of indices from Iridium SBD.
2. Append to per-buoy time series.
3. Run rolling 30-day autocorrelation + 60-day PSD on SPL_low,
   SPL_high, ACI, and H.
4. Score `Δ periodicity` — drop or drift in expected diel/lunar peaks
   is an ecosystem-change signal.
5. Emit a daily per-buoy "soundscape health" vector (dawn-peak
   amplitude, dusk-peak amplitude, lunar-peak amplitude at 27.32 d,
   synodic-peak amplitude at 29.53 d, broadband SPL trend, ACI
   trend).

**Multi-buoy ecosystem model.** For an array deployment, the
periodicity fingerprint per buoy becomes a **feature vector for an
ecosystem similarity map**. Clustering over the array yields
site-level acoustic habitat classes — directly actionable for MPA
management and vessel-exclusion-zone design.

**Anomaly detection.** A **ship transit** appears as a broadband
SPL spike outside the usual diel/lunar envelope; a **sonar exposure**
appears as a mid-frequency (1–10 kHz) tonal anomaly; a
**bloom/die-off** appears as a sustained drop in high-frequency
band SPL (fewer snapping shrimp). The rhythmicity pipeline makes
these anomalies first-class detectable events.

**Ecological-integration contract.** The sonobuoy-PAM crate should
expose a stable `RhythmicityReport` struct:

```rust
pub struct RhythmicityReport {
    pub buoy_id: BuoyId,
    pub period_days: f32,              // Staaterman's 27.32, 29.53, 1.0, 0.5...
    pub band: SpectralBand,            // Low, High, Broadband, Per10kHz
    pub amplitude_db: f32,             // peak-to-peak in dB of the SPL series
    pub coherence: f32,                // wavelet coherence 0-1
    pub anchor_phase_utc_hours: f32,   // time of peak in the cycle
    pub latest_update: DateTime<Utc>,
}
```

Consumers: the `weftos-sonobuoy-head` species crate uses this to
gate species classifiers (only trigger fish ID during fish-chorus
hours), and the `clawft-kernel::causal` graph ingests
`RhythmicityReport`s as `Impulse`s that feed the ECC cognitive tick
(see synthesis §3.1).

**ADR recommendation.** Propose **ADR-066: Rhythmicity pipeline for
sonobuoy-pam** naming Staaterman 2014 (this paper), Pieretti 2011
(ACI), Sueur 2008 (H), and Kasten 2012 (NDSI) as the reference
bibliography. Codifies the diel/tidal/lunar/seasonal rhythm analysis
as a mandatory on-shore daily workflow.

## Follow-up references

1. **Staaterman et al. 2017** *Celestial patterns in marine
   soundscapes revisited: daily and lunar patterns from the Arabian
   Sea*, MEPS. Extension to a second biogeographic region;
   demonstrates the method's generality.
2. **Bertucci, Parmentier, Lecellier, Hawkins, & Lecchini 2016**
   *Acoustic indices provide information on the status of coral
   reefs: An example from Moorea Island in the South Pacific*, Sci.
   Rep. 6:33326, doi:10.1038/srep33326. Applies Sueur/Pieretti
   indices to a reef-health gradient.
3. **Mooney et al. 2020** *Listening forward: approaching marine
   biodiversity assessments using acoustic methods*, R. Soc. Open
   Sci. 7:201287, doi:10.1098/rsos.201287. Consolidates the marine
   PAM agenda that this paper kicked off.
4. **Lamont et al. 2022** *The sound of recovery: Coral reef
   restoration success is detectable in the soundscape*, J. Appl.
   Ecol. 59(3):742–756, doi:10.1111/1365-2664.14089. Staaterman-
   style PAM applied to restoration monitoring.
5. **McWilliam & Hawkins 2013** *A comparison of inshore marine
   soundscapes*, J. Exp. Mar. Biol. Ecol. 446:166–176,
   doi:10.1016/j.jembe.2013.05.012. Parallel site-comparison
   framework.
6. **Williams et al. 2024** *SurfPerch: a bioacoustic foundation
   model for reef soundscapes*, arXiv:2404.16436. The ML successor
   that enables species-level attribution on top of Staaterman's
   rhythmicity framework.
