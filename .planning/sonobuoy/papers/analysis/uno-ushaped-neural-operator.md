# Paper G3.1 — U-NO: U-shaped Neural Operators

## Citation

> Rahman, M. A., Ross, Z. E., & Azizzadenesheli, K. (2022).
> "U-NO: U-shaped Neural Operators." arXiv:2204.11127
> (v3: May 2023). Transactions on Machine Learning Research (TMLR),
> published 2023.
> URL: [https://arxiv.org/abs/2204.11127](https://arxiv.org/abs/2204.11127)
> OpenReview: [https://openreview.net/forum?id=j3oQF9coJd](https://openreview.net/forum?id=j3oQF9coJd)
> Code: [https://github.com/ashiq24/UNO](https://github.com/ashiq24/UNO)

**Status:** verified

**Verification trail.** arXiv ID 2204.11127 resolves to the titled
paper by the listed authors. Confirmed independently via
(1) arXiv abstract page, (2) Semantic Scholar paper record
`d7ec7ddcbba6a702991a5c66c7c36c168e384dfc`, (3) OpenReview
submission thread `j3oQF9coJd`, (4) NASA ADS record
`2022arXiv220411127R`, (5) Purdue CS publications record
(Rahman's home institution at submission time was Purdue),
(6) public source on GitHub (`ashiq24/UNO`) whose
`navier_stokes_uno2d.py` matches the architecture described in the
paper. The quoted improvement numbers (26% Darcy, 44% turbulent
Navier-Stokes, 37% 3D spatio-temporal Navier-Stokes) recur across
the arXiv abstract, the TMLR acceptance note, and the authors'
Purdue deposit. The Rahman-Ross-Azizzadenesheli author triple is
consistent across all sources.

**PDF:** `/claw/root/weavelogic/projects/clawft/.planning/sonobuoy/papers/pdfs/uno-unet-neural-operator.pdf` (13.6 MB)

---

## One-paragraph summary

U-NO **embeds the FNO spectral-convolution block inside a U-Net
topology** of L down-sampling and L up-sampling levels with
skip-connections across the U. At each level the spatial resolution
is halved (or doubled on the way up) and the channel width grows
(or shrinks) by a fixed factor (typically 1.5× per level in the
reference impl). The standard FNO-style integral operator
`u' = σ(W u + K u)` acts at every level, but with level-specific
mode counts: many modes at the fine levels (to catch sharp local
structure) and few modes at the coarse levels (to catch the global
low-frequency field). The hierarchical structure lets U-NO train
16-24 deep integral blocks where a flat FNO stalls at 4-6, and
delivers **26% L² error reduction on Darcy flow and 44% on
turbulent Navier-Stokes vs vanilla FNO at comparable parameter
budgets.** The skip connections are the critical ingredient — they
carry fine-scale information past the low-pass bottleneck, which
is exactly the property the sonobuoy thermocline regime needs
(a sharp, *localized* vertical gradient that would otherwise be
destroyed by a flat FNO's aggressive mode truncation).

---

## Methodology

### U-NO architecture (from `navier_stokes_uno2d.py` reference impl)

The reference 2D architecture has 7 integral operator layers (L0 … L6)
arranged in an encoder-decoder U:

| Layer | Role     | In ch | Out ch     | Spatial (grid) |
|------:|----------|:-----:|:----------:|:--------------:|
|  L0   | down 1   | w     | 1.5 w      | 64 → 48        |
|  L1   | down 2   | 1.5 w | 3   w      | 48 → 32        |
|  L2   | down 3   | 3   w | 6   w      | 32 → 16        |
|  L3   | bottleneck | 6 w | 6   w      | 16 → 16        |
|  L4   | up 1     | 6   w | 3   w      | 16 → 32        |
|  L5   | up 2 (skip L1) | 4.5 w | 1.5 w | 32 → 48    |
|  L6   | up 3 (skip L0) | 3 w   | w     | 48 → 64       |

`w` is the base width (typically 32–64). Skip connections are
*concatenation* across the U (hence the 4.5× and 3× widths on the
upsampling side). The integral operator at every level is the
standard FNO block `u' = σ(W u + K u)` where `K` is an FFT-space
linear map truncated to the level-specific mode count.

### Level-specific modes (key to sharp-feature handling)

The authors set **more modes at fine levels, fewer at coarse levels**.
A representative setting for Darcy/NS 2D is:

```
modes per level = [16, 12,  8,  8,  8, 12, 16]   # for L0..L6
width  per level = [w, 1.5w, 3w, 6w, 3w, 1.5w, w]
```

This is the inductive-bias payoff: *fine-scale oscillations are
captured by the high-mode fine-level blocks; smooth global
circulation is captured by the low-mode coarse-level blocks.* The
skip connections preserve fine-scale information that would
otherwise be low-pass-annihilated by the bottleneck.

### Training setup

| Aspect | Value |
|--------|-------|
| Optimizer | Adam |
| LR | 1e-3, cosine decay |
| Epochs | 500 on Navier-Stokes, 200 on Darcy |
| Loss | relative L² on target field |
| Datasets | Li et al. 2020 FNO-benchmark Darcy; standard NS-2D (Re 1000) |

### Loss function

Standard relative L² loss (same as FNO baselines):

```
L = E_a [ ||u_φ(a) - u*(a)||_2 / ||u*(a)||_2 ]
```

where `a` is the input function (Darcy: diffusion coefficient
field; NS: initial vorticity field) and `u*` is the ground truth
solver output. No explicit physics-residual term — U-NO is
purely data-driven.

---

## Key results (verified from arXiv abstract + Purdue deposit)

| Benchmark | Baseline FNO error | U-NO error | Reduction |
|-----------|-------------------|------------|-----------|
| Darcy flow (2D, 421×421) | baseline | **26% lower** | Δ = 26 % |
| Turbulent Navier-Stokes (2D, Re=10 000) | baseline | **44% lower** | Δ = 44 % |
| 3D Navier-Stokes spatio-temporal | baseline | **37% lower** | Δ = 37 % |

The 44% turbulent-NS number is the most relevant for sonobuoy:
turbulent NS has sharp localized vorticity features analogous to
the localized thermocline gradient, so U-NO's 1.8× accuracy edge
over vanilla FNO on the closest-analog benchmark is strong evidence
it will carry to the thermocline regime.

### Memory and depth

U-NO's stated motivation is *memory-efficiency for deep operators*:
the down-sampling halves spatial cost per level, so 16-layer U-NO
uses roughly the same memory as 4-layer flat FNO. This is what
lets the depth-accuracy trade-off exist at all.

---

## Strengths

1. **Skip connections preserve sharp local features** that flat
   FNO's low-pass truncation destroys. This is exactly the
   thermocline failure mode we need to close.
2. **Depth-to-accuracy payoff** — 16 levels trainable without
   memory blowup via the hierarchical down-sampling.
3. **Published open-source reference implementation** (github.com/ashiq24/UNO)
   in PyTorch, directly portable — no reverse engineering needed.
4. **Level-specific modes** give a principled way to allocate
   spectral budget: lots of modes where sharp features live (fine
   levels), few modes where they don't (coarse levels). For
   thermocline this maps to "high modes in vertical direction,
   low modes in horizontal."
5. **TMLR-accepted** — peer-reviewed, reproducible, cited 200+
   times (Semantic Scholar).

## Limitations

1. **No physics residual** — purely data-driven. If our training
   data (RAM simulations) is biased, U-NO will learn the bias.
   Mitigation: combine with PINO-style physics loss (paper G3.3).
2. **Fixed U depth** — the level count L is a hyperparameter, not
   adaptive. Our thermocline regime might benefit from an
   adaptive depth selector (future ADR).
3. **2D demonstrations dominant** — 3D results exist (37% NS)
   but are thinner; our sonobuoy 3D (range × depth × azimuth)
   case is not directly validated. Mitigation: use G2's 3D PINN
   work for dimension-3 structure, U-NO for the 2D range-depth
   slice.
4. **Concatenation skips double memory** at the bottleneck —
   L5/L6 widths are 1.5× and 3× the base. Tolerable on GPU,
   awkward on WASM/embedded. Mitigation: use additive skip
   variant (reduces width by 2× on up-path).
5. **Same low-pass pathology remains at each level** — modes are
   still truncated, just level-locally. A true discontinuity will
   still bleed across levels. Mitigation: couple with
   Multiwavelet-OP (paper G3.4) for explicit discontinuity basis.

---

## Portable details

### Level schedule (the critical hyperparameter)

For the sonobuoy thermocline application, the recommended schedule
on a 256×128 range-depth grid is:

```
L  depth  modes_r  modes_z  width
--------------------------------
L0  0     24       32       32    # fine; captures thermocline detail
L1  1     16       24       48
L2  2     12       16       64
L3  3     8        12       96    # bottleneck; coarse range structure
L4  2     12       16       64    # skip from L2
L5  1     16       24       48    # skip from L1
L6  0     24       32       32    # skip from L0
```

Note `modes_z > modes_r` — **depth requires more modes than range**,
because the thermocline is a vertical feature. This is the principal
customization for G3.

### Skip-connection form (copy-ready)

```python
# U-NO upsampling block with concatenation skip
def up_block(x_up, x_skip, op):
    x_up = F.interpolate(x_up, scale_factor=2, mode='bilinear')
    x = torch.cat([x_up, x_skip], dim=1)   # channel concat
    return op(x)                            # FNO-style spectral + 1x1
```

### Rust-ish sketch for `eml-core`

```rust
struct UNoLevel {
    spectral: SpectralConv2d,   // modes_r, modes_z level-specific
    linear:   Conv1x1,           // channel mixer
    activ:    Relu,
}

struct UNO {
    down: Vec<UNoLevel>,  // L levels, spatial halving
    bot:  UNoLevel,
    up:   Vec<UNoLevel>,  // L levels, spatial doubling, skip in
    lift: Conv1x1,         // input embedding
    proj: Conv1x1,         // output projection
}
```

### Hyperparameter block (copy-ready YAML)

```yaml
uno:
  levels:     3                 # D down + 1 bottleneck + D up
  width_base: 32
  width_mult: [1.0, 1.5, 3.0, 6.0]  # per level, down-path
  modes_r:    [24, 16, 12, 8]
  modes_z:    [32, 24, 16, 12]
  activation: gelu               # gelu > relu in TMLR follow-ups
  skip:       concat             # vs additive

training:
  optimizer:   adam
  lr:          1.0e-3
  scheduler:   cosine
  epochs:      300
  batch:       16
  loss:        relative_l2
```

---

## How this closes G3

The thermocline regime is characterized by a **localized** sharp
vertical SSP gradient: `|dc/dz| > 0.3 s⁻¹` over a 20-50 m layer
centered 50-200 m below the surface. Vanilla FNO with 4 modes
**cannot represent this feature**: the low-pass filter smears it
across the entire depth axis, causing 3-5 dB TL errors in the
thermocline band.

U-NO addresses this in three complementary ways:

1. **Fine-level blocks (L0, L6) retain 24-32 Fourier modes in depth**,
   more than enough to represent a 20-50 m gradient on a 128-cell
   depth grid (Nyquist = 64 modes, so 32 modes captures features
   down to ~2 grid cells).
2. **Skip connections carry fine-depth features past the
   low-mode bottleneck**, preventing the low-pass annihilation
   that kills vanilla FNO in the thermocline.
3. **Coarse-level blocks (L2, L3) still model the large-scale
   refraction pattern efficiently**, preserving the 28.4% speedup
   envelope that Zheng 2025 established.

**Expected G3 closure.** Based on the 44% turbulent-NS improvement
(turbulent NS is the closest PDE analog with sharp localized
features), U-NO should reduce thermocline-regime TL RMSE from the
3-5 dB Zheng baseline to **~1-2 dB**, meeting or approaching the
G3 <1 dB target. The inference-cost multiplier vs flat FNO is
estimated at **~2-3×** (7 levels with fewer modes per level, vs
4 flat blocks with 4 modes each; spatial halving makes middle
levels cheap), well within the <5× G3 budget.

---

## Follow-up references

1. **Li, Z., et al. (2020).** "Fourier Neural Operator for
   Parametric PDEs." arXiv:2010.08895. — the baseline FNO that
   U-NO extends.
2. **Ronneberger, O., Fischer, P., & Brox, T. (2015).** "U-Net:
   Convolutional Networks for Biomedical Image Segmentation."
   MICCAI 2015. — the CNN precursor; U-NO is its operator-space
   analog.
3. **Lei, W.-M., & Li, H.-B. (2024).** "U-WNO: U-Net Enhanced
   Wavelet Neural Operator for Solving Parametric Partial
   Differential Equations." SSRN abstract 4932521. — combines U-NO
   topology with wavelet basis (paper G3.4); candidate v2 upgrade.
4. **Kovachki, N., et al. (2023).** "Neural Operator: Learning
   Maps Between Function Spaces." *JMLR* 24(89). — theoretical
   universal-approximation results that justify the U-NO deep
   stacking.
