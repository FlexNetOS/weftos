# EML and Causal Collapse: Discovering the Hidden Function

**Research Document -- April 2026**

Investigates whether EML (exp(x) - ln(y)) can discover the closed-form function
governing how new evidence changes causal graph coherence ("causal collapse").

---

## 1. What IS Causal Collapse, Mathematically?

### 1.1 The Graph Laplacian and Algebraic Connectivity

The causal graph's coherence is measured by lambda_2, the algebraic connectivity --
the second-smallest eigenvalue of the graph Laplacian L = D - A, where D is the
degree matrix and A is the (weighted, symmetrized) adjacency matrix.

Our `spectral_analysis()` in `causal.rs` (line 575) computes this via sparse
Lanczos iteration: it builds the sparse adjacency, constructs degree/adjacency
vectors, and runs the Lanczos recurrence to extract lambda_2 and the Fiedler
vector from a tridiagonal approximation solved by Jacobi eigenvalue rotation
(line 1051).

Lambda_2 has well-defined semantics:
- lambda_2 = 0: graph is disconnected (at least two components)
- lambda_2 > 0: graph is connected; larger values mean stronger connectivity
- lambda_2 = n for the complete graph K_n

The Fiedler vector (the eigenvector corresponding to lambda_2) partitions the
graph: its sign pattern reveals the minimum cut.

### 1.2 What Happens When We Add an Edge?

When a new edge (u, v) with weight w is added to graph G, the Laplacian changes:

```
L' = L + w * (e_u - e_v)(e_u - e_v)^T
```

where e_u, e_v are standard basis vectors. This is a rank-1 update. The matrix
perturbation E = w * (e_u - e_v)(e_u - e_v)^T has spectral norm ||E||_2 = 2w
(since (e_u - e_v) has L2 norm sqrt(2)).

**Weyl's inequality** gives:

```
lambda_i(L) <= lambda_i(L') <= lambda_i(L) + ||E||_2
```

So lambda_2 can increase by at most 2w when adding a single edge of weight w.
It can never decrease when adding an edge (the Laplacian is positive semidefinite
and adding edges only increases it).

### 1.3 The Exact Formula for Delta-Lambda_2

For a rank-1 update, there is a sharper result from eigenvalue perturbation theory.
Let phi be the Fiedler vector (normalized eigenvector for lambda_2) of L. Then:

```
delta_lambda_2 = w * (phi[u] - phi[v])^2 + O(w^2)
```

This is the first-order perturbation formula (derivable from the Cauchy interlacing
theorem or directly from Rayleigh quotient perturbation). It states:

**The change in algebraic connectivity when adding edge (u,v) is proportional to
the squared difference of the Fiedler vector components at u and v.**

This has profound intuition:
- If u and v are on the SAME SIDE of the Fiedler cut (phi[u] and phi[v] have the
  same sign and similar magnitude), then (phi[u] - phi[v])^2 is small, and the edge
  barely changes lambda_2.
- If u and v are on OPPOSITE SIDES of the Fiedler cut (phi[u] > 0, phi[v] < 0),
  then (phi[u] - phi[v])^2 is large, and the edge dramatically increases lambda_2.
- The MAXIMUM possible delta_lambda_2 from a single edge occurs when u and v are at
  the extreme ends of the Fiedler vector -- the nodes most "pulled apart" by the
  minimum cut.

### 1.4 IS This an Elementary Function?

The first-order perturbation formula:

```
delta_lambda_2 ~ w * (phi[u] - phi[v])^2
```

This is elementary: multiplication, subtraction, squaring. Even the exact formula
(which accounts for higher-order terms involving the full eigenspectrum) is a
rational function of the eigenvalues and eigenvector components, hence elementary.

**Yes, causal collapse (at least the first-order version) is an elementary function
of the Fiedler vector components and edge weight. EML can represent it exactly.**

In fact, consider: `(phi[u] - phi[v])^2 = phi[u]^2 - 2*phi[u]*phi[v] + phi[v]^2`.
EML can represent each of these terms. Multiplication xy has EML complexity K=17
(per the Odrzywolel paper), and squaring x^2 is a special case. The full expression
is a composition of elementary operations at modest depth.

### 1.5 The Higher-Order Correction

The exact eigenvalue shift for a rank-1 perturbation is given by the secular equation:

```
1 = w * sum_{i >= 2} (phi_i[u] - phi_i[v])^2 / (lambda'_2 - lambda_i)
```

where phi_i are all eigenvectors of L and lambda_i the corresponding eigenvalues.
This is a rational function of the eigenvalues -- still elementary, but it involves
ALL eigenvalues, not just lambda_2.

For practical purposes, the first-order approximation is sufficient when w is small
relative to the spectral gap (lambda_3 - lambda_2).

---

## 2. The Conversation as an EML Composition

### 2.1 Conversation Turns as Functions

Each conversation turn adds evidence to the causal graph. In the DEMOCRITUS loop
(`democritus.rs`), each tick runs SENSE -> EMBED -> SEARCH -> UPDATE -> COMMIT:

1. **SENSE**: drain impulse queue (new evidence arrives)
2. **EMBED**: convert to vector embedding
3. **SEARCH**: find nearest neighbors in HNSW
4. **UPDATE**: add causal node + edges based on similarity
5. **COMMIT**: log the result

The UPDATE phase is where causal collapse happens: edges are added between the new
node and its similar neighbors (line 307-323 in democritus.rs), each edge classified
by type (Causes, Correlates, EvidenceFor, etc.) with a weight derived from cosine
similarity.

Each turn is a function:

```
f_t: (graph_state_t, evidence_t) -> graph_state_{t+1}
```

The coherence trajectory is: lambda_2(0), lambda_2(1), lambda_2(2), ...

### 2.2 EML Trees as Conversation Trajectories

The per-turn coherence change function is:

```
delta_lambda_2(t) = w_t * (phi_t[u_t] - phi_t[v_t])^2
```

But phi_t itself changes after each edge addition! The Fiedler vector of L' is a
function of L and the new edge. So the full composition is:

```
Turn 1: phi_0 -> delta_1 -> phi_1
Turn 2: phi_1 -> delta_2 -> phi_2
Turn 3: phi_2 -> delta_3 -> phi_3
```

Each delta_t is an EML-representable function. The composition delta_3(delta_2(delta_1(...)))
is itself an EML tree -- the output of one EML node feeds the input of the next.

**A conversation IS a linear EML chain.** More precisely, it is a dynamical system
where the state (phi, lambda_2) evolves under the iterated application of EML
functions parameterized by each turn's evidence.

### 2.3 What This Enables

If we learn the per-turn EML function (which maps graph features + evidence features
to delta_lambda_2), we can:

1. **PREDICT**: Before adding evidence, compute the predicted coherence change in O(1).
2. **PLAN**: Rank all possible evidence additions by their predicted coherence impact.
3. **DETECT CYCLES**: If the predicted delta is near zero for all available evidence,
   the conversation is stuck -- no additional evidence will resolve the ambiguity.
4. **STEER**: Choose evidence presentation order to maximize coherence convergence.

---

## 3. The Spectral Decomposition Connection

### 3.1 Phase Transitions in Graph Connectivity

In random graphs G(n, p), the algebraic connectivity undergoes a sharp phase
transition near the connectivity threshold p_c = ln(n) / n:

- For p << p_c: lambda_2 = 0 (graph is disconnected with high probability)
- At p = p_c: lambda_2 transitions from 0 to positive
- For p >> p_c: lambda_2 ~ n*p - 2*sqrt(n*p*(1-p)) (concentration result)

This is the Erdos-Renyi phase transition. In causal graphs, the analogous phenomenon
is: there exists a critical amount of evidence at which the graph "clicks" from
incoherent to coherent. Below this threshold, communities are disconnected; above it,
a single connected component dominates.

The EML coherence model (`eml_coherence.rs`) already uses Erdos-Renyi theoretical
values as training data (line 691-709):

```rust
// Erdos-Renyi G(n, p)
let lambda_2 = (nf * p - 2.0 * (nf * p * (1.0 - p)).sqrt()).max(0.0);
```

This formula IS elementary. EML can represent it exactly.

### 3.2 Eigenvalue Evolution Under Edge Insertion

As edges are added one at a time, lambda_2 follows a staircase function:
- Monotonically non-decreasing (adding edges never decreases lambda_2)
- Jumps occur when the edge bridges the minimum cut
- Smooth increases occur when the edge reinforces an already-strong region

The magnitude of each step is given by the perturbation formula from Section 1.3.
The cumulative effect is the sum of all perturbations, which is itself elementary.

### 3.3 Known Formulas for Graph Families

The existing training regime in `eml_coherence.rs` (lines 625-709) uses these
known closed forms:

| Graph Family | lambda_2 Formula | EML Representable? |
|---|---|---|
| Complete K_n | n | Yes (trivially) |
| Star S_n | 1 | Yes (constant) |
| Cycle C_n | 2(1 - cos(2*pi/n)) | Yes (trig is EML-representable) |
| Path P_n | 2(1 - cos(pi/n)) | Yes (trig is EML-representable) |
| Erdos-Renyi G(n,p) | n*p - 2*sqrt(n*p*(1-p)) | Yes (elementary) |

All of these are elementary functions. The EML master formula at depth 4 (50
parameters) should be able to interpolate between these families.

---

## 4. Composability: EML Trees as Causal Chains

### 4.1 The Core Insight

The Odrzywolel paper proves that eml(x, y) = exp(x) - ln(y) is a Sheffer operator
for continuous mathematics: any elementary function can be expressed as a binary tree
under the grammar S -> 1 | eml(S, S).

A causal chain A -> B -> C is a function composition: the evidence at A causes an
effect at B, which in turn causes an effect at C. Each link in the chain is a
function. If each link-function is elementary, the composition is elementary, and
therefore representable as an EML tree.

### 4.2 Path-Level EML Trees

For a causal path P = (v_1, v_2, ..., v_k), the total coherence contribution of this
path is determined by the product of edge weights along the path and the Fiedler
vector component differences:

```
contribution(P) = prod_{i=1}^{k-1} w_i * (phi[v_1] - phi[v_k])^2
```

The product of weights is elementary. The Fiedler component difference is elementary.
Therefore the contribution function for any single causal path is elementary and
EML-representable.

### 4.3 Graph-Level EML Forests

The full graph's lambda_2 is NOT simply the sum of path contributions -- it emerges
from the global spectral structure. However, the EML coherence model does not need
to decompose into paths. Instead, it operates on aggregate graph features:

```
GraphFeatures {
    node_count, edge_count, avg_degree,
    max_degree, min_degree, density, component_count
}
```

The mapping from these 7 features to lambda_2 is what the EML model learns. The
current model architecture (depth 4, 50 parameters, 3 output heads) produces:
- lambda_2 (algebraic connectivity)
- fiedler_norm (spread of the Fiedler vector)
- uncertainty (confidence interval width)

### 4.4 The Missing Feature: Fiedler Components

The current `GraphFeatures` struct does NOT include Fiedler vector information.
This is the critical gap. The perturbation formula delta_lambda_2 = w * (phi[u] - phi[v])^2
requires knowing phi[u] and phi[v], which are not captured by the 7 aggregate features.

To predict causal collapse, we need to augment GraphFeatures with:
- phi[u]: Fiedler component of the new evidence node (after initial placement)
- phi[v]: Fiedler component of the target node being connected to
- phi_max - phi_min: Fiedler spread (already approximated by fiedler_norm)
- phi_gap: the gap in Fiedler values at the minimum cut

---

## 5. Practical Applications

### 5.1 Cold Case Investigation

**Current workflow**: Add evidence, recompute spectral analysis (O(k*m) via Lanczos),
observe lambda_2 change. Repeat for each piece of evidence.

**EML-enabled workflow**: For each untested piece of evidence e_i:
1. Estimate where e_i would attach (which existing nodes it connects to)
2. Look up those nodes' Fiedler vector components from the last spectral analysis
3. Compute delta_lambda_2 = w * (phi[u] - phi[v])^2 in O(1)
4. Rank all untested evidence by predicted delta_lambda_2
5. Present the highest-impact evidence first

**Concrete example**: A cold case has 200 pieces of untested evidence (DNA samples,
alibis, financial records, phone logs). The causal graph currently has coherence 0.42
(weakly connected -- two main hypotheses with a thin bridge between them). The EML
model predicts that DNA sample #47 would produce delta_lambda_2 = 0.29 because the
suspect node (phi = -0.31) and the victim node (phi = +0.27) are on opposite sides
of the Fiedler cut, and connecting them via DNA evidence (w = 0.92) would bridge the
cut: 0.92 * (-0.31 - 0.27)^2 = 0.92 * 0.3364 = 0.31.

The investigator knows to prioritize the DNA test BEFORE running any experiment.

### 5.2 Robotics (Sensor Selection)

A robot has multiple sensors (lidar, camera, IMU, GPS, ultrasonic). Its world model
is a causal graph where nodes are hypotheses about the environment and edges represent
evidence from sensor readings.

**Problem**: Which sensor should the robot read next to resolve ambiguity?

**EML solution**: For each sensor s_i:
1. Predict where the sensor reading would attach to the causal graph
2. Compute predicted delta_lambda_2 from Fiedler components
3. Read the sensor with the highest predicted coherence gain

This is an information-theoretic greedy strategy: always read the sensor that
maximizes expected coherence improvement. It avoids reading redundant sensors
(those whose readings would land on the same side of the Fiedler cut as existing
evidence) and prioritizes disambiguating sensors (those that bridge the cut).

### 5.3 Conversation AI (Dialogue Steering)

In the ECC framework, each conversation turn adds a scored witness entry to the
causal graph. The `EmlCoherenceModel` (`eml_coherence.rs`) predicts lambda_2 from
graph features in O(1).

**Extension for dialogue steering**:
1. Before the user's next turn, enumerate candidate responses
2. For each candidate, simulate the causal graph update it would produce
3. Predict delta_lambda_2 for each candidate
4. Choose the response that maximizes coherence convergence (or identify if the
   conversation is in a cycle where no response significantly changes coherence)

**Detection of circular conversations**: If max(delta_lambda_2) across all candidate
responses is below a threshold, the conversation is stuck. The system can:
- Escalate to a human
- Introduce a "surprise" question that probes a different part of the Fiedler space
- Acknowledge the impasse explicitly

---

## 6. The Meta-Insight: Conversations as Dynamical Systems

### 6.1 The ECC Paper's Core Claim

The ECC paper described "conversation as the fundamental primitive" of cognitive
architecture. The DEMOCRITUS loop (`democritus.rs`) implements this: every cognitive
tick is a conversation turn that senses, embeds, searches, updates, and commits.

### 6.2 EML as the Mathematical Foundation

EML gives this claim mathematical precision:

1. **A conversation turn is an EML function**: it maps (graph_state, evidence) to a
   new graph_state via the perturbation formula.

2. **The conversation trajectory is an EML tree**: the composition of per-turn
   functions is itself an EML expression, evaluable in closed form.

3. **Coherence is a learnable elementary function**: the mapping from graph features
   to lambda_2 is elementary, and EML can discover it from data.

4. **Causal collapse is predictable**: the change in coherence from adding a specific
   piece of evidence has a known mathematical form involving the Fiedler vector.

### 6.3 The Two-Tier Architecture (DEMOCRITUS)

The `eml_coherence.rs` module already describes the two-tier pattern (line 7-13):

```
- Every tick: coherence_fast() via EML model (~0.1 us)
- When drift exceeds threshold: spectral_analysis() via Lanczos (~500 us),
  then model.record() to feed the training buffer
- Every 1000 exact samples: model.train() to refine parameters
```

This is the correct architecture. The EML model provides O(1) coherence estimates,
and the Lanczos solver provides ground truth for calibration. The key addition this
research enables is: **extend the O(1) path to predict delta_lambda_2 for hypothetical
evidence additions, not just the current graph's lambda_2.**

---

## 7. Mathematical Foundations (Formal Results)

### 7.1 Eigenvalue Perturbation for Graph Laplacians

**Theorem (First-order perturbation for rank-1 Laplacian updates)**:
Let L be the Laplacian of graph G, and let L' = L + w * b * b^T where b = e_u - e_v
and w > 0. Let lambda_2 be the algebraic connectivity of L with corresponding
normalized eigenvector phi. Then:

```
lambda_2(L') = lambda_2(L) + w * (phi^T b)^2 + O(w^2)
            = lambda_2(L) + w * (phi[u] - phi[v])^2 + O(w^2)
```

**Proof sketch**: Direct application of Rayleigh-Schrodinger perturbation theory.
The first-order correction to an eigenvalue under perturbation E is
delta_lambda = phi^T E phi. Here E = w * b * b^T, so
delta_lambda = w * (phi^T b)^2 = w * (phi[u] - phi[v])^2.

**Corollary (Monotonicity)**: Since (phi[u] - phi[v])^2 >= 0 and w > 0,
adding an edge never decreases lambda_2. Equality holds only when phi[u] = phi[v]
(the edge endpoints are in the same Fiedler partition at the same "depth").

### 7.2 Weyl's Inequality (Tight Bounds)

**Theorem (Weyl, 1912)**: For Hermitian matrices A, B:

```
lambda_i(A) + lambda_1(B) <= lambda_i(A + B) <= lambda_i(A) + lambda_n(B)
```

For our rank-1 perturbation B = w * b * b^T: lambda_1(B) = 0 (since B is rank 1,
it has n-1 zero eigenvalues) and lambda_n(B) = w * ||b||^2 = 2w.

Therefore: lambda_2(L) <= lambda_2(L') <= lambda_2(L) + 2w.

This gives a tight upper bound on the maximum coherence change from a single edge.

### 7.3 The Courant-Fischer Minimax Characterization

**Theorem (Courant-Fischer)**: The k-th smallest eigenvalue of a symmetric matrix A is:

```
lambda_k = min_{dim(V)=k} max_{x in V, ||x||=1} x^T A x
```

For the Laplacian, lambda_2 = min over 2-dimensional subspaces containing the
constant vector, of the maximum Rayleigh quotient. This characterization is what
makes the Lanczos algorithm work: it builds a Krylov subspace that approximates the
optimal subspace.

### 7.4 Percolation Threshold

For random graphs G(n, p), the connectivity transition occurs at:

```
p_c = ln(n) / n
```

Below p_c, the graph is almost surely disconnected (lambda_2 = 0).
Above p_c, the graph is almost surely connected (lambda_2 > 0).

In causal graphs, this translates to: there is a critical evidence density at which
the graph transitions from "multiple disconnected hypotheses" to "single coherent
explanation." The EML model should capture this phase transition as a steep sigmoid
in the density-to-lambda_2 mapping.

### 7.5 Pearl's Do-Calculus Connection

Judea Pearl's interventional calculus asks: "What happens if we DO X?" In our
framework, "doing X" means adding evidence X to the causal graph. The causal effect
of adding evidence is the change in coherence (delta_lambda_2).

Pearl's adjustment formula identifies which variables need to be controlled to
estimate a causal effect. In spectral terms, this corresponds to identifying which
Fiedler vector components are relevant to the perturbation -- exactly the
(phi[u] - phi[v])^2 term.

The connection is: **EML-based causal collapse prediction is a spectral implementation
of Pearl's do-calculus for graph coherence.**

---

## 8. Implementation Recommendations

### 8.1 New Feature: Delta-Lambda_2 Prediction

**Goal**: Predict how adding a specific edge (u, v, w) would change lambda_2, without
recomputing spectral analysis.

**Input features** (extend GraphFeatures or create EdgeImpactFeatures):

```rust
struct EdgeImpactFeatures {
    // Current graph state
    current_lambda2: f64,
    fiedler_u: f64,        // Fiedler component at source
    fiedler_v: f64,        // Fiedler component at target
    edge_weight: f64,      // Proposed edge weight
    // Graph context
    degree_u: f64,         // Current degree of source
    degree_v: f64,         // Current degree of target
    component_u: f64,      // Connected component ID of source (0 if same as target, 1 if different)
    component_v: f64,      // Connected component ID of target
    spectral_gap: f64,     // lambda_3 - lambda_2 (determines perturbation accuracy)
}
```

**Architecture**: An EmlModel with depth 3, 9 input features, 1 output head
(31 parameters). Small enough for fast convergence, expressive enough for the
perturbation formula.

**Training data generation**:
1. During normal DEMOCRITUS operation, before each edge addition:
   - Record the current Fiedler vector components at the endpoints
   - Record the pre-addition lambda_2
2. After each edge addition, record the post-addition lambda_2
3. Compute delta_lambda_2 = post - pre
4. Feed (EdgeImpactFeatures, delta_lambda_2) to the EML model

**Ground truth**: The Lanczos spectral analysis already runs periodically for
calibration. We can piggyback on this to collect Fiedler vectors and lambda_2
values for training.

### 8.2 Gap Analysis API

**New function**: `rank_evidence_by_impact()`

```rust
impl CausalGraph {
    /// Rank candidate evidence additions by predicted coherence impact.
    ///
    /// Returns candidates sorted by predicted delta_lambda_2 descending.
    /// Each candidate is (source_node, target_node, predicted_delta, explanation).
    pub fn rank_evidence_by_impact(
        &self,
        candidates: &[(NodeId, NodeId, f32)],  // (source, target, weight)
        fiedler: &[f64],                       // Current Fiedler vector
        lambda_2: f64,                         // Current lambda_2
        model: &EmlModel,                      // Trained delta-lambda_2 predictor
    ) -> Vec<EvidenceRanking> { ... }
}
```

This gives cold case investigators, robotics planners, and dialogue systems a
single API call to determine which evidence to pursue next.

### 8.3 Conversation Cycle Detection

**New function**: `detect_conversation_cycle()`

```rust
/// Detect if recent coherence changes indicate a circular conversation.
///
/// Returns true if the last `window` turns have produced cumulative
/// delta_lambda_2 below `threshold`, indicating the conversation is
/// not converging.
pub fn detect_conversation_cycle(
    coherence_history: &[f64],
    window: usize,
    threshold: f64,
) -> bool {
    if coherence_history.len() < window {
        return false;
    }
    let recent = &coherence_history[coherence_history.len() - window..];
    let total_change = (recent.last().unwrap() - recent.first().unwrap()).abs();
    total_change < threshold
}
```

### 8.4 Integration with DEMOCRITUS Two-Tier Architecture

Modify the DEMOCRITUS tick to include delta-lambda_2 prediction:

```
SENSE -> EMBED -> SEARCH -> PREDICT_IMPACT -> UPDATE -> COMMIT
                              ^-- NEW PHASE
```

The PREDICT_IMPACT phase uses the trained EML model to predict each candidate edge's
coherence impact BEFORE adding it. This enables:
- Logging predicted vs. actual delta_lambda_2 for model training
- Filtering low-impact edges (skip edges that won't change coherence)
- Prioritizing high-impact edges when under time budget

### 8.5 Concrete Training Pipeline

```
Phase 1: Data Collection (Sprint 17+)
  - Instrument DEMOCRITUS UPDATE phase to record Fiedler components before edge insertion
  - Compute and log delta_lambda_2 for each edge added
  - Accumulate 1000+ (EdgeImpactFeatures, delta_lambda_2) pairs

Phase 2: Model Training
  - Create EmlModel::new(3, 9, 1) for delta-lambda_2 prediction
  - Train on collected data using existing coordinate descent
  - Validate: MSE between predicted and actual delta_lambda_2

Phase 3: Integration
  - Add rank_evidence_by_impact() API
  - Add PREDICT_IMPACT phase to DEMOCRITUS
  - Add conversation cycle detection

Phase 4: Evaluation
  - Cold case scenario: does evidence ranking match investigator intuition?
  - Robotics scenario: does sensor selection reduce ambiguity faster?
  - Dialogue scenario: does cycle detection identify stuck conversations?
```

### 8.6 Mathematical Validation

The first thing to validate is whether the perturbation formula alone (without EML)
gives good predictions:

```rust
fn predict_delta_lambda2_analytical(
    fiedler_u: f64,
    fiedler_v: f64,
    edge_weight: f64,
) -> f64 {
    edge_weight as f64 * (fiedler_u - fiedler_v).powi(2)
}
```

If this analytical formula matches the empirically observed delta_lambda_2 to within
5% for single-edge additions, then EML training is unnecessary for first-order
predictions -- the closed form is already known. EML becomes valuable only when:

1. Multiple edges are added simultaneously (per-tick batching in DEMOCRITUS)
2. The Fiedler vector itself changes significantly between spectral analyses
3. Higher-order effects (eigenvalue crossing, spectral gap closure) matter

In these cases, the EML model can learn corrections to the first-order formula
that capture the nonlinear dynamics.

---

## 9. Open Questions

1. **Fiedler vector staleness**: The perturbation formula uses the CURRENT Fiedler
   vector, but phi changes after each edge addition. How many edges can we add before
   phi becomes too stale for accurate predictions? The answer depends on the spectral
   gap: if lambda_3 - lambda_2 is large, phi is robust to perturbation.

2. **Multi-edge perturbation**: When DEMOCRITUS adds K edges in a single tick, the
   total perturbation is the sum of K rank-1 updates. The first-order prediction
   sums the individual deltas, but cross-terms (how edge i affects the Fiedler vector
   seen by edge j) introduce second-order corrections. Can EML learn these corrections?

3. **Phase transition prediction**: Can the EML model predict WHEN the graph will
   transition from disconnected to connected (lambda_2 jumping from 0 to positive)?
   This is the most dramatic form of causal collapse and the most valuable to predict.

4. **Contradictions**: Our causal graph includes `Contradicts` and `Inhibits` edge
   types. These have negative semantic weight but positive graph-theoretic weight
   (they still connect nodes). The perturbation formula treats them like any other
   edge, but semantically they should DECREASE coherence. This requires a separate
   treatment: semantic coherence vs. graph connectivity.

5. **Depth requirements**: The perturbation formula is simple enough for EML depth 2.
   But the full mapping (graph_features + evidence_features -> delta_lambda_2) may
   require depth 3 or 4 if nonlinear interactions between features are important.
   The existing depth-4 coherence model with 50 parameters provides a reasonable
   starting point.

---

## 10. Summary

The hidden function driving causal collapse is **not** hidden at all -- it is the
eigenvalue perturbation formula from spectral graph theory:

```
delta_lambda_2 = w * (phi[u] - phi[v])^2
```

This formula is elementary, and therefore EML-representable. The EML framework adds
value in three ways:

1. **Learning corrections**: When the first-order formula is insufficient (multi-edge
   batches, stale Fiedler vectors, spectral gap closure), EML can learn the nonlinear
   corrections from data.

2. **Composability**: The sequence of per-turn coherence changes IS an EML tree. This
   gives a formal mathematical structure to conversations, enabling trajectory
   prediction and cycle detection.

3. **O(1) prediction**: Combined with the existing two-tier DEMOCRITUS architecture,
   EML models can predict both absolute coherence (lambda_2 from graph features, which
   already works) and relative coherence change (delta_lambda_2 from edge features,
   which this research proposes).

The implementation path is clear: augment the feature set with Fiedler components,
train a delta-lambda_2 predictor, and integrate it into the DEMOCRITUS tick loop.
The analytical formula provides a strong baseline; EML provides learned refinements.
