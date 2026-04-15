# K-STEMIT — Reference Card

**Full title**: K-STEMIT: Knowledge-Informed Spatio-Temporal Efficient
Multi-Branch Graph Neural Network for Subsurface Stratigraphy Thickness
Estimation from Radar Data

**Authors**: Zesheng Liu, Maryam Rahnemoonfar

**Submitted**: 2026-04-10

**arXiv**: [2604.09922](https://arxiv.org/abs/2604.09922)
**PDF**: https://arxiv.org/pdf/2604.09922
**DOI**: https://doi.org/10.48550/arXiv.2604.09922
**Primary classification**: cs.LG (also cs.CV)

## Abstract

K-STEMIT is a neural network for estimating subsurface ice layer thickness
from radar data. It addresses limitations of traditional methods by combining
geometric spatial learning with temporal dynamics modeling, incorporates
physical weather data, and employs adaptive feature fusion across multiple
branches. The model "achieves the highest accuracy while maintaining
near-optimal efficiency" and reduces error by approximately 21% versus
conventional variants when physical priors are integrated.

## Architecture (as applied here)

- **Dual-branch**: Spatial (GraphSAGE on geographic-proximity graphs) +
  Temporal (gated temporal convolution with GLU activation).
- **Adaptive fusion**: Learnable scalar `alpha in [0,1]` combines
  `h = alpha * h_spatial + (1 - alpha) * h_temporal`.
- **Physics-informed features**: 5 MAR climate variables (surface mass
  balance, temperature, refreezing, melt-induced height, snowpack depth)
  integrated via 2D Delaunay interpolation.
- **Fully-connected spatial graphs**: 256 nodes, haversine-distance edge
  weights.

## Reported results

- 15.23% RMSE reduction vs SOTA at 2.4x speed.
- 21% RMSE reduction when physical priors are included.

## Why it matters for sonobuoy

See [`../k-stemit-sonobuoy-mapping.md`](../k-stemit-sonobuoy-mapping.md) for
the full radar-to-acoustic mapping, including:

- GraphSAGE → learned beamforming on distributed buoy arrays
- Gated temporal convolution → learned matched filtering on hydrophones
- Physics-informed features → ocean acoustic priors (SSP, thermocline, etc.)
- Adaptive alpha fusion → unified detect / bearing / species-ID model

**Priority for sonobuoy**: **P0** (foundational).

## Local copy

No local PDF snapshot is committed (storage hygiene). Pull on demand:

```bash
curl -L -o k-stemit.pdf https://arxiv.org/pdf/2604.09922
```

## Original survey entry

The general-applicability (EML coherence) portion of this paper remains in
`.planning/development_notes/knowledge-graph-paper-survey-phase2.md` (§Paper
7: K-STEMIT). Only the sonobuoy-specific mapping was extracted to this
folder.
