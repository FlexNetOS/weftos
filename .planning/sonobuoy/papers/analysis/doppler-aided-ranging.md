# Stojanovic 2008 + D-Sync / LT-Sync — Doppler-Aided Acoustic Ranging

## Citation

### Foundational (Doppler shift in UWA communications)

- **Authors**: Milica Stojanovic, James Preisig
- **Title**: "Underwater Acoustic Communication Channels: Propagation
  Models and Statistical Characterization"
- **Venue**: *IEEE Communications Magazine*, Vol. 47, No. 1 (January
  2009), pp. 84-89
- **DOI**: https://doi.org/10.1109/MCOM.2009.4752682

### Primary (Doppler-enhanced time sync; inter-node relative velocity)

- **Authors**: Feng Lu, Diba Mirza, Curt Schurgers
- **Title**: "D-Sync: Doppler-based Time Synchronization for Mobile
  Underwater Sensor Networks"
- **Venue**: *Sensors* 18(6):1854 (June 2018)
- **DOI**: https://doi.org/10.3390/s18061854
- **Open**: https://www.mdpi.com/1424-8220/18/6/1854 (CC BY 4.0)

### Modern (joint time / velocity / range)

- **Authors**: Junyu Dong, Hongli Gao, Xinxin Wang, et al.
- **Title**: "DE-Sync: A Doppler-Enhanced Time Synchronization for
  Mobile Underwater Sensor Networks"
- **Venue**: *Sensors* 18(6):1861 (June 2018)
- **DOI**: https://doi.org/10.3390/s18061861
- **Open**: https://pmc.ncbi.nlm.nih.gov/articles/PMC6021945/

### Companion (Doppler-estimation signal-processing methods)

- **Walree, P.A. van** (2013). "Propagation and scattering effects
  in underwater acoustic communication channels." *IEEE JOE*
  38(4):614. DOI: 10.1109/JOE.2013.2278913.

## Status

**Verified.** Stojanovic-Preisig 2009 is the canonical UWA channel
reference; DOI 10.1109/MCOM.2009.4752682 (IEEE Comm Mag). D-Sync 2018
is verified via MDPI Sensors DOI 10.3390/s18061854 (CC BY 4.0; PMC
mirror at PMC6028950). DE-Sync is verified via MDPI DOI
10.3390/s18061861 (PMC6021945, CC BY 4.0). Van Walree 2013 is
verified via IEEE JOE DOI 10.1109/JOE.2013.2278913. These are the
standard references for Doppler-aided ranging and clock sync in
underwater networks.

## Historical context

Any acoustic communication between moving nodes in the ocean sees
Doppler shift — the carrier-frequency offset proportional to
relative radial velocity. For a typical 10 kHz carrier and 1 m/s
relative speed, the shift is `f_D = f_c · v / c ≈ 6.7 Hz`, large
compared to the ~1 Hz matched-filter bin width. For decades this
was treated as a **problem** to be compensated before demodulation
(Stojanovic-Preisig 2009 survey).

The 2010s realized Doppler shift carries **information, not just
noise**: it directly measures the radial relative velocity of the
transmitting node. D-Sync (Lu-Mirza-Schurgers 2018) and DE-Sync
(Dong-Gao-Wang 2018) use this to correct time-synchronization
estimates for mobile underwater sensor networks. The natural
extension — which the sonobuoy-ranging literature has not yet
written — is to use Doppler shift from every ping to track
**inter-buoy radial velocity** and therefore drift, closing the
gap between snapshot positions and a full `(p, v)` state estimate.

## Core content

### The Doppler-shift observable

For a narrowband carrier `f_c` transmitted from node A and received
at node B with relative radial velocity `v_AB = (p_B - p_A) · v_rel
/ ||p_B - p_A||`, the received frequency is

    f_rx ≈ f_c · (1 - v_AB / c)

or equivalently the normalized Doppler shift

    f_D = (f_rx - f_c) / f_c ≈ -v_AB / c

is directly the scaled radial-velocity projection.

**Measurement via a standard acoustic modem**: every chirp-
preamble-based receiver already estimates Doppler scaling to
compensate matched-filter bandwidth expansion (otherwise `c` in
`f_D / f_c ≈ v/c` multiplies the chirp duration, warping the
template). This scaling factor is **the Doppler measurement** —
free byproduct of demodulation, no extra hardware.

### Typical numbers

- **Doppler shift**: 6.7 Hz per 1 m/s at 10 kHz carrier
- **Matched-filter scaling-factor resolution**: ~0.01 m/s for a
  ~100 ms chirp with ~1 kHz bandwidth and ~20 dB SNR
- **Drifting buoy relative speed**: 0.1-2 m/s typical (swell,
  current, wind)
- **Scale-factor estimation bandwidth**: narrowband assumption
  breaks above ~5 m/s relative speed

### D-Sync integration (Lu 2018)

The D-Sync protocol combines acoustic packets carrying
`{tx_time, tx_position}` with per-packet Doppler estimates into a
joint linear regression that solves clock skew, clock offset, and
relative velocity simultaneously:

    t_rx,k = (1 + α) · (t_tx,k + d_k) + β
    d_k = d_0 + v_rel · Δt_k / c
    f_D,k = -v_rel / c

Stacking `N` packets gives a joint linear system over `(α, β, d_0,
v_rel)`. Reported accuracies:

- Clock skew: ~1 ppm after 30 packets at 10 s cadence
- Clock offset: ~1 ms at 30-packet convergence
- Relative velocity: ~0.1 m/s RMS
- Position drift rate implied by v_rel: ~5 cm/s RMS

### DE-Sync addition (Dong 2018)

DE-Sync extends by explicitly modeling clock-skew's effect on
Doppler estimation (skew creates a false Doppler bias proportional
to `α · c`). Reports another ~2× improvement in v_rel estimation.

### Van Walree 2013 — signal-processing methods

Describes multiple methods for extracting Doppler:
1. **Correlator-bank**: matched-filter with multiple time-scaled
   templates; peak correlation picks the scaling. ~0.1% precision.
2. **Chirp-rate observer**: on an LFM chirp, Doppler maps to an
   apparent chirp-rate change that can be estimated post-match.
3. **Carrier PLL**: for PSK packets, a phase-locked loop tracks
   the carrier phase residual, outputting Doppler shift as a time
   derivative.

Method 1 is the most common in commercial modems; method 2 is used
in µModem and JANUS; method 3 requires PSK (not FSK, so not vanilla
JANUS). Accuracy scales as `~1 / (SNR × T_pulse × B)`.

## Portable details — joint range-and-velocity tracking

### The augmented EKF state

For each buoy, track not just position but also velocity:

    x_i = (p_i, v_i, δ_i, d_δ_i)       dim 8

where `(p_i, v_i) ∈ R^6` is kinematic state, `δ_i` is clock bias,
`d_δ_i` is clock drift. The observation model per packet
`(i → j)` now provides **two** measurements:

    y_R = c̄ · (t_rx - t_tx) + ε_R           (range, as OWTT)
    y_D = -f_D,ij / f_c · c̄                 (radial velocity)

with Jacobians

    ∂y_R / ∂p_i = -û,   ∂y_R / ∂p_j = +û
    ∂y_D / ∂v_i = -û,   ∂y_D / ∂v_j = +û

where `û` is the unit vector from `i` to `j`. Both measurements
share the same unit vector — the range constrains position,
velocity constrains drift. The coupled update is stable even in
near-collinear geometry because range- and velocity- Jacobians have
complementary null spaces.

### Why this matters for drifting buoys

- A drifting buoy field has ~1 m/s differential drift; without
  velocity tracking, a position update delayed by 10 s accumulates
  ~10 m of unmodelled drift error.
- With Doppler-derived velocity tracking at 0.1 m/s precision, the
  buoy field maintains positional self-consistency between GPS
  updates.
- Joint range + velocity measurement halves the number of TDMA
  slots needed for a given target accuracy (same-packet
  observation of both).

### Rust skeleton

```rust
/// Packet with embedded range-and-velocity measurement.
#[derive(Debug, Clone, Copy)]
pub struct DopplerRangeMeasurement {
    pub peer_id: u16,
    pub range_m: f64,
    pub range_sigma_m: f32,
    pub radial_velocity_mps: f64,
    pub velocity_sigma_mps: f32,
    pub t_rx_us: u64,
}

/// 8-dim EKF per buoy (p, v, δ, d_δ).
pub struct RangeVelocityEKF { /* ... */ }

impl RangeVelocityEKF {
    pub fn update(&mut self, peer: &BuoyState, m: &DopplerRangeMeasurement) {
        // Compute unit vector û = (peer.p - self.p) / ||.||
        // Residuals:   r_R = m.range_m - ||self.p - peer.p||
        //              r_D = m.radial_velocity_mps
        //                     - û·(self.v - peer.v)
        // Kalman update on (p, v, δ, d_δ).
        todo!()
    }
}
```

### SSP disambiguation

The Doppler observable has a `1/c` factor — so its estimate carries
a small SSP bias. For sonobuoy ranging at ~10 kHz carrier and
~1500 m/s, a 1 m/s c-error biases velocity by ~0.07%. Negligible
compared to 0.1 m/s Doppler precision. SSP does not disrupt
Doppler-aided ranging the way it affects pure OWTT ranging.

## Integration with the sonobuoy stack

Doppler-aided ranging is the **force-multiplier** that makes OWTT
scale cleanly to a drifting buoy field. Where OWTT gives position
snapshots at TDMA cadence (0.1-1 Hz), Doppler shift extracted from
the same matched-filter output gives per-pair radial velocity "for
free" — no additional bandwidth, no additional power. In the
K-STEMIT-extended architecture the 8-dim EKF state
`(p_i, v_i, δ_i, d_δ_i)` per buoy is the canonical ranging-
subsystem output; it supersedes the 3-dim position-only model and
is what the spatial branch (Tzirakis-2021 GCN, Grinstein-2023
Relation-Network) should consume for meaningful edge weights. The
velocity state also feeds the active-imaging branch (Kiang-2022
multistatic SAS) which assumes known platform velocities — SYNTHESIS
§10 explicitly flags this as a v4 open problem, and Doppler-aided
ranging closes it in v3. A new `eml_core::operators::doppler_fuse`
operator exposes the matched-filter scale factor as a trainable
observable with learnable bias (accounts for hardware-specific
filter asymmetry). The ranging-subsystem loop becomes: every TDMA
broadcast delivers `(range, radial_velocity, rx_timestamp)` to the
EKF; the EKF propagates and updates; velocity errors no longer
accumulate as position drift.

## Strengths

1. **Free measurement** — Doppler scaling is already computed for
   demodulation; repurposing it as a navigation observable is zero
   marginal power and zero marginal bandwidth.
2. **Complementary to range** — range constrains position, Doppler
   constrains velocity; same-packet joint update is strictly more
   informative than range alone.
3. **Robust to SSP bias** — `1/c` factor only, ~0.1% effect; SSP
   variation doesn't dominate.
4. **Standard-compliant** — JANUS chirp preamble is exactly the
   right waveform for Method 2 (chirp-rate observer) Doppler
   extraction.
5. **Velocity tracking enables SAS v4** — Kiang-2022 multistatic SAS
   requires known platform velocities; Doppler ranging supplies
   them natively.

## Limitations

1. **Narrowband assumption fails at high relative speed** — >5 m/s
   relative motion violates the narrowband-Doppler approximation;
   need full wideband Doppler expansion (time-scale factor).
   Typical drifting buoys stay well below this.
2. **Carrier PLL requires coherent modulation** — JANUS FSK doesn't
   support it; must use chirp-rate observer on the preamble or
   switch to PSK payload (e.g., JANUS+PSK extension). Reduces
   precision ~2-3×.
3. **Doppler-clock ambiguity** — velocity and clock-drift rate
   are observationally similar over short windows; long-baseline
   statistics (minutes) disentangle them, short-baseline inversions
   can confound them.
4. **Multipath-induced Doppler spreading** — direct + bounce
   arrivals at different angles have different Doppler; matched
   filter against direct-path template only. Multipath Doppler
   looks like noise.
5. **Vertical velocity unobservable for surface-only arrays** — a
   field of surface buoys measuring each other gets horizontal
   velocity only. z-direction needs a depth sensor (cheap) or
   reciprocal transmissions with known vertical separation.

## Follow-up references

1. **Stojanovic, M., Preisig, J.** (2009). "Underwater Acoustic
   Communication Channels: Propagation Models and Statistical
   Characterization." *IEEE Commun. Mag.* 47(1):84. DOI:
   10.1109/MCOM.2009.4752682. The canonical survey; Section III
   covers Doppler characterization.
2. **Walree, P.A. van** (2013). "Propagation and scattering effects
   in underwater acoustic communication channels." *IEEE JOE*
   38(4):614. DOI: 10.1109/JOE.2013.2278913. Signal-processing
   methods for Doppler extraction.
3. **Dong, J. et al.** (2018). "DE-Sync: A Doppler-Enhanced Time
   Synchronization for Mobile Underwater Sensor Networks."
   *Sensors* 18(6):1861. DOI: 10.3390/s18061861. Joint clock-skew
   + Doppler estimation.
4. **Sharif, B.S., Neasham, J., Hinton, O.R., Adams, A.E.** (2000).
   "A computationally efficient Doppler compensation system for
   underwater acoustic communications." *IEEE JOE* 25(1):52. DOI:
   10.1109/48.820735. Classical Doppler compensation algorithm.
5. **Aparicio, J. et al.** (2011). "Underwater acoustic positioning
   system based on Doppler shift measurements." IEEE OCEANS 2011.
   Doppler-only range-rate positioning; earliest explicit use of
   Doppler as a navigation observable.
