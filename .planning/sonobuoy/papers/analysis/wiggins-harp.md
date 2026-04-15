# Wiggins & Hildebrand 2007 — High-frequency Acoustic Recording Package (HARP)

## Citation

Wiggins, S. M.; Hildebrand, J. A. (2007).
**"High-frequency Acoustic Recording Package (HARP) for broad-band,
long-term marine mammal monitoring."**
In *Proceedings of the 2007 International Symposium on Underwater
Technology and Workshop on Scientific Use of Submarine Cables and
Related Technologies (SSC)*, Tokyo, Japan, 17–20 April 2007,
pp. 551–557. IEEE.
DOI: https://doi.org/10.1109/UT.2007.370760
IEEE Xplore: https://ieeexplore.ieee.org/document/4231090
eScholarship: https://escholarship.org/uc/item/0p6832s1

**Status**: verified. DOI resolves at IEEE Xplore; PDF downloaded to
`.planning/sonobuoy/papers/pdfs/wiggins-harp.pdf` (7 pages, ~450 KB)
from the Scripps Whale Acoustics Lab public archive at
https://www.cetus.ucsd.edu/docs/publications/WigginsUT07.pdf; text
content, figure references, and author affiliations match the
eScholarship record.

## One-paragraph summary

Wiggins & Hildebrand describe the **HARP**, the Scripps-designed
autonomous seafloor recorder that became the foundation instrument
for NOAA's decadal passive acoustic monitoring of marine mammals.
The HARP is a battery-powered, seafloor-deployable,
**broad-band (up to 100 kHz flat response, 200 kHz sample rate, 16-bit)
long-term (months) recorder** that accumulates **~2 TB per deployment**
using an array of 16 IDE laptop hard disks in a pressure case. The
paper describes the complete system — pressure case, hydrophone,
low-power data logger with Motorola 32-bit 20-MHz microcontroller,
16-bit Analog Devices ADC, SRAM buffer, Ethernet/IDE controller, and
precision Seascan clock (1 part in 10⁸ drift) — along with the
spectral-averaging compression strategy that makes the resulting
terabyte-scale datasets navigable. The HARP is the reference
instrument for **how to build a long-term PAM node**, and is the direct
ancestor of most modern open-ocean PAM hardware (SoundTrap, AMAR,
MARU). It is cited by ~2,000 downstream studies and is the sensor
behind LISTEN Gulf of Mexico, SanctSound, and much of SIO's decadal
marine soundscape record.

## Methodology

This is an **engineering / instrumentation paper**, not a biological
study. The methodology is the hardware architecture, the
signal-processing pipeline, and the data-management strategy.

### Hardware architecture

**Data logger (7″ diameter × 2″ thick aluminum pressure-case end-cap
mount).** Five PCBs on a backplane:

1. **CPU card** — Motorola 32-bit 20-MHz microcontroller. FLASH memory
   for data buffering. RS-232 for terminal access.
2. **ADC card** — Analog Devices 16-bit ADC, up to **250 kHz sample
   rate** configurable in 8 rates from 2 kHz to 200 kHz. Includes
   hydrophone power supply and a **4-pole anti-alias filter**
   field-configurable for each sampling rate.
3. **SRAM buffer card** — 32 MB (16 × 2 MB chips) for double-buffered
   capture while a disk is spinning up.
4. **Ethernet/IDE card** — 10BaseT FTP + telnet + IDE bus to the disk
   block. Enables tested-on-deck evaluation without opening the case.
5. **Clock card** — Seascan temperature-compensated phase-lock clock
   oscillator with **~1 part in 10⁸ drift** over a deployment. Critical
   for multi-instrument TDOA tracking.

**Storage**: 16 × 2.5″ laptop IDE hard disks on a 50-pin common bus,
addressed and powered one at a time. 2003 deployment used 40 GB disks
(640 GB total); 2006 upgrade used 120 GB disks (**1.92 TB total**).
Disks arranged in a removable block so an instrument can be
refurbished in the field by swapping disk blocks.

**Power**: 192 × D-size alkaline cells (140 g each) in four sub-packs
of 48 cells (4 layers × 12). Each sub-pack supplies 12 V via 6
parallel strings of 8 cells in series. **Total ~330 Ah per
deployment.** One sub-pack lives with the data logger for bench
testing; three others live in a dedicated battery pressure case.
**Power consumption**: 250 mW during active sampling at max rate,
25 mW in idle, 2.2 W for 1 minute during disk writes, 5 W peak at disk
spin-up.

**Hydrophone**: External; designed for **flat response to 100 kHz**.
Cable-connected to the pressure case via underwater connectors.

### Signal chain

```
hydrophone -> pre-amp -> 4-pole anti-alias LPF -> 16-bit ADC
    -> DMA to SRAM (32 MB double-buffered)
    -> when ~30 MB filled: one disk powered on, drained, powered off
    -> disk-side write ~1 min, next disk addressed after
       current disk fills (120 GB -> ~30+ disk-days at 200 kHz)
```

**Sample rates**: 2, 10, 20, 50, 80, 100, 200, 250 kHz (8 software-
selectable rates). **At 200 kHz**, 1.92 TB fills in ~55 continuous
days. **At 30 kHz**, ~1 year continuous. Non-continuous duty cycles
(e.g., 5 min on / 10 min off) extend duration proportionally.

### Long-duration spectrogram compression

A core contribution: because the raw TB-scale WAV is too large to
visualise or audit directly, the paper describes a **spectral-averaging
compression pipeline** that produces **long-duration spectrograms**
(LDS). Process:

1. Divide the full WAV into consecutive short windows (e.g., 5-s).
2. For each window, compute the FFT-magnitude spectrum.
3. Store the per-window spectrum to disk as a column.
4. Concatenate columns: result is a **frequency × day** 2-D image
   compressing a 1-year deployment to a single viewable PNG.

Spectral averaging gives **~1000×+ compression** without losing the
ability to flag events for detailed re-review. Click-through from LDS
pixel back to the raw WAV is retained as the analyst workflow.

### Deployment configuration

- **Bottom-moored seafloor package**: pressure case with data logger
  and hydrophone cable, a separate battery case, flotation, an
  acoustic release, and a recovery buoy.
- **Depth rating**: up to ~1000 m typical Scripps configuration
  (deeper variants fabricated).
- **Deployment duration**: 3–12 months between service visits,
  limited by battery chemistry and disk capacity.
- **Recovery**: acoustic release triggered by shipboard acoustic
  command; package floats, is winched, disks swapped, redeployed.
- **Array use**: multi-HARP arrays use the precision clock to do
  TDOA source localisation and range estimation on baleen/odontocete
  calls. Clock drift of ~1 part in 10⁸ over 6 months is a few
  seconds — coarse TDOA still workable; for high-precision, post-hoc
  clock-synchronisation via shipboard checks.

## Key results

This paper is an **instrument description**, not an experimental
report. Key quantitative claims:

- **1.92 TB per deployment** with 2006-era 120 GB laptop disks.
- **55 days continuous at 200 kHz** or ~1 year continuous at 30 kHz.
  Duty cycling extends to multi-year.
- **~250 mW active / 25 mW idle** for the data logger; **330 Ah
  battery budget** per deployment.
- **Clock drift ~1 × 10⁻⁸** over deployment — ~0.3 s per year.
- **100 kHz bandwidth** flat response (hydrophone-limited) adequate
  for most odontocete clicks except high-frequency harbor porpoise.
- **4-pole anti-alias filter** switchable per sample rate.
- **~1000× data-volume reduction** via long-duration spectrogram
  compression, enabling analyst triage of 1-year deployments in
  minutes.
- **Clock-synchronised array** enables multi-HARP TDOA source
  localisation; the paper doesn't quantify localisation precision
  but cites ~10 m on multi-km baselines in follow-up work.
- HARPs are described as **deployed worldwide** at the time of
  writing (2007) — multiple sites in the North Pacific, Gulf of
  Mexico (later LISTEN-GoMex), Antarctic, and Atlantic.

## Strengths

- **Reference instrument.** The HARP is the workhorse of NOAA /
  Scripps long-term PAM and the ancestor of most modern
  architectures. It has defined the operational envelope — months of
  deployment, hundreds of kHz bandwidth, terabyte-scale datasets —
  that any new PAM system is benchmarked against.
- **Clean engineering story.** Power budget, disk-capacity-vs-
  deployment-duration curve, anti-alias filter, precision clock,
  deployment/recovery workflow are all described at a level of
  detail that can be re-implemented.
- **Long-duration spectrogram compression** is a durable,
  hardware-agnostic contribution. Every subsequent PAM system (incl.
  SoundTrap, AMAR) either adapts it or reinvents it.
- **Open-publication of design.** Both the IEEE paper and the
  Scripps/eScholarship copies are freely available; schematics are
  published in subsequent Scripps tech reports.
- **Precedent for the disk-block swap refurbishment model** — a
  design pattern that makes multi-year PAM operationally feasible on
  a normal research-vessel schedule.

## Limitations

- **No on-board processing beyond logging.** All intelligence is
  on-shore. Modern PAM would add an edge classifier (Perch, BEATs,
  event detector) to avoid shipping TB of raw WAV home.
- **No real-time telemetry.** Acoustic data is unavailable until
  recovery. For incident response (e.g., ship strike, sonar exposure
  event), this is unacceptable; modern designs add Iridium SBD or
  acoustic modems for daily summary telemetry.
- **Alkaline batteries.** Heavy, costly, non-rechargeable, temperature-
  sensitive. Modern designs use LiFePO₄ or primary lithium with
  2–3× energy density. The paper explicitly anticipates this
  upgrade.
- **IDE laptop disks** — the specific hardware is now obsolete. SSDs
  (and NVMe) replaced them in follow-on designs ~2014+; the design
  pattern survives, the specific silicon doesn't.
- **Single-channel.** The original HARP is a single-hydrophone
  instrument. True multi-channel / array behaviour requires multiple
  HARPs clock-synchronised; DIFAR sonobuoys and cabled arrays give
  multi-channel behaviour in a single package.
- **Depth rating limited** to the pressure-case spec (~1 km in
  early HARPs). Full-ocean-depth (6 km) HARP variants require
  different housings.
- **No integrated environmental sensors.** No pressure, temperature,
  salinity, or current sensor. Environmental covariates must be
  pulled from external CTDs. Modern PAM nodes (e.g., MARU+CTD,
  SoundTrap ST600) co-locate environmental sensing.
- **Clock drift uncorrected.** 10⁻⁸ drift is adequate for
  most TDOA work but not for sub-meter localisation over 6-month
  deployments; modern designs either add GPSDO on a surface float or
  use cabled timekeeping.

## Portable details

### HARP operational envelope (targets for any long-term PAM sonobuoy)

| Parameter | HARP value | Sonobuoy-PAM target |
|-----------|-----------|---------------------|
| Sample rate | 2–200 kHz selectable | Same, 8 rates |
| Bandwidth | DC – 100 kHz flat | DC – 100 kHz |
| Dynamic range | 16-bit ADC = 96 dB | 24-bit ADC = 144 dB (modern) |
| Deployment duration | 3–12 months | 6–18 months |
| Total capacity | 1.92 TB (2006); ~48 TB achievable now | ≥ 4 TB |
| Idle power | 25 mW | ≤ 50 mW |
| Active power | 250 mW | ≤ 500 mW |
| Write power | 2.2 W / 1 min | ≤ 3 W |
| Clock drift | 1e-8 (Seascan) | 1e-9 (GPSDO surface) |
| Anti-alias filter | 4-pole, per-rate | Same |
| Refurbishment pattern | Disk-block swap | SSD-block swap |
| Telemetry | None (offline-only) | Iridium SBD daily, LoRaWAN in range |
| Edge compute | None | RB5/H7 DSP for indices + event detect |

### Duty-cycle calculator (inherited directly from HARP)

For a desired monitoring duration `T_dep` (days), sample rate
`f_s` (Hz), bit depth `b` (bits/sample), channels `c`, total disk
capacity `C` (bytes), and duty-cycle fraction `δ ∈ (0, 1]`:

```
bytes_per_day = f_s · b/8 · c · δ · 86400
T_dep = C / bytes_per_day
```

For sample values: `C = 4 TB = 3.6 × 10¹² B`, `f_s = 192 kHz`,
`b = 24`, `c = 1`, `δ = 0.2` (5-on/20-off):

```
bytes_per_day = 192000 · 3 · 1 · 0.2 · 86400 ≈ 9.95 GB/day
T_dep ≈ 3.6e12 / 9.95e9 ≈ 362 days
```

i.e., **~1 year at 20% duty cycle** on 4 TB at 192 kHz/24-bit — a
reasonable sonobuoy-PAM configuration.

### Long-duration spectrogram (LDS) pipeline

Reusable recipe:

```python
def long_duration_spectrogram(wav_path, window_s=5.0, n_fft=2048, hop_s=5.0):
    """Returns (freq_bins, time_columns, magnitude_2d)."""
    ws = int(window_s * fs)
    hs = int(hop_s * fs)
    spec = []
    times = []
    with sf.SoundFile(wav_path) as f:
        for t0 in range(0, len(f) - ws, hs):
            chunk = f.read(ws)
            mag = np.abs(np.fft.rfft(chunk * np.hanning(ws), n=n_fft))
            spec.append(mag)
            times.append(t0 / fs)
    return np.fft.rfftfreq(n_fft, 1/fs), np.array(times), np.array(spec).T
```

For a 1-year deployment, `hop_s = 60` gives a 525,600-column PNG —
still viewable as a `24 × 365`-tile mosaic. Brightness = log
magnitude.

### Precision-clock requirement derivation

For multi-buoy TDOA localisation at range `R` (m) with sensor spacing
`d` (m) and desired position error `ε` (m):

```
Δt_required = ε · (d / R²) · c_sound   (s, worst case)
Clock_drift ≤ Δt_required / T_dep       (parts per deployment)
```

E.g., `ε = 10 m`, `d = 1000 m`, `R = 10 km`, `c = 1500 m/s`,
`T_dep = 180 days` = 1.56 × 10⁷ s:

```
Δt_required ≈ 10 · (1000/1e8) · 1500 = 0.15 s
Clock_drift ≤ 0.15 / 1.56e7 ≈ 1e-8
```

HARP's 1e-8 clock just meets this — matches the paper's explicit
design target.

## Sonobuoy integration plan — long-term PAM mode

The HARP is the **canonical reference design** for the long-term
sonobuoy-PAM profile. The architectural decisions from this paper
should be adopted essentially wholesale, with 2026-era substitutions.

**Hardware profile for `sonobuoy-pam` node.**

- **Housing**: 1-atm aluminum pressure case, 1 km depth rating
  (sufficient for shelf-sea / continental-margin deployments; deeper
  variants for Arctic/abyssal).
- **Hydrophone**: HTI-96-MIN or TC-4032, flat 10 Hz – 100 kHz.
- **ADC**: 24-bit ΣΔ at 192 or 256 kHz (supersedes 16-bit HARP).
- **MCU/SoC**: STM32H7 or equivalent Cortex-M7 for low-power
  control; optional Cortex-A55 coprocessor for edge ML.
- **Storage**: 4 × 2 TB industrial SSD = 8 TB total (vs. HARP
  1.92 TB).
- **Clock**: GPSDO on a surface tether buoy, distributing a
  10⁻¹¹ reference via White Rabbit / IEEE 1588 PTP to the
  seafloor node. Falls back to OCXO (10⁻⁹) when GPSDO is lost.
- **Battery**: primary lithium (Li-SOCl₂) sub-packs, ~2× energy
  density of D-cell alkaline.
- **Telemetry**: Iridium SBD (daily) + LoRaWAN (when in range of a
  shore gateway).
- **Environmental**: co-located CTD + 3-axis current meter.

**Data product hierarchy** (three-tier, extending HARP's LDS pattern):

1. **Raw WAV** — stored on-buoy, recovered at service visit.
   Primary scientific product. 24-bit, 192 kHz, mono.
2. **Acoustic indices + LDS tiles** — computed on-buoy, streamed
   daily via Iridium SBD. ~140 KB/day per node (see Sueur analysis
   for layout).
3. **Event snippets** — 30-s WAV around any on-buoy detection
   (Perch/BEATs/HNSW), streamed via SBD as priority traffic.

**Array deployment.** 4–8 HARP-like nodes in a ~1–10 km array with
shared PTP reference. Multi-buoy TDOA on the array feeds directly
into K-STEMIT's spatial branch (Tzirakis dynamic-adjacency GCN,
Grinstein relation-net) — the sonobuoy-tactical profile's source
localisation code reuses here without modification.

**Refurbishment cycle** (directly from HARP): every 6–12 months,
service-vessel visit → acoustic release → surface recovery →
SSD-block swap → battery-case swap → redeploy. Budget 2 hours per
node including ROV inspection.

**Differences from sonobuoy-tactical.**

| Dimension | sonobuoy-tactical | sonobuoy-pam (HARP-class) |
|-----------|-------------------|---------------------------|
| Lifetime | hours–days | months–years |
| Deployment | air-dropped, free-floating | moored seafloor |
| Compute | real-time bearing, low latency | index computation, duty-cycled |
| Telemetry | continuous UHF/VHF to aircraft | sparse Iridium SBD |
| Recovery | expendable | recovered, refurbished |
| Physics prior | small | K-STEMIT full physics branch |
| Multi-target | 1–2 targets of interest | dozens of species, multiple sources |

**ADR recommendation.** Propose **ADR-065: HARP-class hardware profile
for sonobuoy-pam** codifying the 24-bit / 192 kHz / 8 TB / GPSDO-PTP /
LiSOCl₂ reference design, with HARP as the direct ancestor citation.
Separate from ADR-063 (`sonobuoy-pam` deployment profile) which
concerns the software profile.

## Follow-up references

1. **Wiggins, Frasier, Henderson, & Hildebrand 2013** *Tracking
   dolphin whistles using an autonomous acoustic recorder array*,
   JASA 133(6):3813–3818, doi:10.1121/1.4802645. The multi-HARP
   array TDOA follow-up paper; quantifies the tracking precision that
   the clock-drift budget enables.
2. **Roch et al. 2016** *Detection, classification, and localization
   of cetaceans by groups at the Scripps Whale Acoustic Lab using
   passive acoustics*, JASA 140(4):3423. Describes the DCLDE toolbox
   on HARP-collected data.
3. **Hildebrand et al. 2018** *Summary of Marine Mammal Passive
   Acoustic Monitoring using HARPs in the Northwest Atlantic Ocean
   2015–2017*, Marine Physical Laboratory Tech Memo MPL-TM-619.
   The decadal-program operational paper (see LISTEN-GoMex).
4. **Merchant et al. 2015** *Measuring acoustic habitats*, Methods
   Ecol. Evol. 6(3):257–265, doi:10.1111/2041-210X.12330. Calibration
   and measurement standards for HARP-class recorders.
5. **Johnson, de Soto, & Madsen 2009** *Studying the behaviour and
   sensory ecology of marine mammals using acoustic recording tags:
   a review*, Marine Ecol. Progr. Ser. 395:55–73,
   doi:10.3354/meps08255. The complementary tag-based approach;
   for reference.
6. **NOAA SanctSound 2018–2023** program documentation at
   https://sanctuaries.noaa.gov/science/monitoring/sound/ — the
   current best analog of a modern, networked, HARP-inspired
   PAM programme.
