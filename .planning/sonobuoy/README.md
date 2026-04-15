# Sonobuoy Project — Research Root

Research and planning folder for the sonobuoy / underwater acoustic sensing
project. Extracted from the phase-2 knowledge-graph paper survey on
2026-04-15 and augmented with additional sonar, fish-ID, and marine
bioacoustics literature.

## Scope

The sonobuoy project is a planned crate in the clawft workspace that
unifies three signal-processing tasks — detection, bearing estimation, and
species ID — into a single learned model operating on distributed
hydrophone-array data. Shared infrastructure with:

- **Quantum cognitive layer** — `quantum_register` graph-to-layout
  mapping reuses for buoy-array geometry graphs.
- **EML learned-function layer** — replaces hardcoded signal-processing
  thresholds.
- **HNSW vector service** — call/signature retrieval for species ID.

## Contents

| File | Purpose |
|------|---------|
| `k-stemit-sonobuoy-mapping.md` | Full K-STEMIT → sonobuoy mapping (radar-to-acoustic, learned beamforming, physics priors). Extracted from the phase-2 KG survey. |
| `papers/k-stemit.md` | K-STEMIT reference card (abstract, arXiv link, architecture summary). |
| `papers/survey.md` | Curated survey of additional sonar / fish-ID / bioacoustics papers (populated by the researcher agent). |

## Foundational architecture (from K-STEMIT)

```text
                        adaptive alpha
                             |
                             v
     +-------------+    +---------+    +--------------+
     | GraphSAGE   |--->|  fuse   |--->| detect head  |
     | spatial     |    |         |--->| bearing head |
     +-------------+    |         |--->| species head |
                        |         |
     +-------------+    |         |
     | GLU-gated   |--->|         |
     | temporal    |    |         |
     +-------------+    +---------+

     node features:       inputs:
     - buoy GPS             - hydrophone time series per buoy
     - depth                - spectrogram features
     - SSP                  - TDOA correlations (optional)
     - thermocline
     - sea state
```

## Workflow from here

1. Read `k-stemit-sonobuoy-mapping.md` for the foundational architecture.
2. Read `papers/survey.md` for complementary papers (passive sonar, fish ID,
   marine bioacoustics, graph-based array processing, audio foundation
   models).
3. Draft `ADR-053: Spatio-Temporal Dual-Branch Architecture for Sensor
   Systems` once the paper survey stabilizes.
4. Scaffold a `crates/clawft-sonobuoy/` crate with the dual-branch model
   skeleton, reusing `quantum_register::build_register` for array geometry.
