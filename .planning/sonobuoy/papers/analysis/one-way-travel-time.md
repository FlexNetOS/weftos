# Webster, Eustice, Singh, Whitcomb 2012 — Single-Beacon One-Way Travel Time Acoustic Navigation

## Citation

### Primary (2012 journal paper)

- **Authors**: Sarah E. Webster, Ryan M. Eustice, Hanumant Singh,
  Louis L. Whitcomb
- **Title**: "Advances in single-beacon one-way-travel-time acoustic
  navigation for underwater vehicles"
- **Venue**: *International Journal of Robotics Research*, Vol. 31,
  No. 8 (July 2012), pp. 935–950
- **DOI**: https://doi.org/10.1177/0278364912446166
- **Open PDF (UMich Eustice lab)**:
  http://robots.engin.umich.edu/publications/swebster-2012a.pdf
- **Downloaded**: `.planning/sonobuoy/papers/pdfs/one-way-travel-time.pdf`

### Earlier companion (2011 JFR)

- **Authors**: Ryan M. Eustice, Louis L. Whitcomb, Hanumant Singh,
  Matthew Grund
- **Title**: "Synchronous-clock one-way-travel-time acoustic
  navigation for underwater vehicles"
- **Venue**: *Journal of Field Robotics*, Vol. 28, No. 1 (2011),
  pp. 121–136
- **DOI**: https://doi.org/10.1002/rob.20364
- **Open PDF**:
  http://robots.engin.umich.edu/publications/reustice-2011a.pdf

## Status

**Verified.** The 2012 IJRR paper is indexed at SAGE Publishing under
DOI 10.1177/0278364912446166 (vol 31, issue 8, pp. 935-950) and is
mirrored open-access on Eustice's lab page at the University of
Michigan. The 2011 JFR companion is verified via DOI 10.1002/rob.20364.
The 735 KB PDF was successfully downloaded. All four authors — Webster
(UW APL), Eustice (UMich), Singh (WHOI/Northeastern), Whitcomb (JHU) —
are canonical underwater-navigation researchers with dozens of
follow-up papers on OWTT.

## Historical context

Classical LBL (Hunt-1974) requires a **round-trip** acoustic
interrogation: vehicle pings, transponder replies, vehicle measures
RTT. This imposes two costs: (a) update rate is halved by the
reply wait (a 5 km range → ~7 s cycle), and (b) the approach does
not scale — `N` vehicles in one field interfere with each other's
interrogations. By 2007 WHOI/UMich/JHU realized that if vehicle
and beacon share a **synchronous clock** (~1 µs precision), the
beacon can simply broadcast its own identity and timestamp on a
scheduled TDMA slot, and any listening vehicle computes

    R = c̄ · (t_receive - t_transmit)

from the one-way travel time. No reply is required. This is
**OWTT navigation** — one broadcast serves an arbitrary number of
listeners, the update rate is the full acoustic-cycle speed, and
the same RF frame can carry the beacon's own position. Eustice-2007
gave the deep-water demonstration (4000 m, Mid-Atlantic Ridge);
Eustice-2011 formalized the EKF framework; Webster-2012 generalized
to a **single moving beacon** (surface ship, not moored transponder)
and extended the theoretical observability analysis.

For the sonobuoy project, OWTT is the preferred scheme over LBL
because (a) buoys broadcast rather than interrogate, so they can
use TDMA scheduling; (b) ping collision is trivially avoided; (c)
the same broadcast carries each buoy's GPS fix, enabling joint
GPS-drift + acoustic-ranging fusion; and (d) bandwidth-efficient —
`N` buoys need `N` TDMA slots per cycle, not `N²` interrogations.

## Core content

### The synchronous-clock primitive

Each node has a clock disciplined to a common time base. The
reference grade in Eustice-2011: Seascan SISMTB oven-controlled
crystal oscillator (OCXO), Allan deviation `σ_y(τ=1 s) ≈ 10⁻¹¹`,
drift ~5 µs/hr (corresponds to ~7.5 mm/hr equivalent range drift).
Nodes are synchronized at deploy time via GPS (sub-µs at surface)
and drift open-loop thereafter; periodic re-sync is performed
when the buoy surfaces or when a clean OWTT measurement triangulates
the clock offset.

### The OWTT packet

Each broadcast carries
```
  [transmit_id | transmit_time_usec | gps_position | gps_time]
```
at a known TDMA slot. Any listener at time `t_rx` computes

    R = c̄ · (t_rx - transmit_time)

(adjusted for any known internal-delay offsets of both transmit
and receive hardware).

### The EKF for OWTT

The vehicle's state is augmented with the clock bias:

    x = (p, v, c_bias, c_drift)           dim ≥ 8

Observation of an OWTT range `R_i` from beacon `i` at position
`q_i(t_tx)` gives

    z_i = ||p(t_rx) - q_i(t_tx)|| + c̄ · c_bias - measurement noise

The Jacobian has a straightforward form; Webster-2012 Table 2
reports conditional observability: with one moving beacon, `p` is
observable if the beacon's trajectory is not collinear with the
vehicle's velocity. The paper gives the full theoretical analysis
and simulation verification.

### Measurement accuracy (Webster-2012 Sec. 6 + Eustice-2011 Sec. 5)

From field trials:
- **OWTT measurement noise**: ~0.1 ms RMS (after compensating for
  internal delays) → ~15 cm equivalent range noise at 1500 m/s.
- **Clock drift over 1 hr, uncorrected**: 5-10 µs → ~1-2 cm
  equivalent range drift.
- **Position accuracy, single-beacon moving, 1 hr survey**: ~2-5 m
  RMS (limited by beacon-GPS noise, not acoustic ranging itself).
- **Position accuracy, multi-beacon LBL-equivalent**: 1-2 m RMS.
- **Usable range**: up to 10 km line-of-sight at 9 kHz.
- **Ping rate achievable**: TDMA with 4-16 beacons, each gets
  0.1-1 Hz update; total channel utilization 50-100%.

### WHOI Micro-Modem specifics (the reference hardware)

- **Carrier**: 9 kHz or 25 kHz
- **Waveform**: LFM chirp (500 Hz - 4 kHz sweep) for timing, PSK for
  data payload
- **Matched-filter timing resolution**: ~50 µs (cross-correlation
  of received chirp with reference)
- **Packet payload**: 32 bytes (Mini), 256 bytes (Rate-0)
- **Acoustic TX power**: ~190 dB re 1 µPa @ 1 m
- **Electrical TX power**: ~20 W peak, 0.5 ms duration → ~10 mJ per
  ping

This hardware is the "micro-modem" referenced throughout the
underwater-robotics literature; it is directly usable for
sonobuoy-class nodes.

## Portable details — the OWTT protocol and clock model

### Clock discipline model (Eustice-2011 §3)

Local clock: `t_local(t) = t + bias(t)` with `bias` modeled as
integrated Gaussian random walk:

    ḃ(t) = d(t)                     clock drift rate
    ḋ(t) = η(t),   η ~ N(0, q_d)    random-walk acceleration

For a good OCXO: `q_d ≈ (10⁻¹¹)² / s`. Over 1 hr uncompensated
drift has variance ~(10⁻¹¹ · 3600)² ≈ (36 ns)² — i.e. negligible.
Over 12 hr it's ~1 µs. The buoy must either re-sync at surface
(GPS pulse-per-second) every ~6-12 hr, or carry a chip-scale
atomic clock (CSAC, ~150 µW, $2-5k) with `σ_y(τ) ≈ 10⁻¹¹` drift
stability.

### Rust skeleton

```rust
/// One OWTT broadcast packet (per JANUS-compatible encoding or
/// WHOI Micro-Modem Rate-0).
#[derive(Debug, Clone)]
pub struct OwttBroadcast {
    pub node_id: u16,
    pub transmit_time_us: u64,     // monotonic since shared epoch
    pub gps_position: Option<[f64; 3]>,
    pub gps_time_us: Option<u64>,
    pub clock_bias_est_us: f32,
    pub internal_delay_us: u16,
}

/// Incoming OWTT measurement at a listening buoy.
#[derive(Debug, Clone, Copy)]
pub struct OwttMeasurement {
    pub peer_id: u16,
    pub peer_position: [f64; 3],
    pub tx_time_us: u64,
    pub rx_time_us: u64,
    pub c_bar_mps: f64,
    pub timing_sigma_us: f32,
}

impl OwttMeasurement {
    pub fn range(&self) -> (f64, f64) {
        let dt = (self.rx_time_us - self.tx_time_us) as f64 * 1e-6;
        let r = self.c_bar_mps * dt;
        let sigma = self.c_bar_mps * (self.timing_sigma_us as f64) * 1e-6;
        (r, sigma)
    }
}

/// Extended Kalman filter state. Per-buoy.
pub struct OwttEKF {
    pub position_m: [f64; 3],
    pub velocity_mps: [f64; 3],
    pub clock_bias_s: f64,
    pub clock_drift: f64,
    pub cov: [[f64; 8]; 8],
}

impl OwttEKF {
    pub fn propagate(&mut self, dt: f64) { /* random walk + IMU */ }
    pub fn update_owtt(&mut self, m: &OwttMeasurement) { /* EKF step */ }
    pub fn gps_update(&mut self, p_gps: [f64; 3], sigma_gps: f64) { /* resync */ }
}
```

### TDMA scheduling

With `N` buoys sharing one channel and acoustic cycle `T_cyc` (~100
ms minimum at 10 kHz):

    slot duration = T_cyc + max_range / c̄ ≈ 0.1-4 s
    epoch = N · slot duration

For N=8 buoys at 5 km max range, epoch ≈ 4 × 8 = 32 s, per-buoy
update rate ~0.03 Hz. For tighter spacing (500 m, N=4), epoch ~2
s, ~0.5 Hz per buoy — the regime OWTT is designed for.

## Integration with the sonobuoy stack

OWTT is the **preferred ranging protocol** for a drifting sonobuoy
field (vs round-trip LBL per Hunt-1974). Three reasons align with
the K-STEMIT-extended architecture: (1) TDMA scales linearly not
quadratically with buoy count — important because the spatial branch
(Tzirakis-2021 GCN + Grinstein-2023 Relation-Network) wants ≥8
buoys for good adjacency; (2) each broadcast packet carries the
buoy's GPS fix natively, enabling the sensor-position gap flagged in
SYNTHESIS.md §10 to be closed with the same packet that gives the
range; (3) chip-scale atomic clocks at ~150 µW are compatible with
the 4-tier power budget (SYNTHESIS.md §3, ADR-069) — the Tier-2
~5 mW Cortex-M4 can host a CSAC without violating the 5 mW
envelope. The OwttBroadcast packet above extends WeftOS's
existing `Impulse` queue (range-change events fire when `σ_{ij}(t)`
exceeds threshold); the OwttEKF runs as a Tier-3 periodic task at
0.1-1 Hz. The distance matrix `D(t)` and its covariance are
published to `clawft-sonobuoy-spatial` which feeds Tzirakis' dynamic
adjacency directly. This replaces ADR-053-era assumption of
haversine-on-GPS and provides the meter-scale adjacency that the
Grinstein Relation-Network consumes as a metadata feature.

## Strengths

1. **Scales to many buoys** — TDMA slots mean `N` buoys share one
   channel at the cost of `1/N` update rate per buoy. LBL scales
   badly past ~4 vehicles.
2. **Bandwidth-efficient** — one broadcast per TDMA slot, not a
   query/reply pair.
3. **Theoretical observability established** — Webster-2012's
   single-moving-beacon observability analysis applies directly to
   a drifting-buoy field where some buoys are occasional GPS
   references.
4. **Commercial hardware available** — WHOI Micro-Modem
   (µModem-2), Teledyne Benthos, EvoLogics S2CR. Acoustic TX,
   matched-filter RX, and MAC are solved problems.
5. **Packet carries metadata** — GPS fix, clock bias estimate,
   health state all ride the same broadcast. The ranging channel
   becomes the telemetry channel.

## Limitations

1. **Requires synchronized clocks** — CSAC or equivalent, ~1 µs
   precision. Cheap OCXOs drift too fast (~1 ms per day) and
   require frequent resync.
2. **Observability needs geometry** — single-beacon case needs
   beacon trajectory non-collinear with vehicle velocity. With ≥3
   buoys this is automatic; with 2 it is a design constraint.
3. **No bidirectional residuals for OAT** — single OWTT per pair
   per epoch gives one travel-time measurement, so SSP estimation
   (Munk-Wunsch 1979) requires multiple pairs and multiple paths.
   Reciprocal transmissions (A→B and B→A) give the best SSP
   + current estimates; pure OWTT only gets SSP.
4. **Multipath still a problem** — matched-filter timing resolution
   is ~50 µs against direct arrival only. Surface/bottom bounces
   add ambiguity; must be resolved by arrival-angle gating or by
   physics-prior rejection.
5. **Vulnerable to clock attacks / spoofing** — any adversary
   broadcasting a forged timestamp can inject arbitrary range
   errors. The WeftOS `rvf-crypto` layer handles identity/signing
   at the packet level.

## Follow-up references

1. **Eustice, R.M., Whitcomb, L.L., Singh, H., Grund, M.** (2007).
   "Experimental Results in Synchronous-Clock One-Way-Travel-Time
   Acoustic Navigation for Autonomous Underwater Vehicles." Proc.
   Robotics Science and Systems. The first experimental
   demonstration.
2. **Eustice, R.M., Whitcomb, L.L., Singh, H., Grund, M.** (2011).
   "Synchronous-clock one-way-travel-time acoustic navigation for
   underwater vehicles." *J. Field Robotics* 28(1):121. DOI:
   10.1002/rob.20364. The formal EKF formulation.
3. **Claus, B., Kepper, J.H., Suman, S., Kinsey, J.C.** (2018).
   "Closed-loop one-way-travel-time navigation using low-grade
   odometry for autonomous underwater vehicles." *J. Field Robotics*
   35(4):504. DOI: 10.1002/rob.21746. OWTT with cheap hardware —
   directly relevant to sonobuoy cost envelope.
4. **Freitag, L., Grund, M., Singh, S., Partan, J., Koski, P.,
   Ball, K.** (2005). "The WHOI Micro-Modem: an Acoustic
   Communications and Navigation System for Multiple Platforms."
   Proc. MTS/IEEE OCEANS 2005. Hardware reference for Eustice-2011
   and Webster-2012.
5. **Fallon, M.F., Papadopoulos, G., Leonard, J.J.** (2010). "A
   Measurement Distribution Framework for Cooperative Navigation
   using Multiple AUVs." IEEE ICRA 2010. Extends OWTT to
   cooperative multi-vehicle geometry.
