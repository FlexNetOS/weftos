# Bahr, Leonard, Fallon 2009 — Cooperative Localization for AUVs + Otero 2023 — Drifting-Buoy Positioning

## Citation

### Primary (cooperative AUV localization)

- **Authors**: Alexander Bahr, John J. Leonard, Maurice F. Fallon
- **Title**: "Cooperative Localization for Autonomous Underwater
  Vehicles"
- **Venue**: *International Journal of Robotics Research*, Vol. 28,
  No. 6 (June 2009), pp. 714–728
- **DOI**: https://doi.org/10.1177/0278364908100561
- **Open PDF (MIT DSpace)**:
  https://dspace.mit.edu/bitstream/handle/1721.1/58207/bahr_fallon_leonard_ijrr2009.pdf

### Primary (modern drifting-buoy positioning, direct sonobuoy analog)

- **Authors**: Pablo Otero, Álvaro Hernández-Romero,
  Miguel-Ángel Luque-Nieto, Alfonso Ariza
- **Title**: "Underwater Positioning System Based on Drifting Buoys
  and Acoustic Modems"
- **Venue**: *Journal of Marine Science and Engineering*, Vol. 11,
  No. 4 (March 2023), Article 682
- **DOI**: https://doi.org/10.3390/jmse11040682
- **Open**: https://www.mdpi.com/2077-1312/11/4/682 (CC BY 4.0)

### Companion (LBL equations, same group)

- **Otero, P., Hernández-Romero, Á., Luque-Nieto, M.Á.** (2022).
  "LBL System for Underwater Acoustic Positioning: Concept and
  Equations." arXiv:2204.08255.

## Status

**Verified.** The 2009 IJRR paper is indexed at SAGE Publishing under
DOI 10.1177/0278364908100561 (vol 28, issue 6, pp. 714-728) and is
open-access on MIT DSpace. The Otero 2023 JMSE paper is verified via
MDPI DOI 10.3390/jmse11040682 (volume 11, issue 4, article 682, CC
BY 4.0 license). The Otero 2022 arXiv companion is verified via
arXiv:2204.08255 (3 pp, 9 equations). The MDPI paper is the single
most direct modern reference for the project concept — it describes
a surface network of four GNSS-equipped drifting buoys with acoustic
modems that give positioning to underwater users without requiring
synchronized atomic clocks.

## Historical context

Bahr-Leonard-Fallon 2009 is the foundational cooperative-localization
paper for AUVs: instead of every vehicle being localized
independently against static beacons, a network of vehicles shares
acoustic range and position information, and each vehicle solves a
joint estimation problem over all members' positions. The algorithmic
innovation is a **distributed EKF with delayed-state update** — each
vehicle maintains an estimate of all others' positions and uncertainties
and updates them as acoustic packets arrive out of order with
high-latency propagation (~1 s/km). This is the canonical reference
for every subsequent "AUV swarm" navigation paper.

Otero et al. 2023 is the closest published system to the clawft
sonobuoy ranging concept: **four GNSS-equipped drifting buoys on the
surface, each broadcasting its position + transmit timestamp on a
scheduled TDMA slot; an underwater user listens and derives its own
position from pseudo-ranges without needing a synchronized clock**.
The four-buoy geometry is exactly what the K-STEMIT spatial branch
wants, and the TDoA-pseudorange formulation avoids the strong clock
requirement of OWTT (Webster-2012). For any paper that directly
maps onto the sonobuoy-ranging project, this is the one.

## Core content

### Bahr-Leonard-Fallon 2009 — cooperative framework

**The problem.** `N` AUVs operate in a shared volume. Each has a
local DVL (Doppler velocity log) + IMU giving dead-reckoned position
with bias that grows ~1% of distance traveled. Acoustic modems allow
occasional inter-vehicle range measurements (RTT or OWTT). How
should the team fuse all this into self-consistent global positions?

**The algorithm.** Each vehicle maintains an augmented EKF state
including its own position + estimates of peer positions at recent
time stamps. A range measurement from peer `j` at `t_tx` to vehicle
`i` at `t_rx` is processed by (a) projecting vehicle `i`'s state
back to `t_tx`, (b) using the broadcast packet's payload to update
peer `j`'s position estimate at `t_tx`, (c) computing the range
residual, and (d) applying the EKF correction to both estimates.
This is a **decentralized smoother** — no central coordinator, each
vehicle computes its own solution.

**Key result.** Experimental evaluation on MIT Sailing Pavilion (3
surface kayaks as AUV surrogates) + WHOI Slocum glider + WHOI
Bluefin12 AUV with surface-craft aids. Dead-reckon error reduced by
5-10× compared to no cooperation; absolute error bounded at ~2-3 m
RMS over multi-hour missions.

**The observability insight.** Pairs of vehicles converging on
identical trajectories lose observability — the range residual
can't distinguish common-mode drift. The paper gives a rank-based
observability test and design rules: keep at least one vehicle as
a "surface craft" with GPS, or maintain non-trivial relative
geometry among submerged peers.

### Otero et al. 2023 — drifting-buoy positioning

**The architecture.** Four surface buoys, each equipped with:
- GNSS receiver (1-5 m accuracy, 1 Hz fix)
- Acoustic modem (transmit only, down-looking transducer)
- LoRa transceiver (for recovery/control)
- Microcontroller + power supply

Each buoy transmits **sequentially** (TDMA) a packet containing its
own position `q_i` at the instant of transmission `t_tx,i` and `t_tx,i`
itself (from its GNSS-disciplined clock). The underwater user
hears all four broadcasts and computes its position from
**pseudo-ranges**:

    ρ_i = c̄ · (t_rx,i - t_tx,i)        = R_i + c̄ · Δ

where `Δ` is the user's clock offset (unknown constant over the
4-broadcast window ~10 s). Subtracting pairs eliminates `Δ`:

    ρ_i - ρ_1 = R_i - R_1               hyperbolic equations

Four buoys give three independent hyperbolic equations for three
unknowns `(x, y, z)` — the user's position. **No synchronized atomic
clock is needed at the user**; the system is pure TDoA.

**Key result.** The paper is an analytical design study (feasibility
+ equations + error analysis), not a sea trial. It reports:
- Minimum acoustic data rate: 640 bps
- Minimum acoustic bandwidth: ~1 kHz
- Max buoy-to-buoy distance: ~866 m for good geometry
- Typical baseline span: 500-1500 m
- Position error due to buoy drift during the 10-s broadcast window:
  bounded by `v_buoy · T_frame / 2` ≈ 5-10 m for surface drift at
  1 m/s over 10 s
- Movement-related error dominates over ranging error in a
  drifting field — motivating faster TDMA cycles

### Otero 2022 arXiv — LBL equations

The 2022 preprint (9 equations, 3 pp) gives the underlying LBL math
the 2023 paper references. Hyperbolic trilateration:

    (x - q_i_x)² + (y - q_i_y)² + (z - q_i_z)² = R_i²
    R_i = ρ_i - c̄ · Δ        for i = 1..4

Subtracting:

    (R_i² - R_j²) = (ρ_i² - ρ_j²) - 2 · c̄ · Δ · (ρ_i - ρ_j)

which linearizes on `Δ` and `(x, y, z)` simultaneously.

## Portable details — the cooperative-TDoA algorithm

### Joint state and update

`N` buoys at GPS-known positions `q_i(t)`, each with clock offset
`δ_i`. User at unknown `p(t)`. Measurement per broadcast `i`:

    y_i = c̄ · (t_rx - t_tx^{i}) + ε_i
        = ||p - q_i|| + c̄ · (δ_user - δ_i) + ε_i

Defining composite clock bias `b ≡ c̄ · δ_user`, and using the fact
that GPS disciplines each buoy so `δ_i ≈ 0` up to GNSS noise
(~10 ns), we have the standard GPS pseudorange form

    y_i = ||p - q_i|| + b + ε_i'     with ε_i' absorbing GNSS jitter

This is exactly GPS's own equations. Solve by weighted least squares
(3+1 unknowns, ≥4 measurements):

    p̂ = argmin Σ_i w_i · (y_i - ||p - q_i|| - b)²

### Error budget (inter-buoy drift regime)

For a sonobuoy field at 500 m spacing, carrier 10 kHz, 256-sample
matched-filter chirp:
- Matched-filter timing jitter: ~50 µs → 7.5 cm range
- Sound-speed uncertainty: ~0.5 m/s over 500 m → 17 cm range
- Buoy-GPS noise: ~2 m per pair (both GPS receivers)
- Clock bias from GNSS discipline: ~10 ns → 1.5 cm (negligible)
- Surface drift during 1-s TDMA frame: ~1 m
**Total: ~2.5 m position RMS**, dominated by buoy-GPS noise, not
acoustic. Once the acoustic ranging itself is <1 m, adding a
smoothing filter over GPS reduces the GPS contribution to
~0.5-1 m — the regime where the ranging subsystem is GPS-
competitive.

### Rust skeleton

```rust
/// Drifting buoy that broadcasts position + timestamp.
pub struct RangingBuoy {
    pub id: u16,
    pub gnss: GnssFix,              // { position, time, sigma }
    pub clock: Clock,                // disciplined to GPS PPS
    pub transducer: AcousticTx,
    pub tdma_slot: u8,
}

/// Incoming broadcast heard by this node.
#[derive(Debug, Clone, Copy)]
pub struct Broadcast {
    pub peer_id: u16,
    pub peer_position: [f64; 3],
    pub peer_position_sigma: [f64; 3],
    pub tx_time_us: u64,
    pub rx_time_us: u64,
    pub matched_filter_sigma_us: f32,
}

/// Cooperative pseudo-range solver. Takes `N ≥ 4` broadcasts per
/// epoch and returns the listener's position + clock-offset estimate.
pub fn solve_pseudorange(
    broadcasts: &[Broadcast],
    c_bar_mps: f64,
    prior: [f64; 4],
) -> ([f64; 3], f64, [[f64; 4]; 4]) {
    // Weighted least-squares; returns (position, b, covariance).
    todo!()
}
```

### Joint multi-buoy EKF (Bahr-Leonard-Fallon-style)

When every buoy is also a user (reciprocal broadcasts), the cooperative
EKF maintains a `4N`-dimensional state `(p_1, v_1, ..., p_N, v_N)`
with cross-correlations, updated on every peer-to-peer broadcast.
This is exactly the Bahr-2009 algorithm adapted to the sonobuoy
topology. Scales to ~30-50 buoys before the `O(N²)` covariance
update becomes expensive.

## Integration with the sonobuoy stack

Otero-2023 is the **reference architecture** for the clawft ranging
subsystem — its 4-buoy TDoA scheme maps directly onto the K-STEMIT
spatial branch's desire for ≥4 buoys with meter-scale relative
geometry, and crucially it avoids the strong clock-sync requirement
of OWTT (Webster-2012). The pseudo-range form is exactly GPS's own
equations, so the Kalman filter, RAIM integrity monitoring, and
weighted-least-squares machinery from decades of GNSS engineering
port over directly. Bahr-Leonard-Fallon-2009 is the **generalization**
to the case where all buoys are also users, giving a decentralized
joint state that respects the out-of-order, high-latency nature of
acoustic channels. In the K-STEMIT pipeline, the joint-EKF output
becomes the `D(t), σ(t)` pair consumed by Tzirakis-2021 GCN
adjacency, Grinstein-2023 Relation-Network metadata, and Chen-Rao-2025
Grassmannian subspace DoA. The cooperative state is maintained by a
new WeftOS module `clawft-sonobuoy-ranging::coop_ekf` that publishes
range-change events to the `Impulse` queue whenever `σ_{ij}(t)`
crosses a threshold. The ranging subsystem's clock model naturally
extends the ECC `cognitive_tick` schedule: each buoy's `PROPAGATE →
SENSE → UPDATE` loop runs at the TDMA slot cadence (0.1-1 Hz), with
GPS re-sync events at surface.

## Strengths

1. **No atomic clock required** (Otero-2023) — GNSS-disciplined
   clocks give ~10 ns precision at each buoy, enough for pseudo-range
   TDoA without CSAC. Saves $2-5k and 150 µW per buoy.
2. **Proven cooperative framework** (Bahr-2009) — the delayed-state
   EKF is the standard for underwater cooperative localization; has
   real sea-trial validation.
3. **Observability design rules** (Bahr-2009) — explicit rank
   conditions for when a formation is self-localizable.
4. **Scales gracefully** — TDMA slots keep channel utilization
   linear in `N`; the joint EKF has `O(N²)` state but compressions
   (sparse information filter, factor-graph) take it to `O(N)`.
5. **Open-source licensing** — Otero-2023 is CC BY 4.0; can be
   re-implemented without IP risk.

## Limitations

1. **Otero-2023 is analytical, not experimental** — no sea trial
   validates the claimed accuracies in open water. Clawft should
   expect to do its own shakedown.
2. **4-buoy minimum** — pseudo-range TDoA needs ≥4 broadcasts to
   solve `(x, y, z, b)`. Sub-4 fields need OWTT or LBL.
3. **Covariance grows in unobservable directions** — Bahr-2009
   notes that common-mode drift among peers is unobservable
   without an external reference. In a sonobuoy field, at least
   one buoy with a clean GPS fix anchors the solution.
4. **TDMA scheduling is a coordination problem** — requires a
   protocol above the broadcast layer. JANUS (Potter-Alves 2014)
   or a custom lightweight MAC solves this.
5. **Multipath in shallow water degrades pseudo-ranges** — the
   papers assume direct-path arrival dominates, which fails in
   littoral deployments (<50 m depth).

## Follow-up references

1. **Fallon, M.F., Papadopoulos, G., Leonard, J.J.** (2010). "A
   Measurement Distribution Framework for Cooperative Navigation
   using Multiple AUVs." IEEE ICRA 2010. Extends Bahr-2009 to
   handle acoustic packet losses and delays.
2. **Paull, L., Seto, M., Leonard, J.J.** (2014). "Decentralized
   Cooperative Trajectory Estimation for Autonomous Underwater
   Vehicles." IEEE IROS 2014. Factor-graph reformulation of
   Bahr-2009, `O(N)` in state.
3. **Ferri, G., Munafò, A., LePage, K.D.** (2018). "An
   Autonomous Underwater Vehicle Data-Driven Control Strategy for
   Target Tracking." IEEE JOE 43(2):323. DOI:
   10.1109/JOE.2018.2797558. Cooperative ranging + target-tracking
   integration.
4. **Walls, J.M., Eustice, R.M.** (2014). "An origin state method
   for communication constrained cooperative localization with
   robustness to packet loss." IJRR 33(9):1191. DOI:
   10.1177/0278364914525856. Delayed-state EKF variant for lossy
   acoustic channels.
5. **Otero, P., Hernández-Romero, Á., Luque-Nieto, M.Á.** (2022).
   "LBL System for Underwater Acoustic Positioning: Concept and
   Equations." arXiv:2204.08255. The companion theoretical note
   for Otero-2023.
