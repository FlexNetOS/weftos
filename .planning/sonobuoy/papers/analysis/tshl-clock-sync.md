# Syed & Heidemann 2006 — TSHL + Modern Underwater Clock-Sync Extensions

## Citation

### Primary (TSHL foundation)

- **Authors**: Affan A. Syed, John Heidemann
- **Title**: "Time Synchronization for High Latency Acoustic Networks"
- **Venue**: Proc. IEEE INFOCOM 2006, 25th IEEE International
  Conference on Computer Communications, Barcelona, April 2006,
  pp. 1-12
- **DOI**: https://doi.org/10.1109/INFOCOM.2006.161
- **Open PDF (USC/ISI)**:
  https://ant.isi.edu/~johnh/PAPERS/Syed06a.pdf
- **Downloaded**: `.planning/sonobuoy/papers/pdfs/tshl-clock-sync.pdf`
- **Tech-report companion**: USC/ISI Technical Report ISI-TR-2005-602.

### Modern extensions (mobile / Doppler-aware)

- **Lu, F., Mirza, D., Schurgers, C.** (2018). "D-Sync: Doppler-based
  Time Synchronization for Mobile Underwater Sensor Networks."
  *Sensors* 18(6):1854. DOI: 10.3390/s18061854.
- **Liu, J., Zhou, Z., Peng, Z., Cui, J.-H., Zuba, M., Fiondella, L.**
  (2013). "Mobi-Sync: Efficient Time Synchronization for Mobile
  Underwater Sensor Networks." *IEEE Trans. Parallel Distrib.
  Syst.* 24(2):406. DOI: 10.1109/TPDS.2012.164.
- **Wang, B. et al.** (2025). "LT-Sync: A Lightweight Time
  Synchronization Scheme for High-Speed Mobile Underwater Acoustic
  Sensor Networks." *J. Marine Sci. Eng.* 13(3):528. DOI:
  10.3390/jmse13030528.
- **Microsemi CSAC SA.45s datasheet**. Chip-scale atomic clock,
  ~150 µW, Allan deviation σ_y(τ=1s) ≈ 3×10⁻¹⁰.

## Status

**Verified.** Syed & Heidemann 2006 is at IEEE INFOCOM proceedings
under DOI 10.1109/INFOCOM.2006.161 (Barcelona, April 2006) and on
Heidemann's ISI publication page. Lu 2018 is verified via MDPI DOI
10.3390/s18061854 (open access CC BY). Liu 2013 is verified via IEEE
DOI 10.1109/TPDS.2012.164. Wang 2025 LT-Sync verified via MDPI DOI
10.3390/jmse13030528. PDFs for the Syed-2006 paper (252 KB) are
directly accessible. All references are primary sources, not memory-
compiled.

## Historical context

Wireless sensor networks (WSN) in 2000-2005 settled on PTP-like
protocols (RBS, TPSN, FTSP) assuming propagation delay is
negligible vs clock skew. Underwater acoustic networks violated this
assumption catastrophically: a 5 km link has ~3.3 s of propagation
delay, so the delay is **five orders of magnitude larger** than
typical clock skew over the same interval. Straight application of
PTP to underwater modems gave seconds of sync error.

TSHL (Time Synchronization for High Latency) was Syed and Heidemann's
INFOCOM-2006 answer: explicitly model propagation delay as a
first-class variable, separate the skew-estimation phase from the
offset-estimation phase, and handle them with separate message
patterns. TSHL is the canonical reference for every subsequent
underwater time-sync paper; it's listed as "foundational" in the
2018 Sensors survey (Zhang 2018) that classifies ~30 follow-on
protocols.

Modern extensions (Mobi-Sync 2013, D-Sync 2018, LT-Sync 2025) add
**mobility awareness** — TSHL assumes stationary nodes, which fails
for drifting sonobuoys. D-Sync's key innovation is to exploit
Doppler-shift measurements (already computed by the modem to
demodulate) to estimate propagation-delay *change*, closing the
gap to mobile nodes.

For the clawft sonobuoy field, clock sync is the critical enabling
technology for OWTT ranging (Webster-Eustice 2012). TSHL gives the
protocol-level architecture; the CSAC (chip-scale atomic clock)
gives the physical-layer clock with ~150 µW draw.

## Core content

### TSHL's two-phase protocol

Classical PTP exchanges `t1 → t2 → t3 → t4` and solves for skew and
offset jointly, assuming symmetric propagation delays. Underwater,
**the propagation delay itself is unknown and dwarfs the offset**,
so joint estimation is poorly conditioned. TSHL separates the two:

**Phase 1: skew estimation (one-way).** A reference node broadcasts
periodic beacons with timestamps `T_ref,1, T_ref,2, ..., T_ref,N`.
Receiver timestamps its local reception as `T_loc,1, ..., T_loc,N`.
Assuming unchanged propagation delay `d` and linear clock model
`T_loc = (1+α) · (T_ref + d) + β`, the receiver linearly regresses

    T_loc,k - T_loc,1 = (1+α) · (T_ref,k - T_ref,1) + noise

to estimate skew `α` **independently of d and β**. This is the key
insight: **skew can be learned from one-way broadcasts if the
propagation delay is constant**.

**Phase 2: offset estimation (two-way).** With skew known, a
standard PTP round-trip exchange solves `β` and `d` jointly using
the symmetric-delay assumption. Two-way exchange is needed exactly
once after many one-way beacons, so channel cost is low.

### Key numerical result (Syed-Heidemann 2006)

- Skew estimation converges to ~1 ppm RMS in 30-60 one-way beacons
  at 10-second cadence (~5-10 min total).
- Offset estimation reaches ~1 ms RMS after 5-10 two-way exchanges.
- Post-sync timing error drift: ~5-50 µs/hr depending on
  temperature stability.
- Bandwidth cost: ~1 packet per 10 s per node during initial
  sync, one packet per ~10 min steady state.

### Failure modes TSHL does not solve

1. **Moving nodes**: propagation delay changes during the skew-
   estimation phase, biasing the regression. This is the Mobi-Sync /
   D-Sync / LT-Sync patch.
2. **Multipath**: reflected arrivals alias into timing estimation.
   TSHL assumes matched-filter output selects the direct-path peak;
   broken in shallow water.
3. **Asymmetric delays**: if the two-way PTP round-trip sees
   different ray paths in the two directions (different SSP gradient
   contributions per direction), the symmetric-delay assumption
   breaks and offset estimation biases. Not a problem in open deep
   water; real in littoral deployments.

### Modern hardware: the chip-scale atomic clock (CSAC)

TSHL's software-level discipline depends on the underlying clock
being stable enough that skew `α` is constant over the estimation
window (minutes). TCXO (temperature-compensated crystal) gives
~0.1-10 ppm, enough for TSHL to handle. Modern OCXO (oven-controlled
crystal) gives ~0.01 ppm. **CSAC (chip-scale atomic clock)** —
Microsemi SA.45s and equivalents — gives ~0.0003 ppm at ~150 µW, a
$2-5k component. CSAC is what enables open-loop OWTT ranging for
hours without re-sync.

Power budget comparison:
| Clock | Stability σ_y(1s) | Drift / hour | Power | Cost |
|-------|-------------------|--------------|-------|------|
| Cheap TCXO | 10⁻⁷ | ~1 ms | 10 µW | $5 |
| Good OCXO  | 10⁻¹¹ | ~30 µs | 100 mW | $500 |
| CSAC (SA.45s) | 3×10⁻¹⁰ | ~1 µs | 150 µW | $3000 |
| Lab-grade Cs | 10⁻¹³ | ~0.3 ns | 50 W | $50k |

CSAC is the sweet spot for sonobuoys — the `µs`-level drift matches
OWTT's range-error target of ~15 cm, and the 150 µW draw fits inside
the Tier-2 (5 mW) budget from SYNTHESIS.md §3.

### D-Sync / LT-Sync mobility extensions

Surface buoys drift at ~0.5-2 m/s. Over a 10-s TSHL skew-estimation
window, the direct-path distance to a reference buoy changes by
~5-20 m, corresponding to ~3-13 ms of delay change — *much* larger
than TSHL's target precision. D-Sync's fix:

1. Modem-reported Doppler shift `f_D / f_c = v_rel / c` is measured
   per packet during normal demodulation.
2. Propagation-delay *change* is estimated as
   `Δd = v_rel · dt ≈ (f_D / f_c) · c · dt`.
3. Skew regression now uses corrected times:
   `T_ref,k^{corrected} = T_ref,k + d_0 + Σ Δd_{j<k}`.

Reduces mobile clock-sync error by 5-10× in sea-trial experiments.

## Portable details — the sonobuoy clock-sync stack

### GPS-at-surface as the anchor

Sonobuoys spend most of deployment at the surface (antenna above
water). GPS provides:
- Position: ~1-5 m lat/lon (L1 SPS) or ~10 cm (RTK if reference)
- Time: pulse-per-second (PPS) at ~10 ns precision

Every GPS PPS is used to (a) reset the CSAC to true GPS time, (b)
log the CSAC's offset and drift-rate for subsequent open-loop
operation, and (c) re-anchor the TSHL-level skew/offset state.

### On-buoy clock discipline loop

```
  GPS PPS (1 Hz) ── steers ──▶ CSAC (~150 µW, ~1 µs/hr drift)
                                 │
                                 ▼
                     Ranging modem timing reference
                                 │
                                 ▼
                  JANUS transmit: tx_time_us on each broadcast
```

Between surfaces, CSAC drifts open-loop. With ~1 µs/hr drift and
a re-sync interval of 6 hours (typical between surface intervals
in tactical mode), accumulated offset is ~6 µs = 0.9 cm range
equivalent — well below the ranging error floor.

### Rust skeleton

```rust
/// Clock state per buoy.
#[derive(Debug, Clone, Copy)]
pub struct ClockState {
    pub bias_us: i64,             // (T_local - T_ref) since last PPS
    pub drift_ppm: f64,            // open-loop drift rate
    pub last_pps_us: u64,          // last GPS PPS anchor
    pub allan_dev_1s: f64,         // CSAC spec, ~3e-10
}

impl ClockState {
    pub fn propagate(&mut self, dt_us: u64) {
        self.bias_us += (self.drift_ppm * dt_us as f64 * 1e-6) as i64;
    }
    pub fn on_gps_pps(&mut self, pps_time_us: u64) {
        let now = self.last_pps_us + /* elapsed */ 0;
        let old_bias = self.bias_us;
        self.bias_us = 0;
        self.drift_ppm = (old_bias as f64) / ((now - self.last_pps_us) as f64);
        self.last_pps_us = pps_time_us;
    }
}

/// TSHL protocol state (stationary mode).
pub struct TshlState {
    pub ref_node_id: u16,
    pub skew_est: f64,
    pub offset_est: f64,
    pub delay_est: f64,
    pub one_way_samples: Vec<(u64, u64)>,  // (t_ref, t_local)
}

/// D-Sync extension: Doppler-corrected skew update.
pub fn dsync_update(
    state: &mut TshlState,
    t_ref: u64,
    t_local: u64,
    doppler_shift: f32,    // normalized, v_rel / c
    dt_since_prev: u64,
) { /* ... */ todo!() }
```

## Integration with the sonobuoy stack

Clock sync is the **prerequisite for OWTT ranging** (Webster-Eustice
2012, covered in `one-way-travel-time.md`) and therefore for the
entire ranging-subsystem scope of this addendum. The recommended
stack for clawft sonobuoys combines three layers: (1) **GPS PPS at
the surface** as the primary time reference, anchoring the buoy at
~10 ns whenever antenna-above-water; (2) **CSAC (Microsemi SA.45s
class)** as the open-loop holdover between surface events, drifting
at ~1 µs/hr — compatible with the Tier-2 5 mW budget from ADR-069;
(3) **TSHL + D-Sync protocol** above the hardware, giving the
distributed sync for buoys that can't see GPS (submerged subsurface
nodes, or tactical-silent buoys that refuse to deploy the GPS
antenna). The protocol runs as a WeftOS `CognitiveTick` subsystem
at 0.1 Hz average rate (one packet exchange per ~10 s), publishing
`ClockState` to the ranging EKF. The clock state also joins the
`mesh_chain.rs` Raft log so that a leader election can elect a new
clock master if the current one loses GPS lock. Adversary modelling:
because clock sync is a spoofing attack surface, every TSHL / D-Sync
message is signed via `rvf-crypto`, and the Multi-Krum aggregator
(ADR-076) rejects outlier skew reports.

## Strengths

1. **First-class treatment of propagation delay** — TSHL's
   separation of skew and offset phases is exactly right for
   acoustic networks; the 20 years of follow-on work build on but
   don't replace this insight.
2. **Bandwidth-efficient** — one broadcast per 10 s steady-state is
   compatible with the JANUS 80 bps channel.
3. **Hardware story is solved** — CSAC at 150 µW + GPS PPS gives
   ~1 µs/hr open-loop drift; fits tier-2 power envelope.
4. **Composes with OWTT ranging** — the Webster-Eustice OWTT EKF
   takes TSHL-synchronized timestamps as input; they are layered
   cleanly.
5. **Mobility-aware extensions exist** — D-Sync (2018), LT-Sync
   (2025) handle drifting nodes with 5-10× improved accuracy.

## Limitations

1. **TSHL assumes stationary nodes** — baseline protocol breaks for
   drifting buoys; must use D-Sync or LT-Sync in practice.
2. **GPS dependency at the surface** — buoys that can't deploy a
   GPS antenna (submerged, fouled, jammed) drift open-loop forever
   without a peer sync.
3. **CSAC cost** — ~$3k per buoy is non-trivial for expendable
   sonobuoys; acceptable for tactical (ADR-070) but rough for
   low-cost PAM deployments.
4. **No Byzantine robustness in-protocol** — TSHL assumes honest
   reference nodes. Must layer Multi-Krum (ADR-076) above.
5. **Asymmetric propagation delay biases offset** — in shallow
   coastal deployments with strong SSP gradients, the reciprocal-
   path assumption fails; residual offset error ~100 µs possible.

## Follow-up references

1. **Liu, J., Zhou, Z., Peng, Z., Cui, J.-H., Zuba, M., Fiondella, L.**
   (2013). "Mobi-Sync: Efficient Time Synchronization for Mobile
   Underwater Sensor Networks." *IEEE TPDS* 24(2):406. DOI:
   10.1109/TPDS.2012.164. First mobility-aware underwater sync.
2. **Lu, F., Mirza, D., Schurgers, C.** (2018). "D-Sync: Doppler-
   based Time Synchronization for Mobile Underwater Sensor
   Networks." *Sensors* 18(6):1854. DOI: 10.3390/s18061854. Uses
   modem Doppler shift for delay-change estimation.
3. **Wang, B. et al.** (2025). "LT-Sync: A Lightweight Time
   Synchronization Scheme for High-Speed Mobile Underwater Acoustic
   Sensor Networks." *J. Marine Sci. Eng.* 13(3):528. DOI:
   10.3390/jmse13030528. Current state-of-the-art for high-speed
   mobile nodes.
4. **Microchip / Microsemi** (2018). *SA.45s Chip-Scale Atomic
   Clock Datasheet*. Publicly distributed. The clock hardware.
5. **Zhang, Y., Yang, X., Wang, Y.** (2018). "Survey of Time
   Synchronization in Underwater Acoustic Sensor Networks."
   *Sensors* 18(10):3397. DOI: 10.3390/s18103397. Comprehensive
   survey situating TSHL and all descendants.
