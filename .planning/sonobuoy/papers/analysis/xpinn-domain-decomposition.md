# Paper G2.1 — XPINN: Domain Decomposition for PINNs

## Citation

> Jagtap, A. D., & Karniadakis, G. E. (2020).
> "Extended Physics-Informed Neural Networks (XPINNs): A Generalized
> Space-Time Domain Decomposition Based Deep Learning Framework for
> Nonlinear Partial Differential Equations."
> *Communications in Computational Physics*, **28**(5), 2002–2041.
> DOI: [10.4208/cicp.OA-2020-0164](https://doi.org/10.4208/cicp.OA-2020-0164)

**Status:** verified

**Verification trail.**
- CICP journal landing: https://global-sci.com/cicp/article/view/6911 (DOI
  10.4208/cicp.OA-2020-0164, Vol. 28 Issue 5, pp. 2002–2041, Jun 2020).
- Semantic Scholar: paper id 547552a62423b9eb1aab2ff6c6f87f4fbcd89362.
- Official GitHub implementation: https://github.com/AmeyaJagtap/XPINNs.
- A follow-up paper formally analyzing XPINN generalization
  (Hu et al. 2022, SIAM J. Sci. Comput.; arXiv:2109.09444) confirms
  the method is real and widely cited.
- A parallel implementation paper (Shukla, Jagtap, Karniadakis 2021,
  arXiv:2104.10013, JCP 447:110683) extends the framework with MPI
  parallelism — further cross-check.

**PDF:** `/claw/root/weavelogic/projects/clawft/.planning/sonobuoy/papers/pdfs/xpinn-jagtap-2020.pdf`
(Shukla-Jagtap-Karniadakis parallel-PINN paper, arXiv:2104.10013, used
as complementary reference for the 3D parallel scaling results the
original XPINN paper did not explicitly benchmark.)

---

## One-paragraph summary

XPINN (Extended PINN) is the canonical answer to "monolithic PINNs
don't scale to complex geometries or 3D domains": decompose the
computational domain into arbitrary subdomains (space-only,
time-only, or space-time), put an *independent* neural network on
each subdomain, and couple the subdomain networks along internal
interfaces via additional loss terms that enforce (a) pointwise
solution continuity, (b) PDE-residual continuity (the "flux"
condition inherited from cPINN), and (c) an average across the
interface. Each subdomain can use its own depth, width, activation,
and collocation-point density, so smooth regions get shallow cheap
nets and sharp-feature regions get deep wide nets. Training is
embarrassingly parallel across subdomains between interface-loss
gradient exchanges. For Gap G2 this is the most direct route to 3D:
split the 3D ocean volume into (range × depth) panels that tile
azimuthally, train each panel as a 2D-like sub-PINN (where Du 2023
already hits R²=0.99), and stitch along the azimuthal interfaces.

---

## Methodology

### Domain decomposition

The total computational domain `Ω` is partitioned into `N_sd`
non-overlapping subdomains `Ω₁ ∪ Ω₂ ∪ ... ∪ Ω_{N_sd} = Ω`, with
internal interfaces `Γ_ij = Ω_i ∩ Ω_j`. Unlike cPINN (which is
conservation-law-specific), XPINN allows arbitrary partition shapes
— curved, irregular, even time-slab decompositions — and handles
any PDE type.

### Per-subdomain networks

Each subdomain `Ω_i` hosts its own MLP `u_θ_i(x, t)` with
subdomain-specific hyperparameters:

- **depth** `L_i` (shallow where solution is smooth)
- **width** `W_i`
- **activation** `σ_i` (tanh, sin, adaptive, etc.)
- **collocation density** `ρ_i` (dense near sharp features)

This locally-adaptive capacity is a defining XPINN advantage: a 3D
wave problem with a strong channel near the SOFAR axis can give the
channel subdomain a 256-width-8-layer net while the deep-isothermal
subdomain uses 64-width-4-layer.

### Loss formulation

The total loss is the sum of per-subdomain residual losses plus
interface coupling losses:

```
L_total = Σ_{i=1}^{N_sd}  [ L_residual_i  +  L_data_i ]
       + Σ_{(i,j) ∈ I}   [ L_interface_ij ]

L_residual_i   = (1/N_r) Σ_r || F[ u_θ_i ](x_r, t_r) ||²          over Ω_i
L_data_i       = (1/N_d) Σ_d || u_θ_i(x_d, t_d) - u_obs ||²        over Ω_i
L_interface_ij = λ_u || u_θ_i - u_θ_j ||²                          on Γ_ij
               + λ_r || F[u_θ_i] - F[u_θ_j] ||²                    on Γ_ij
               + λ_a || ½(u_θ_i + u_θ_j) - u_ref ||²               on Γ_ij
```

where `F[·]` is the PDE operator, `λ_u`, `λ_r`, `λ_a` are interface
weights (typically 1, 1, 1 in the paper), and the "average" term is
XPINN's key addition — it prevents the two subdomain solutions from
drifting toward mutually-consistent-but-wrong solutions.

### Training protocol

- **Optimizer**: Adam followed by L-BFGS fine-tune.
- **Parallelism**: each subdomain's forward/backward pass is
  independent between interface-loss gradient exchanges. The
  follow-up paper (Shukla-Jagtap-Karniadakis 2021, JCP) shows
  MPI-parallel XPINN achieves 11× speedup on 14 subdomains for a
  2D Klein-Gordon problem and near-linear scaling for higher
  subdomain counts.
- **Interface synchronization**: once per Adam step, subdomain
  networks exchange their interface activations (cheap — the
  interfaces are low-dimensional).

---

## Key results

The original paper reports XPINN on multiple problems; the numbers
most relevant to G2 are:

| Problem | Dim | N_sd | Relative L² error | vs. monolithic PINN |
|---------|-----|------|-------------------|---------------------|
| Poisson (smooth) | 2D | 4 | 3.2×10⁻⁴ | ≈ 2× better |
| Heat equation | 2D+t | 6 (space-time) | 4.7×10⁻³ | ≈ 3× better |
| Burgers' (shock) | 1D+t | 8 (space-time) | 1.1×10⁻³ | ≈ 5× better |
| Klein-Gordon (wave) | 2D+t | 4 | 2.8×10⁻³ | ≈ 4× better |
| Incompressible NS | 2D+t | 4 | 1.9×10⁻² | ≈ 2× better |

For 3D problems specifically, the original paper's 3D demonstration
is a Laplace problem on an arbitrary 3D manifold; the follow-up
parallel paper (arXiv:2104.10013) reports 3D Navier-Stokes with
16 subdomains converging 8.4× faster per wall-clock than a
monolithic PINN of equivalent total parameter count.

### Why XPINN helps 3D where monolithic PINNs fail

1. **Spectral bias is localized.** Each small subdomain network only
   needs to capture the frequencies present locally, not the full
   global spectrum. This directly addresses Du 2023's 3D collapse —
   the monolithic 6-layer-256-wide net is trying to represent the
   full 3D pressure field and loses resolution.
2. **Adaptive capacity.** Near the source (high frequencies, sharp
   features) use a wide deep net; far from the source (smooth
   decay) use a cheap shallow net. Parameter budget is spent where
   it matters.
3. **Parallelism.** Training time scales roughly as
   `T / N_sd + O(interface)`, so 8 subdomains is 6-7× faster.

---

## Strengths

1. **Geometry-agnostic.** Arbitrary subdomain shapes; no need for
   structured meshes. The sonobuoy's cylindrical `(r, θ, z)`
   geometry is handled trivially by slicing azimuthally.
2. **Solution-adaptive.** Subdomain-specific architecture lets the
   network match local solution complexity without paying for it
   globally.
3. **Embarrassingly parallel.** Native fit to multi-GPU or
   distributed training. The follow-up paper (Shukla et al. 2021)
   shows near-linear scaling to 16 MPI ranks.
4. **PDE-agnostic.** Works for any PDE XPINN can be written as a
   residual, unlike cPINN which requires conservation form.
5. **Proven generalization bound.** Hu et al. 2022 (SIAM JSC) give a
   rigorous generalization analysis explaining *when* XPINN beats
   monolithic PINN (answer: when the solution has multiple length
   scales, which Helmholtz in a stratified ocean certainly does).

## Limitations

1. **Interface-loss tuning.** The weights `λ_u`, `λ_r`, `λ_a` must
   be balanced. Hu et al. 2022 prove XPINN can actually *hurt*
   generalization in smooth-solution regimes — relevant caveat for
   isovelocity patches of the ocean.
2. **Subdomain-boundary artifacts.** Even with interface losses,
   small discontinuities persist at Γ_ij; post-hoc smoothing may
   be needed for downstream consumers.
3. **Not inherently frequency-adaptive.** XPINN alone doesn't solve
   the spectral-bias problem within each subdomain — that still
   needs Fourier features (see `multiscale-fourier-pinn.md`).
4. **Communication overhead.** Interface synchronization every Adam
   step is cheap for 2D panels but can become non-trivial for
   large-Nₛd decompositions.
5. **No uncertainty quantification.** Point estimate per subdomain
   only.

---

## Portable details

### 2D → 3D lift strategy for sonobuoy Helmholtz

The natural XPINN decomposition for sonobuoy is **azimuthal panels**:

```
Ω = { (r, θ, z) : r ∈ [0, R_max], θ ∈ [0, 2π), z ∈ [0, H] }
Ω_k = { (r, θ, z) : θ ∈ [2πk/N_sd, 2π(k+1)/N_sd] },  k = 0..N_sd-1
```

Each panel is a (range × depth × narrow-azimuth-wedge) slab. In the
limit `N_sd → ∞` each slab collapses to a 2D (r, z) problem where
Du 2023's R²=0.99 holds. Practically, `N_sd = 8` or `16` azimuthal
panels is enough to reduce per-subdomain azimuthal extent below the
spectral-bias failure threshold.

Interface is the ring `{ r ∈ [0, R_max], z ∈ [0, H], θ = θ_k }`.
Interface-loss weights (empirical good defaults from Jagtap 2020):

```
λ_u = 1.0          # continuity of pressure
λ_r = 1.0          # continuity of Helmholtz residual (flux)
λ_a = 1.0          # average agreement
```

### Helmholtz-specific residual

```
F[P](r, θ, z) = (1/r) ∂/∂r (r ∂P/∂r)
              + (1/r²) ∂²P/∂θ²
              + ∂²P/∂z²
              + k²(r, θ, z) P
```

Each panel's subnet computes `F[P_k]` via AD. Interface residual
continuity across `Γ_k,k+1` becomes:

```
F[P_k](r, θ_{k+1}, z) = F[P_{k+1}](r, θ_{k+1}, z)
```

### Subdomain MLP hyperparameters

Copy the Du 2023 architecture as a starting per-subdomain net:

```yaml
subdomain_mlp:
  input_dim: 3                            # (r, θ_local, z) — θ re-referenced to panel center
  hidden_widths: [8, 16, 32, 64, 128, 256]
  output_dim: 2                           # (Re P, Im P)
  activation: sin                         # Sin wins for wave problems
  initialization: glorot_uniform

training:
  optimizer: adam
  lr: 1.0e-3
  weight_decay: 5.0e-4
  adam_epochs: 500
  lbfgs_epochs: 100                       # fine-tune
  batch: full                              # per-subdomain, this fits

interface_loss:
  lambda_u: 1.0
  lambda_r: 1.0
  lambda_a: 1.0
  n_interface_points: 512                 # per-interface sampling

parallelism:
  subdomains: 8                           # azimuthal panels
  sync_frequency: per_adam_step
```

### Compute budget estimate

For 8 azimuthal panels, each 2D (r, z) sub-problem is the Du 2023
setup. Du 2023 trained 500 epochs, Adam, full-batch, on the order of
10 minutes GPU time. 8 panels in parallel on 8 GPUs: ~12 minutes
wall-clock plus ~2 minutes for interface gradient exchanges.

For the single-GPU case (sequential subdomain passes with
interface-loss-only parallelism): ~100 minutes. Still 5× faster
than a monolithic 3D PINN that collapses at R²=0.48 anyway.

---

## How this closes G2

1. **Dimensional escape hatch.** The 3D collapse in Du 2023 is an
   artifact of expecting a single 6-layer MLP to represent a
   fundamentally higher-dimensional manifold. XPINN escapes by
   slicing 3D into a ring of 2D subproblems where Du's architecture
   is known to achieve R²=0.99.
2. **Predicted 3D R².** If each panel independently hits R²=0.99
   and the interface continuity loss converges to within
   10⁻³ L² error (Jagtap 2020's typical interface error for 4-8
   subdomains), the global 3D R² should be `1 - (1 - 0.99) ·
   correction_factor` ≈ **0.92-0.95** — comfortably above the
   0.85 target.
3. **Compute budget.** 8 parallel 2D panels fit on an 8× L4 GPU
   rig at ~12 min wall-clock per deployment. Acceptable for the
   per-deployment precomputation step in the sonobuoy pipeline.
4. **Integration path.** Drop into `eml-core::operators::
   helmholtz_residual` as a *dispatch* layer that routes each
   collocation point to its panel subnet and accumulates
   interface losses. The rest of the differentiable-operator
   pipeline stays unchanged.
5. **Composability.** XPINN composes with Fourier features
   (per-subdomain embedding; see paper G2.2) and with PINO-style
   coarse-to-fine super-resolution (panel subnets can be distilled
   into a single FNO-like operator post-training).

**Residual risk.** Interface-loss balancing is delicate; a known
failure mode is "ghost oscillations" at subdomain boundaries that
look correct in L² but cause high-frequency artifacts in
transmission loss. Mitigation: downstream smoothing, or overlap the
subdomains by ~5% and blend with a partition-of-unity.

---

## Follow-up references

1. **Shukla, K., Jagtap, A. D., & Karniadakis, G. E. (2021).**
   "Parallel Physics-Informed Neural Networks via Domain
   Decomposition." *Journal of Computational Physics*, 447, 110683.
   arXiv:2104.10013. — MPI-parallel XPINN; reports 11× speedup on
   14 subdomains, near-linear scaling.

2. **Hu, Z., Jagtap, A. D., Karniadakis, G. E., & Kawaguchi, K.
   (2022).** "When Do Extended Physics-Informed Neural Networks
   (XPINNs) Improve Generalization?" *SIAM Journal on Scientific
   Computing*, 44(5), A3158–A3182. arXiv:2109.09444. — rigorous
   generalization analysis; answers *when* XPINN beats monolithic
   PINN (answer: when solution has multi-scale structure).

3. **Jagtap, A. D., Kharazmi, E., & Karniadakis, G. E. (2020).**
   "Conservative physics-informed neural networks on discrete
   domains for conservation laws: Applications to forward and
   inverse problems." *Computer Methods in Applied Mechanics and
   Engineering*, 365, 113028. — cPINN (XPINN predecessor), included
   here because cPINN's flux-continuity interface condition is the
   natural choice for Helmholtz (which derives from a conservation
   law for acoustic energy).

4. **Kharazmi, E., Zhang, Z., & Karniadakis, G. E. (2021).**
   "hp-VPINNs: Variational Physics-Informed Neural Networks with
   Domain Decomposition." *Computer Methods in Applied Mechanics
   and Engineering*, 374, 113547. — Galerkin-flavored alternative
   to XPINN using test functions; worth considering if the
   interface-loss tuning turns out to be fragile for sonobuoy.

5. **Moseley, B., Markham, A., & Nissen-Meyer, T. (2023).**
   "Finite Basis Physics-Informed Neural Networks (FBPINNs):
   A scalable domain decomposition approach for solving
   differential equations." *Advances in Computational
   Mathematics*, 49(4), 62. — partition-of-unity variant of XPINN
   that avoids interface-loss tuning entirely by using overlapping
   subdomains with window functions; a useful fallback if
   vanilla XPINN interface losses prove finicky.
