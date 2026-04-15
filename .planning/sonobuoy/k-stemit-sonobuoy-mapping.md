# K-STEMIT → Sonobuoy Mapping

**Source paper**: Zesheng Liu, Maryam Rahnemoonfar. *"K-STEMIT: Spatio-Temporal
GNN for Subsurface Estimation from Radar."* arXiv:2604.09922.
**Reference card**: [`papers/k-stemit.md`](papers/k-stemit.md)
**Extracted from**: `.planning/development_notes/knowledge-graph-paper-survey-phase2.md` (Paper 7, phase-2 KG survey, 2026-04-10)
**Moved**: 2026-04-15

---

This document extracts the sonobuoy-specific analysis of K-STEMIT from the
phase-2 knowledge-graph paper survey. The general EML-coherence applicability
of K-STEMIT remains in the original survey; only the sonobuoy-specific
mapping lives here.

---

## Why K-STEMIT transfers to sonobuoy

K-STEMIT addresses *ice-penetrating radar* but the techniques map onto
*underwater acoustic sensing* with strong parallels:

### Radar-to-acoustic signal processing mapping

| Radar (K-STEMIT) | Acoustic (Sonobuoy) | Mapping strength |
|-------------------|----------------------|-------------------|
| Radargram (2D backscatter image) | Spectrogram (2D time-frequency) | **Strong** -- both are 2D representations of 1D signal returns |
| Ice layer boundaries (reflections) | Species vocalizations (acoustic signatures) | **Strong** -- both are pattern recognition in noisy returns |
| Speckle noise + acquisition artifacts | Ocean ambient noise + multipath | **Strong** -- both degrade signal quality |
| S-C band (6 GHz bandwidth) | Sonar bands (1 Hz - 200 kHz) | **Moderate** -- different physics but same signal processing abstractions |
| Snow Radar flight lines (spatial track) | Buoy array geometry (spatial distribution) | **Strong** -- both are spatially distributed sensor networks |
| MAR atmospheric model (physical priors) | Ocean acoustic propagation model (physical priors) | **Very strong** -- both inject domain physics into ML |

## Specific techniques applicable to sonobuoy

### 1. GraphSAGE spatial processing on buoy array geometry

K-STEMIT builds fully-connected graphs from 256 radar trace points using
haversine distance. For the distributed buoy array, construct a graph where
each buoy is a node, edges weighted by acoustic propagation delay (function
of distance, sound speed profile, and bathymetry). GraphSAGE's neighborhood
aggregation `x'(v) = W1*x(v) + W2*AGG x(u)` then learns spatial features
that encode array geometry, enabling the model to implicitly learn
beamforming-like spatial filtering without explicit beamformer design.

### 2. Gated temporal convolution for acoustic time series

The GLU-gated temporal branch directly applies to sonobuoy hydrophone time
series. Each buoy produces continuous acoustic data; the gated convolution
`P * sigma(Q) + R` can learn to extract transient acoustic events (whale
calls, ship propeller signatures, sonar pings) while suppressing stationary
noise. This is analogous to matched filtering but learned from data rather
than designed.

### 3. Adaptive spatial-temporal fusion for detection / classification

The learnable `alpha in [0,1]` that balances spatial and temporal branches is
directly applicable to the sonobuoy's dual task:

- **Detection** (is something there?) is primarily temporal — a single buoy
  can detect transient energy.
- **Bearing estimation** is primarily spatial — requires cross-correlation
  across the array.
- **Species ID** requires both — spectral signature (temporal) + source
  location (spatial) for disambiguation.

The adaptive alpha lets the model learn to weight these differently for
different targets.

### 4. Physics-informed node features from ocean models

K-STEMIT integrates 5 MAR atmospheric variables as node features. For
sonobuoy, inject:

- Sound speed profile (depth-dependent, from CTD or climatology)
- Thermocline depth (affects propagation paths)
- Sea state / wind speed (affects surface noise)
- Current velocity (affects bearing estimation via Doppler)
- Bottom type (affects bottom-bounce propagation)

These physical priors would be integrated via the same Delaunay
triangulation interpolation K-STEMIT uses.

### 5. Dimensionality reduction strategy

K-STEMIT strips static geographic coordinates from the temporal branch,
concatenates them only into the spatial branch. For sonobuoy: strip buoy
GPS positions from the acoustic feature branch (they don't change per
sample), keep only dynamic acoustic features (spectral energy, zero-crossing
rate, mel coefficients). This prevents the temporal model from overfitting
to buoy identity rather than acoustic content.

## Detection range / Species ID / Bearing estimation impact

- **Detection range**: K-STEMIT's physics-informed approach achieved 21%
  RMSE reduction over pure data-driven baselines. For sonobuoy, injecting
  sound speed profile and thermocline data as node features should improve
  detection range estimation by correcting for propagation loss that pure
  ML models cannot learn from acoustic data alone.

- **Species ID**: The gated temporal convolution with GLU activation
  provides learned matched filtering. Unlike fixed-template matched
  filters (which require a priori species call libraries), the learned
  filter can generalize to intra-species call variation. Combined with
  GraphSAGE's spatial context (nearby buoys seeing the same source at
  different angles), this should improve species ID accuracy for
  vocalizing species.

- **Bearing estimation**: GraphSAGE's neighborhood aggregation on the
  array geometry graph implicitly learns time-difference-of-arrival
  (TDOA) relationships. The haversine-weighted edges encode propagation
  delays. For a distributed array where buoy positions drift, this is
  superior to conventional TDOA beamforming because it adapts to the
  actual (possibly irregular) array geometry rather than assuming a
  fixed array.

## Beamforming / Array processing applicability

K-STEMIT does **not** explicitly introduce beamforming or array processing
techniques. However, its GraphSAGE spatial processing on haversine-weighted
edges is *functionally equivalent to learned beamforming*:

- Conventional delay-and-sum beamforming:
  `y = sum_i w_i * x_i(t - tau_i)` where `tau_i` are steering delays and
  `w_i` are array weights.
- GraphSAGE aggregation: `x'(v) = W1*x(v) + W2*AGG_{u in N(v)} x(u)`
  where `AGG` is a learned function over weighted neighbors.

The key advantage: GraphSAGE learns the aggregation function from data,
which can capture non-linear propagation effects (refraction, multipath,
scattering) that linear delay-and-sum beamforming cannot. For a distributed
buoy array with irregular geometry and time-varying sound speed profiles,
this learned aggregation may outperform conventional beamforming.

**Limitation**: K-STEMIT's graphs are fully connected (all 256 nodes
interconnected). For a buoy array with N=20-100 buoys, full connectivity
is feasible. For larger arrays, a k-nearest-neighbors graph (k=8-16) would
be more computationally efficient while preserving local spatial structure.

## Priority for the sonobuoy project

**P0** — The dual-branch spatio-temporal architecture is the highest-value
contribution for the sonobuoy project. It provides a unified framework for
detection, bearing estimation, and species ID that replaces three separate
signal processing pipelines with one learned model.

## ADR candidate

**ADR-053: Spatio-Temporal Dual-Branch Architecture for Sensor Systems** —
adopt K-STEMIT's architecture for the sonobuoy project. (Originally listed
in the phase-2 survey's ADR roadmap; retained here as the canonical home.)
