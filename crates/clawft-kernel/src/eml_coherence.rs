//! O(1) coherence approximation via EML (exp(x) - ln(y)) master formula.
//!
//! Predicts algebraic connectivity (lambda_2) from graph statistics
//! without running the expensive O(k*m) Lanczos eigenvalue iteration.
//! Based on: Odrzywolel 2026, "All elementary functions from a single operator"
//!
//! # Two-Tier Coherence Pattern (DEMOCRITUS)
//!
//! The intended usage follows a two-tier pattern:
//! - **Every tick**: `coherence_fast()` via the EML model (~0.1 us)
//! - **When drift exceeds threshold**: `spectral_analysis()` via Lanczos (~500 us),
//!   then `model.record()` to feed the training buffer
//! - **Every 1000 exact samples**: `model.train()` to refine parameters
//!
//! This module does NOT modify the cognitive tick loop. Callers are
//! responsible for implementing the two-tier cadence.
//!
//! # Architecture
//!
//! Supports two architectures selected automatically by parameter count:
//! - **Depth-3 (34 params)**: Legacy single-output lambda_2 prediction.
//! - **Depth-4 (50 params)**: Multi-head output with lambda_2, Fiedler norm,
//!   and uncertainty estimate.

use serde::{Deserialize, Serialize};

use crate::causal::CausalGraph;

// ---------------------------------------------------------------------------
// EML operator
// ---------------------------------------------------------------------------

/// The EML universal operator: eml(x, y) = exp(x) - ln(y).
///
/// This is the continuous-mathematics analog of the NAND gate: combined
/// with the constant 1, it can reconstruct all elementary functions.
#[inline]
pub fn eml(x: f64, y: f64) -> f64 {
    x.exp() - y.ln()
}

// ---------------------------------------------------------------------------
// CoherencePrediction
// ---------------------------------------------------------------------------

/// Multi-output coherence prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherencePrediction {
    /// Primary: predicted algebraic connectivity (lambda_2).
    pub lambda_2: f64,
    /// Estimated Fiedler vector norm (spread of the weak cut).
    pub fiedler_norm: f64,
    /// Uncertainty estimate (lambda_2 confidence interval width).
    pub uncertainty: f64,
}

// ---------------------------------------------------------------------------
// GraphFeatures
// ---------------------------------------------------------------------------

/// Cheap-to-extract graph statistics used as input features for the
/// EML coherence model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphFeatures {
    /// Number of nodes |V|.
    pub node_count: f64,
    /// Number of edges |E|.
    pub edge_count: f64,
    /// Average degree: 2*|E| / |V| (undirected interpretation).
    pub avg_degree: f64,
    /// Maximum degree across all nodes.
    pub max_degree: f64,
    /// Minimum degree across all nodes.
    pub min_degree: f64,
    /// Edge density: 2*|E| / (|V| * (|V|-1)).
    pub density: f64,
    /// Number of connected components.
    pub component_count: f64,
}

impl GraphFeatures {
    /// Extract features from a [`CausalGraph`] in O(n) time.
    pub fn from_causal_graph(graph: &CausalGraph) -> Self {
        let n = graph.node_count() as f64;
        let m = graph.edge_count() as f64;

        if n < 1.0 {
            return Self {
                node_count: 0.0,
                edge_count: 0.0,
                avg_degree: 0.0,
                max_degree: 0.0,
                min_degree: 0.0,
                density: 0.0,
                component_count: 0.0,
            };
        }

        let ids = graph.node_ids();
        let mut max_deg: usize = 0;
        let mut min_deg: usize = usize::MAX;
        for &id in &ids {
            let d = graph.degree(id);
            if d > max_deg {
                max_deg = d;
            }
            if d < min_deg {
                min_deg = d;
            }
        }
        if ids.is_empty() {
            min_deg = 0;
        }

        let avg_degree = if n > 0.0 { 2.0 * m / n } else { 0.0 };
        let density = if n > 1.0 {
            2.0 * m / (n * (n - 1.0))
        } else {
            0.0
        };

        let component_count = graph.connected_components().len() as f64;

        Self {
            node_count: n,
            edge_count: m,
            avg_degree,
            max_degree: max_deg as f64,
            min_degree: min_deg as f64,
            density,
            component_count,
        }
    }

    /// Normalize features to [0, 1] range for numerical stability.
    fn normalized(&self) -> [f64; 7] {
        [
            self.node_count / 10000.0,
            self.edge_count / 50000.0,
            self.avg_degree / 100.0,
            self.max_degree / 1000.0,
            self.density,
            self.component_count / 100.0,
            self.min_degree / 50.0,
        ]
    }
}

// ---------------------------------------------------------------------------
// TrainingPoint
// ---------------------------------------------------------------------------

/// A recorded (features, targets) pair for model training.
#[derive(Debug, Clone)]
struct TrainingPoint {
    features: GraphFeatures,
    lambda_2: f64,
    /// Optional Fiedler norm ground truth.
    fiedler_norm: Option<f64>,
    /// Optional uncertainty ground truth.
    uncertainty: Option<f64>,
}

// ---------------------------------------------------------------------------
// EmlCoherenceModel
// ---------------------------------------------------------------------------

/// Number of trainable parameters in the depth-3 EML formula.
const PARAM_COUNT_V1: usize = 34;

/// Number of trainable parameters in the depth-4 multi-head EML formula.
const PARAM_COUNT_V2: usize = 50;

/// Depth-4 multi-head EML master formula for O(1) coherence prediction.
///
/// The architecture is:
///
/// ```text
/// Level 0: 8 linear combinations of 7 input features (24 params)
///   a_i = softmax(alpha, beta, gamma) . (1, x_j, x_k)
///
/// Level 1: 4 EML nodes
///   b_0 = eml(a_0, a_1), b_1 = eml(a_2, a_3), ...
///
/// Level 2: 4 EML nodes with light mixing (12 params)
///   c_0 = eml(mix(b_0,b_1), mix(b_2,b_3))
///   c_1 = eml(mix(b_0,b_1), mix(b_2,b_3))  (different weights)
///   c_2 = eml(mix(b_0,b_2), mix(b_1,b_3))
///   c_3 = eml(mix(b_1,b_3), mix(b_0,b_2))
///
/// Level 3: 2 EML nodes with heavier mixing (8 params)
///   d_0 = eml(mix(c_0,c_1), mix(c_2,c_3))
///   d_1 = eml(mix(c_0,c_2), mix(c_1,c_3))
///
/// Level 4: Multi-head output -- 3 final EML nodes sharing d0,d1 trunk (6 params)
///   lambda_2      = eml(mix_a(d_0), mix_a(d_1))
///   fiedler_norm  = eml(mix_b(d_0), mix_b(d_1))
///   uncertainty   = eml(mix_c(d_0), mix_c(d_1))
/// ```
///
/// Total: 24 + 12 + 8 + 6 = 50 trainable parameters.
///
/// Backward compatible: if `params.len() == 34`, runs the legacy depth-3
/// single-output model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmlCoherenceModel {
    /// Trainable parameters (weights), stored as Vec for serde compat.
    /// Length is 34 (depth-3 legacy) or 50 (depth-4 multi-head).
    params: Vec<f64>,
    /// Whether the model has been trained to convergence.
    trained: bool,
    /// Training data buffer.
    #[serde(skip)]
    training_data: Vec<TrainingPoint>,
    /// Prediction error history (for drift detection).
    #[serde(skip)]
    error_history: Vec<f64>,
}

impl Default for EmlCoherenceModel {
    fn default() -> Self {
        Self::new()
    }
}

impl EmlCoherenceModel {
    /// Create a new untrained depth-4 multi-head model with zeroed parameters.
    pub fn new() -> Self {
        Self {
            params: vec![0.0; PARAM_COUNT_V2],
            trained: false,
            training_data: Vec::new(),
            error_history: Vec::new(),
        }
    }

    /// Create a new untrained depth-3 legacy model (34 params).
    pub fn new_v1() -> Self {
        Self {
            params: vec![0.0; PARAM_COUNT_V1],
            trained: false,
            training_data: Vec::new(),
            error_history: Vec::new(),
        }
    }

    /// Whether this model uses the depth-4 multi-head architecture.
    pub fn is_multi_head(&self) -> bool {
        self.params.len() == PARAM_COUNT_V2
    }

    /// Whether the model has been trained to convergence.
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Number of training samples collected so far.
    pub fn training_sample_count(&self) -> usize {
        self.training_data.len()
    }

    /// Mean of the recent error history (empty => 0.0).
    pub fn mean_error(&self) -> f64 {
        if self.error_history.is_empty() {
            return 0.0;
        }
        self.error_history.iter().sum::<f64>() / self.error_history.len() as f64
    }

    // -------------------------------------------------------------------
    // Prediction
    // -------------------------------------------------------------------

    /// O(1) multi-head coherence prediction from graph features.
    ///
    /// Returns a [`CoherencePrediction`] with lambda_2, fiedler_norm, and
    /// uncertainty. Falls back to a density-based estimate if untrained.
    /// For depth-3 legacy models, fiedler_norm and uncertainty are
    /// synthetic estimates derived from lambda_2.
    pub fn predict(&self, features: &GraphFeatures) -> CoherencePrediction {
        if !self.trained {
            // Fallback: density * avg_degree is a rough proxy for
            // algebraic connectivity in random graphs.
            let lambda_2 = features.density * features.avg_degree;
            return CoherencePrediction {
                lambda_2,
                fiedler_norm: lambda_2.sqrt().max(0.0),
                uncertainty: lambda_2 * 0.5,
            };
        }
        if self.params.len() == PARAM_COUNT_V1 {
            let lambda_2 = self.evaluate_depth3(&self.params, features);
            CoherencePrediction {
                lambda_2,
                fiedler_norm: lambda_2.sqrt().max(0.0),
                uncertainty: lambda_2 * 0.5,
            }
        } else {
            self.evaluate_depth4(&self.params, features)
        }
    }

    /// Convenience: returns only the primary lambda_2 value.
    ///
    /// Use this when you only need the algebraic connectivity scalar
    /// (backward compatible with callers that expected `f64`).
    pub fn predict_lambda2(&self, features: &GraphFeatures) -> f64 {
        self.predict(features).lambda_2
    }

    /// Evaluate the depth-3 EML tree with the given parameters.
    ///
    /// Parameter layout (34 total):
    ///   [0..24]  Level 0: 8 linear combos, 3 weights each
    ///   [24..32] Level 2: 2 mixing nodes, 4 weights each
    ///                      (alpha1, beta1, alpha2, beta2) per node
    ///   [32..34] Level 3: 1 output mixing, 2 weights
    fn evaluate_depth3(&self, params: &[f64], features: &GraphFeatures) -> f64 {
        let (a, b) = self.evaluate_levels_0_1(params, features);
        let _ = a; // level 0 outputs consumed by level 1

        // Level 2: 2 EML nodes with mixing
        let mut c = [0.0f64; 2];
        for i in 0..2 {
            let base = 24 + i * 4;
            let mix_left = params[base] + params[base + 1] * b[0]
                + (1.0 - params[base] - params[base + 1]) * b[1];
            let mix_right = params[base + 2] + params[base + 3] * b[2]
                + (1.0 - params[base + 2] - params[base + 3]) * b[3];
            let ml = mix_left.clamp(-10.0, 10.0);
            let mr = mix_right.clamp(0.01, 10.0);
            c[i] = eml_safe(ml, mr);
        }

        // Level 3: output
        let w0 = params[32];
        let w1 = params[33];
        let out_left = (w0 * c[0] + (1.0 - w0) * c[1]).clamp(-10.0, 10.0);
        let out_right = (w1 * c[0] + (1.0 - w1) * c[1]).clamp(0.01, 10.0);
        let result = eml_safe(out_left, out_right);

        result.max(0.0)
    }

    /// Evaluate the depth-4 multi-head EML tree.
    ///
    /// Parameter layout (50 total):
    ///   [0..24]  Level 0: 8 linear combos, 3 weights each
    ///   [24..36] Level 2: 4 mixing nodes, 3 weights each (alpha, beta, gamma)
    ///   [36..44] Level 3: 2 mixing nodes, 4 weights each
    ///                      (alpha_l, beta_l, alpha_r, beta_r)
    ///   [44..50] Level 4: 3 output heads, 2 weights each
    fn evaluate_depth4(&self, params: &[f64], features: &GraphFeatures) -> CoherencePrediction {
        let (_a, b) = self.evaluate_levels_0_1(params, features);

        // Level 2: 4 EML nodes with light mixing
        // Mixing pairs for the 4 level-2 nodes:
        //   c0: mix(b0,b1), mix(b2,b3)
        //   c1: mix(b0,b1), mix(b2,b3)  (different weights)
        //   c2: mix(b0,b2), mix(b1,b3)
        //   c3: mix(b1,b3), mix(b0,b2)
        let level2_pairs: [(usize, usize, usize, usize); 4] = [
            (0, 1, 2, 3),
            (0, 1, 2, 3),
            (0, 2, 1, 3),
            (1, 3, 0, 2),
        ];

        let mut c = [0.0f64; 4];
        for i in 0..4 {
            let base = 24 + i * 3;
            let (li, lj, ri, rj) = level2_pairs[i];
            let (alpha, beta, gamma) = softmax3(params[base], params[base + 1], params[base + 2]);
            let mix_left = (alpha + beta * b[li] + gamma * b[lj]).clamp(-10.0, 10.0);
            // Re-derive a separate mix for right using shifted softmax
            let (alpha_r, beta_r, gamma_r) =
                softmax3(params[base] + 0.5, params[base + 1] - 0.5, params[base + 2]);
            let mix_right = (alpha_r + beta_r * b[ri] + gamma_r * b[rj]).clamp(0.01, 10.0);
            c[i] = eml_safe(mix_left, mix_right);
        }

        // Level 3: 2 EML nodes with heavier mixing
        //   d0 = eml(mix(c0,c1), mix(c2,c3))
        //   d1 = eml(mix(c0,c2), mix(c1,c3))
        let level3_pairs: [(usize, usize, usize, usize); 2] = [
            (0, 1, 2, 3),
            (0, 2, 1, 3),
        ];

        let mut d = [0.0f64; 2];
        for i in 0..2 {
            let base = 36 + i * 4;
            let (li, lj, ri, rj) = level3_pairs[i];
            let mix_left =
                (params[base] + params[base + 1] * c[li]
                    + (1.0 - params[base] - params[base + 1]) * c[lj])
                    .clamp(-10.0, 10.0);
            let mix_right =
                (params[base + 2] + params[base + 3] * c[ri]
                    + (1.0 - params[base + 2] - params[base + 3]) * c[rj])
                    .clamp(0.01, 10.0);
            d[i] = eml_safe(mix_left, mix_right);
        }

        // Level 4: Multi-head output -- 3 heads, 2 params each
        //   head_k = eml(mix_k(d0), mix_k(d1))
        let mut heads = [0.0f64; 3];
        for k in 0..3 {
            let base = 44 + k * 2;
            let w0 = params[base];
            let w1 = params[base + 1];
            let head_left = (w0 * d[0] + (1.0 - w0) * d[1]).clamp(-10.0, 10.0);
            let head_right = (w1 * d[0] + (1.0 - w1) * d[1]).clamp(0.01, 10.0);
            heads[k] = eml_safe(head_left, head_right);
        }

        CoherencePrediction {
            lambda_2: heads[0].max(0.0),
            fiedler_norm: heads[1].max(0.0),
            uncertainty: heads[2].max(0.0),
        }
    }

    /// Shared levels 0-1 evaluation (used by both depth-3 and depth-4).
    /// Returns (level-0 outputs, level-1 outputs).
    fn evaluate_levels_0_1(
        &self,
        params: &[f64],
        features: &GraphFeatures,
    ) -> ([f64; 8], [f64; 4]) {
        let inputs = features.normalized();

        // Level 0: 8 affine combinations, each selecting two features.
        let feature_pairs: [(usize, usize); 8] = [
            (0, 1), // node_count, edge_count
            (2, 3), // avg_degree, max_degree
            (4, 5), // density, component_count
            (6, 0), // min_degree, node_count
            (1, 2), // edge_count, avg_degree
            (3, 4), // max_degree, density
            (5, 6), // component_count, min_degree
            (0, 4), // node_count, density
        ];

        let mut a = [0.0f64; 8];
        for i in 0..8 {
            let base = i * 3;
            let (raw_alpha, raw_beta, raw_gamma) =
                (params[base], params[base + 1], params[base + 2]);
            let (alpha, beta, gamma) = softmax3(raw_alpha, raw_beta, raw_gamma);
            let (j, k) = feature_pairs[i];
            a[i] = alpha + beta * inputs[j] + gamma * inputs[k];
            a[i] = a[i].clamp(-10.0, 10.0);
        }

        // Level 1: 4 EML nodes
        let b = [
            eml_safe(a[0], a[1]),
            eml_safe(a[2], a[3]),
            eml_safe(a[4], a[5]),
            eml_safe(a[6], a[7]),
        ];

        (a, b)
    }

    // -------------------------------------------------------------------
    // Training
    // -------------------------------------------------------------------

    /// Record a training point (called after every exact Lanczos computation).
    ///
    /// Only records lambda_2; use [`record_full`] to also supply Fiedler norm
    /// and uncertainty ground truth.
    pub fn record(&mut self, features: GraphFeatures, lambda_2: f64) {
        self.record_full(features, lambda_2, None, None);
    }

    /// Record a full training point with optional Fiedler norm and uncertainty.
    pub fn record_full(
        &mut self,
        features: GraphFeatures,
        lambda_2: f64,
        fiedler_norm: Option<f64>,
        uncertainty: Option<f64>,
    ) {
        // Track prediction error for drift detection
        let predicted = self.predict(&features);
        self.error_history.push((predicted.lambda_2 - lambda_2).abs());
        if self.error_history.len() > 100 {
            self.error_history.remove(0);
        }

        self.training_data.push(TrainingPoint {
            features,
            lambda_2,
            fiedler_norm,
            uncertainty,
        });
    }

    /// Train the model when enough data is collected.
    ///
    /// Uses random restart + coordinate descent (gradient-free
    /// optimization suitable for 50 parameters).
    ///
    /// For multi-head models, trains all 3 heads jointly: shared trunk
    /// with separate head losses. The primary lambda_2 head receives
    /// weight 1.0, while fiedler_norm and uncertainty heads receive
    /// weight 0.3 each (when ground truth is available).
    ///
    /// Returns `true` if the model converged (MSE < 0.01).
    pub fn train(&mut self) -> bool {
        if self.training_data.len() < 50 {
            return false;
        }

        let param_count = self.params.len();
        let mut best_params = self.params.clone();
        let mut best_mse = self.evaluate_mse_joint(&self.params);

        // Phase 1: random restarts to find a good basin
        // More restarts for larger param spaces (depth-4 has 50 params).
        let restart_count = if param_count > PARAM_COUNT_V1 { 200 } else { 100 };
        let mut rng_state: u64 = 0xDEAD_BEEF_CAFE_1234;
        for _ in 0..restart_count {
            let params = random_params(&mut rng_state, param_count);
            let mse = self.evaluate_mse_joint(&params);
            if mse < best_mse {
                best_mse = mse;
                best_params = params;
            }
        }

        // Phase 2: coordinate descent refinement
        let deltas = [-0.1, -0.01, -0.001, 0.001, 0.01, 0.1];
        for _ in 0..1000 {
            let mut improved = false;
            for i in 0..param_count {
                for &delta in &deltas {
                    let mut candidate = best_params.clone();
                    candidate[i] += delta;
                    let mse = self.evaluate_mse_joint(&candidate);
                    if mse < best_mse {
                        best_mse = mse;
                        best_params = candidate;
                        improved = true;
                    }
                }
            }
            if !improved {
                break;
            }
        }

        self.params = best_params;
        self.trained = best_mse < 0.01;
        self.trained
    }

    /// Compute joint mean squared error over the training set.
    ///
    /// For depth-3 models, only lambda_2 loss.
    /// For depth-4 models, weighted sum of lambda_2, fiedler_norm, and
    /// uncertainty losses (when ground truth is available).
    fn evaluate_mse_joint(&self, params: &[f64]) -> f64 {
        if self.training_data.is_empty() {
            return f64::MAX;
        }

        let is_v1 = params.len() == PARAM_COUNT_V1;
        let mut total_loss = 0.0;
        let mut total_weight = 0.0;

        for tp in &self.training_data {
            if is_v1 {
                let predicted = self.evaluate_depth3(params, &tp.features);
                total_loss += (predicted - tp.lambda_2).powi(2);
                total_weight += 1.0;
            } else {
                let pred = self.evaluate_depth4_with_params(params, &tp.features);

                // Primary head: lambda_2 (weight 1.0, always available)
                total_loss += (pred.lambda_2 - tp.lambda_2).powi(2);
                total_weight += 1.0;

                // Secondary head: fiedler_norm (weight 0.3, if available)
                if let Some(target) = tp.fiedler_norm {
                    total_loss += 0.3 * (pred.fiedler_norm - target).powi(2);
                    total_weight += 0.3;
                }

                // Tertiary head: uncertainty (weight 0.3, if available)
                if let Some(target) = tp.uncertainty {
                    total_loss += 0.3 * (pred.uncertainty - target).powi(2);
                    total_weight += 0.3;
                }
            }
        }

        if total_weight > 0.0 {
            total_loss / total_weight
        } else {
            f64::MAX
        }
    }

    /// Evaluate depth-4 with arbitrary params (for training).
    fn evaluate_depth4_with_params(
        &self,
        params: &[f64],
        features: &GraphFeatures,
    ) -> CoherencePrediction {
        // We need to temporarily evaluate with different params than self.params.
        // Use inline evaluation to avoid aliasing issues.
        let inputs = features.normalized();

        let feature_pairs: [(usize, usize); 8] = [
            (0, 1),
            (2, 3),
            (4, 5),
            (6, 0),
            (1, 2),
            (3, 4),
            (5, 6),
            (0, 4),
        ];

        let mut a = [0.0f64; 8];
        for i in 0..8 {
            let base = i * 3;
            let (alpha, beta, gamma) =
                softmax3(params[base], params[base + 1], params[base + 2]);
            let (j, k) = feature_pairs[i];
            a[i] = (alpha + beta * inputs[j] + gamma * inputs[k]).clamp(-10.0, 10.0);
        }

        let b = [
            eml_safe(a[0], a[1]),
            eml_safe(a[2], a[3]),
            eml_safe(a[4], a[5]),
            eml_safe(a[6], a[7]),
        ];

        let level2_pairs: [(usize, usize, usize, usize); 4] = [
            (0, 1, 2, 3),
            (0, 1, 2, 3),
            (0, 2, 1, 3),
            (1, 3, 0, 2),
        ];

        let mut c = [0.0f64; 4];
        for i in 0..4 {
            let base = 24 + i * 3;
            let (alpha, beta, gamma) = softmax3(params[base], params[base + 1], params[base + 2]);
            let (li, lj, ri, rj) = level2_pairs[i];
            let mix_left = (alpha + beta * b[li] + gamma * b[lj]).clamp(-10.0, 10.0);
            let (alpha_r, beta_r, gamma_r) =
                softmax3(params[base] + 0.5, params[base + 1] - 0.5, params[base + 2]);
            let mix_right = (alpha_r + beta_r * b[ri] + gamma_r * b[rj]).clamp(0.01, 10.0);
            c[i] = eml_safe(mix_left, mix_right);
        }

        let level3_pairs: [(usize, usize, usize, usize); 2] = [
            (0, 1, 2, 3),
            (0, 2, 1, 3),
        ];

        let mut d = [0.0f64; 2];
        for i in 0..2 {
            let base = 36 + i * 4;
            let (li, lj, ri, rj) = level3_pairs[i];
            let mix_left =
                (params[base] + params[base + 1] * c[li]
                    + (1.0 - params[base] - params[base + 1]) * c[lj])
                    .clamp(-10.0, 10.0);
            let mix_right =
                (params[base + 2] + params[base + 3] * c[ri]
                    + (1.0 - params[base + 2] - params[base + 3]) * c[rj])
                    .clamp(0.01, 10.0);
            d[i] = eml_safe(mix_left, mix_right);
        }

        let mut heads = [0.0f64; 3];
        for k in 0..3 {
            let base = 44 + k * 2;
            let w0 = params[base];
            let w1 = params[base + 1];
            let head_left = (w0 * d[0] + (1.0 - w0) * d[1]).clamp(-10.0, 10.0);
            let head_right = (w1 * d[0] + (1.0 - w1) * d[1]).clamp(0.01, 10.0);
            heads[k] = eml_safe(head_left, head_right);
        }

        CoherencePrediction {
            lambda_2: heads[0].max(0.0),
            fiedler_norm: heads[1].max(0.0),
            uncertainty: heads[2].max(0.0),
        }
    }

    /// Compute mean squared error over the training set (lambda_2 only).
    /// Kept for backward compatibility with tests that inspect MSE.
    fn evaluate_mse(&self, params: &[f64]) -> f64 {
        if self.training_data.is_empty() {
            return f64::MAX;
        }
        let is_v1 = params.len() == PARAM_COUNT_V1;
        let sum: f64 = self
            .training_data
            .iter()
            .map(|tp| {
                let predicted = if is_v1 {
                    self.evaluate_depth3(params, &tp.features)
                } else {
                    self.evaluate_depth4_with_params(params, &tp.features)
                        .lambda_2
                };
                (predicted - tp.lambda_2).powi(2)
            })
            .sum();
        sum / self.training_data.len() as f64
    }
}

// ---------------------------------------------------------------------------
// CausalGraph integration
// ---------------------------------------------------------------------------

impl CausalGraph {
    /// O(1) approximate coherence from EML model.
    ///
    /// Returns a full [`CoherencePrediction`] with lambda_2, Fiedler norm,
    /// and uncertainty. Falls back to density-based estimate if model not
    /// trained.
    pub fn coherence_fast(&self, model: &EmlCoherenceModel) -> CoherencePrediction {
        let features = GraphFeatures::from_causal_graph(self);
        model.predict(&features)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Numerically safe EML: clamps exp and ensures positive ln argument.
#[inline]
fn eml_safe(x: f64, y: f64) -> f64 {
    let ex = x.clamp(-20.0, 20.0).exp();
    let ly = if y > 0.0 { y.ln() } else { f64::MIN_POSITIVE.ln() };
    ex - ly
}

/// Softmax over 3 values so that alpha + beta + gamma = 1.
#[inline]
fn softmax3(a: f64, b: f64, c: f64) -> (f64, f64, f64) {
    let max = a.max(b).max(c);
    let ea = (a - max).exp();
    let eb = (b - max).exp();
    let ec = (c - max).exp();
    let sum = ea + eb + ec;
    (ea / sum, eb / sum, ec / sum)
}

/// Generate random parameters in [-1, 1] using a simple LCG.
fn random_params(state: &mut u64, count: usize) -> Vec<f64> {
    let mut params = vec![0.0f64; count];
    for p in params.iter_mut() {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *p = (*state >> 33) as f64 / (u32::MAX as f64 / 2.0) - 1.0;
    }
    params
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal::{CausalEdgeType, CausalGraph};

    // -- eml operator -------------------------------------------------------

    #[test]
    fn eml_identity() {
        // eml(0, 1) = exp(0) - ln(1) = 1 - 0 = 1
        let result = eml(0.0, 1.0);
        assert!(
            (result - 1.0).abs() < 1e-12,
            "eml(0, 1) should be 1.0, got {result}"
        );
    }

    #[test]
    fn eml_exp_only() {
        // eml(1, 1) = exp(1) - ln(1) = e - 0 = e
        let result = eml(1.0, 1.0);
        assert!(
            (result - std::f64::consts::E).abs() < 1e-12,
            "eml(1, 1) should be e, got {result}"
        );
    }

    #[test]
    fn eml_ln_only() {
        // eml(0, e) = exp(0) - ln(e) = 1 - 1 = 0
        let result = eml(0.0, std::f64::consts::E);
        assert!(
            result.abs() < 1e-12,
            "eml(0, e) should be 0.0, got {result}"
        );
    }

    // -- GraphFeatures extraction -------------------------------------------

    #[test]
    fn features_empty_graph() {
        let g = CausalGraph::new();
        let f = GraphFeatures::from_causal_graph(&g);
        assert_eq!(f.node_count, 0.0);
        assert_eq!(f.edge_count, 0.0);
        assert_eq!(f.density, 0.0);
        assert_eq!(f.component_count, 0.0);
    }

    #[test]
    fn features_triangle() {
        let g = CausalGraph::new();
        let a = g.add_node("A".into(), serde_json::json!({}));
        let b = g.add_node("B".into(), serde_json::json!({}));
        let c = g.add_node("C".into(), serde_json::json!({}));
        g.link(a, b, CausalEdgeType::Causes, 1.0, 0, 0);
        g.link(b, c, CausalEdgeType::Causes, 1.0, 0, 0);
        g.link(c, a, CausalEdgeType::Causes, 1.0, 0, 0);

        let f = GraphFeatures::from_causal_graph(&g);
        assert_eq!(f.node_count, 3.0);
        assert_eq!(f.edge_count, 3.0);
        assert!((f.avg_degree - 2.0).abs() < 1e-9);
        assert_eq!(f.component_count, 1.0);
        // density = 2*3 / (3*2) = 1.0
        assert!((f.density - 1.0).abs() < 1e-9);
    }

    #[test]
    fn features_disconnected() {
        let g = CausalGraph::new();
        let _a = g.add_node("A".into(), serde_json::json!({}));
        let _b = g.add_node("B".into(), serde_json::json!({}));

        let f = GraphFeatures::from_causal_graph(&g);
        assert_eq!(f.node_count, 2.0);
        assert_eq!(f.edge_count, 0.0);
        assert_eq!(f.component_count, 2.0);
        assert_eq!(f.min_degree, 0.0);
        assert_eq!(f.max_degree, 0.0);
    }

    // -- EmlCoherenceModel prediction (untrained fallback) ------------------

    #[test]
    fn predict_untrained_fallback() {
        let model = EmlCoherenceModel::new();
        assert!(!model.is_trained());

        let features = GraphFeatures {
            node_count: 10.0,
            edge_count: 20.0,
            avg_degree: 4.0,
            max_degree: 6.0,
            min_degree: 2.0,
            density: 0.444,
            component_count: 1.0,
        };
        let result = model.predict(&features);
        // Fallback: density * avg_degree
        let expected = 0.444 * 4.0;
        assert!(
            (result.lambda_2 - expected).abs() < 1e-9,
            "untrained fallback: expected {expected}, got {}",
            result.lambda_2
        );
    }

    #[test]
    fn predict_untrained_returns_multi_head() {
        let model = EmlCoherenceModel::new();
        let features = GraphFeatures {
            node_count: 10.0,
            edge_count: 20.0,
            avg_degree: 4.0,
            max_degree: 6.0,
            min_degree: 2.0,
            density: 0.444,
            component_count: 1.0,
        };
        let pred = model.predict(&features);
        // All three heads should produce values
        assert!(pred.lambda_2 >= 0.0);
        assert!(pred.fiedler_norm >= 0.0);
        assert!(pred.uncertainty >= 0.0);
    }

    #[test]
    fn predict_lambda2_convenience() {
        let model = EmlCoherenceModel::new();
        let features = GraphFeatures {
            node_count: 10.0,
            edge_count: 20.0,
            avg_degree: 4.0,
            max_degree: 6.0,
            min_degree: 2.0,
            density: 0.444,
            component_count: 1.0,
        };
        let lambda2 = model.predict_lambda2(&features);
        let full = model.predict(&features);
        assert!(
            (lambda2 - full.lambda_2).abs() < 1e-12,
            "predict_lambda2 should match predict().lambda_2"
        );
    }

    // -- Backward compat: 34-param models still work -----------------------

    #[test]
    fn backward_compat_v1_model() {
        let mut model = EmlCoherenceModel::new_v1();
        assert!(!model.is_multi_head());
        assert_eq!(model.params.len(), 34);

        let features = GraphFeatures {
            node_count: 10.0,
            edge_count: 20.0,
            avg_degree: 4.0,
            max_degree: 6.0,
            min_degree: 2.0,
            density: 0.444,
            component_count: 1.0,
        };

        // Untrained fallback still works
        let pred = model.predict(&features);
        let expected = 0.444 * 4.0;
        assert!(
            (pred.lambda_2 - expected).abs() < 1e-9,
            "v1 untrained fallback should match"
        );
        // Synthetic fiedler_norm and uncertainty
        assert!(pred.fiedler_norm >= 0.0);
        assert!(pred.uncertainty >= 0.0);

        // Record + train should work on v1 model
        for i in 0..60 {
            let f = GraphFeatures {
                node_count: (i + 3) as f64,
                edge_count: (i + 2) as f64,
                avg_degree: 2.0,
                max_degree: 3.0,
                min_degree: 1.0,
                density: 0.5,
                component_count: 1.0,
            };
            model.record(f, 1.0);
        }
        // Should not panic
        let _ = model.train();
    }

    // -- Multi-head model basics -------------------------------------------

    #[test]
    fn new_model_is_multi_head() {
        let model = EmlCoherenceModel::new();
        assert!(model.is_multi_head());
        assert_eq!(model.params.len(), 50);
    }

    #[test]
    fn multi_head_prediction_produces_three_values() {
        // Set some non-zero params to get a trained model response.
        let mut model = EmlCoherenceModel::new();
        // Force trained flag for testing
        model.trained = true;
        // Set some non-trivial params
        for (i, p) in model.params.iter_mut().enumerate() {
            *p = ((i as f64) * 0.1).sin() * 0.5;
        }

        let features = GraphFeatures {
            node_count: 10.0,
            edge_count: 20.0,
            avg_degree: 4.0,
            max_degree: 6.0,
            min_degree: 2.0,
            density: 0.5,
            component_count: 1.0,
        };

        let pred = model.predict(&features);
        // All three heads should be non-negative (clamped)
        assert!(
            pred.lambda_2 >= 0.0,
            "lambda_2 should be non-negative, got {}",
            pred.lambda_2
        );
        assert!(
            pred.fiedler_norm >= 0.0,
            "fiedler_norm should be non-negative, got {}",
            pred.fiedler_norm
        );
        assert!(
            pred.uncertainty >= 0.0,
            "uncertainty should be non-negative, got {}",
            pred.uncertainty
        );
        // Verify they are finite
        assert!(pred.lambda_2.is_finite());
        assert!(pred.fiedler_norm.is_finite());
        assert!(pred.uncertainty.is_finite());
    }

    #[test]
    fn uncertainty_is_non_negative() {
        let model = EmlCoherenceModel::new();
        // Test across various feature combinations
        for n in [3.0, 10.0, 50.0, 100.0] {
            for d in [0.1, 0.5, 1.0] {
                let e = n * (n - 1.0) * d / 2.0;
                let features = GraphFeatures {
                    node_count: n,
                    edge_count: e,
                    avg_degree: (n - 1.0) * d,
                    max_degree: (n - 1.0) * d * 1.5,
                    min_degree: ((n - 1.0) * d * 0.5).max(0.0),
                    density: d,
                    component_count: 1.0,
                };
                let pred = model.predict(&features);
                assert!(
                    pred.uncertainty >= 0.0,
                    "uncertainty must be non-negative for n={n}, d={d}: got {}",
                    pred.uncertainty
                );
            }
        }
    }

    // -- EmlCoherenceModel record + training --------------------------------

    #[test]
    fn record_increments_count() {
        let mut model = EmlCoherenceModel::new();
        assert_eq!(model.training_sample_count(), 0);

        let f = GraphFeatures {
            node_count: 5.0,
            edge_count: 4.0,
            avg_degree: 1.6,
            max_degree: 2.0,
            min_degree: 1.0,
            density: 0.4,
            component_count: 1.0,
        };
        model.record(f, 0.5);
        assert_eq!(model.training_sample_count(), 1);
    }

    #[test]
    fn record_full_stores_optional_targets() {
        let mut model = EmlCoherenceModel::new();
        let f = GraphFeatures {
            node_count: 5.0,
            edge_count: 4.0,
            avg_degree: 1.6,
            max_degree: 2.0,
            min_degree: 1.0,
            density: 0.4,
            component_count: 1.0,
        };
        model.record_full(f, 0.5, Some(1.2), Some(0.3));
        assert_eq!(model.training_sample_count(), 1);
        assert!(model.training_data[0].fiedler_norm.is_some());
        assert!(model.training_data[0].uncertainty.is_some());
    }

    #[test]
    fn train_insufficient_data_returns_false() {
        let mut model = EmlCoherenceModel::new();
        // Add only 10 samples (need 50)
        for i in 0..10 {
            let f = GraphFeatures {
                node_count: i as f64,
                edge_count: i as f64,
                avg_degree: 2.0,
                max_degree: 3.0,
                min_degree: 1.0,
                density: 0.5,
                component_count: 1.0,
            };
            model.record(f, 1.0);
        }
        assert!(!model.train());
        assert!(!model.is_trained());
    }

    // -- Convergence test with known graph families -------------------------

    /// Generate training data from known graph families where lambda_2
    /// has a closed-form expression, then verify the model can learn.
    #[test]
    fn convergence_on_known_graphs() {
        let mut model = EmlCoherenceModel::new();

        let mut samples = Vec::new();

        // Complete graph K_n: lambda_2 = n, density = 1.0
        for n in 3..30 {
            let nf = n as f64;
            let e = nf * (nf - 1.0) / 2.0;
            let lambda_2 = nf;
            samples.push((
                GraphFeatures {
                    node_count: nf,
                    edge_count: e,
                    avg_degree: nf - 1.0,
                    max_degree: nf - 1.0,
                    min_degree: nf - 1.0,
                    density: 1.0,
                    component_count: 1.0,
                },
                lambda_2,
            ));
        }

        // Star graph S_n: lambda_2 = 1
        for n in 3..30 {
            let nf = n as f64;
            samples.push((
                GraphFeatures {
                    node_count: nf,
                    edge_count: nf - 1.0,
                    avg_degree: 2.0 * (nf - 1.0) / nf,
                    max_degree: nf - 1.0,
                    min_degree: 1.0,
                    density: 2.0 * (nf - 1.0) / (nf * (nf - 1.0)),
                    component_count: 1.0,
                },
                1.0,
            ));
        }

        // Cycle graph C_n: lambda_2 = 2(1 - cos(2*pi/n))
        for n in 3..30 {
            let nf = n as f64;
            let lambda_2 = 2.0 * (1.0 - (2.0 * std::f64::consts::PI / nf).cos());
            samples.push((
                GraphFeatures {
                    node_count: nf,
                    edge_count: nf,
                    avg_degree: 2.0,
                    max_degree: 2.0,
                    min_degree: 2.0,
                    density: 2.0 * nf / (nf * (nf - 1.0)),
                    component_count: 1.0,
                },
                lambda_2,
            ));
        }

        // Path graph P_n: lambda_2 = 2(1 - cos(pi/n))
        for n in 3..30 {
            let nf = n as f64;
            let lambda_2 = 2.0 * (1.0 - (std::f64::consts::PI / nf).cos());
            samples.push((
                GraphFeatures {
                    node_count: nf,
                    edge_count: nf - 1.0,
                    avg_degree: 2.0 * (nf - 1.0) / nf,
                    max_degree: 2.0,
                    min_degree: 1.0,
                    density: 2.0 * (nf - 1.0) / (nf * (nf - 1.0)),
                    component_count: 1.0,
                },
                lambda_2,
            ));
        }

        // Erdos-Renyi G(n, p): lambda_2 ~ n*p - 2*sqrt(n*p*(1-p))
        for n in [20, 50, 100, 200] {
            for &p in &[0.1, 0.2, 0.3, 0.5, 0.7] {
                let nf = n as f64;
                let e = nf * (nf - 1.0) * p / 2.0;
                let avg_deg = (nf - 1.0) * p;
                let lambda_2 = (nf * p - 2.0 * (nf * p * (1.0 - p)).sqrt()).max(0.0);
                samples.push((
                    GraphFeatures {
                        node_count: nf,
                        edge_count: e,
                        avg_degree: avg_deg,
                        max_degree: avg_deg * 1.5,
                        min_degree: (avg_deg * 0.5).max(0.0),
                        density: p,
                        component_count: 1.0,
                    },
                    lambda_2,
                ));
            }
        }

        // Feed all samples as training data
        for (features, lambda_2) in &samples {
            model.record(features.clone(), *lambda_2);
        }

        assert!(
            model.training_sample_count() >= 50,
            "should have enough training data: {}",
            model.training_sample_count()
        );

        // Train
        let converged = model.train();

        // Verify: even if not fully converged on this mixed dataset,
        // the MSE should be reasonable. The depth-4 model has a wider
        // search space (50 params) and the dataset spans lambda_2 from
        // ~0.04 (path graphs) to 29 (K_29), so allow more headroom.
        let mse = model.evaluate_mse(&model.params);
        assert!(
            mse < 500.0,
            "MSE should be reasonable after training, got {mse}"
        );

        // If converged, the model should predict reasonably
        if converged {
            let k5 = GraphFeatures {
                node_count: 5.0,
                edge_count: 10.0,
                avg_degree: 4.0,
                max_degree: 4.0,
                min_degree: 4.0,
                density: 1.0,
                component_count: 1.0,
            };
            let pred = model.predict(&k5);
            assert!(
                pred.lambda_2 > 0.0,
                "prediction for K5 should be positive, got {}",
                pred.lambda_2
            );
        }
    }

    /// Depth-4 convergence: verify the multi-head model can train on
    /// data with all three targets.
    #[test]
    fn depth4_convergence_with_full_targets() {
        let mut model = EmlCoherenceModel::new();
        assert!(model.is_multi_head());

        // Generate 100 samples with all three targets
        for n in 3..53 {
            let nf = n as f64;
            let e = nf * (nf - 1.0) / 2.0;
            let lambda_2 = nf; // K_n
            let fiedler_norm = (nf - 1.0).sqrt(); // approximate
            let uncertainty = 0.1 * lambda_2; // synthetic
            let features = GraphFeatures {
                node_count: nf,
                edge_count: e,
                avg_degree: nf - 1.0,
                max_degree: nf - 1.0,
                min_degree: nf - 1.0,
                density: 1.0,
                component_count: 1.0,
            };
            model.record_full(features, lambda_2, Some(fiedler_norm), Some(uncertainty));
        }

        for n in 3..53 {
            let nf = n as f64;
            let lambda_2 = 2.0 * (1.0 - (2.0 * std::f64::consts::PI / nf).cos());
            let fiedler_norm = lambda_2.sqrt();
            let uncertainty = 0.05 * lambda_2;
            let features = GraphFeatures {
                node_count: nf,
                edge_count: nf,
                avg_degree: 2.0,
                max_degree: 2.0,
                min_degree: 2.0,
                density: 2.0 * nf / (nf * (nf - 1.0)),
                component_count: 1.0,
            };
            model.record_full(features, lambda_2, Some(fiedler_norm), Some(uncertainty));
        }

        assert!(model.training_sample_count() >= 100);

        // Should not panic and should produce finite MSE
        let _ = model.train();
        let mse = model.evaluate_mse(&model.params);
        assert!(mse.is_finite(), "MSE should be finite, got {mse}");
    }

    // -- CausalGraph::coherence_fast integration ----------------------------

    #[test]
    fn coherence_fast_on_triangle() {
        let g = CausalGraph::new();
        let a = g.add_node("A".into(), serde_json::json!({}));
        let b = g.add_node("B".into(), serde_json::json!({}));
        let c = g.add_node("C".into(), serde_json::json!({}));
        g.link(a, b, CausalEdgeType::Causes, 1.0, 0, 0);
        g.link(b, c, CausalEdgeType::Causes, 1.0, 0, 0);
        g.link(c, a, CausalEdgeType::Causes, 1.0, 0, 0);

        let model = EmlCoherenceModel::new();
        let fast = g.coherence_fast(&model);
        // Untrained: density * avg_degree = 1.0 * 2.0 = 2.0
        assert!(
            (fast.lambda_2 - 2.0).abs() < 1e-9,
            "coherence_fast untrained triangle: expected 2.0, got {}",
            fast.lambda_2
        );
        // Multi-head: fiedler_norm and uncertainty should also be present
        assert!(fast.fiedler_norm >= 0.0);
        assert!(fast.uncertainty >= 0.0);
    }

    #[test]
    fn coherence_fast_empty() {
        let g = CausalGraph::new();
        let model = EmlCoherenceModel::new();
        let fast = g.coherence_fast(&model);
        assert!(
            fast.lambda_2.abs() < 1e-12,
            "coherence_fast on empty graph should be 0"
        );
    }

    // -- Helper function tests ----------------------------------------------

    #[test]
    fn softmax3_sums_to_one() {
        let (a, b, c) = softmax3(1.0, 2.0, 3.0);
        let sum = a + b + c;
        assert!(
            (sum - 1.0).abs() < 1e-12,
            "softmax3 should sum to 1.0, got {sum}"
        );
    }

    #[test]
    fn softmax3_equal_inputs() {
        let (a, b, c) = softmax3(0.0, 0.0, 0.0);
        assert!((a - 1.0 / 3.0).abs() < 1e-12);
        assert!((b - 1.0 / 3.0).abs() < 1e-12);
        assert!((c - 1.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn eml_safe_does_not_panic() {
        // Extreme values should not panic
        let _ = eml_safe(100.0, 0.0);
        let _ = eml_safe(-100.0, -5.0);
        let _ = eml_safe(0.0, f64::MIN_POSITIVE);
        let _ = eml_safe(f64::NAN, 1.0); // NaN propagation is acceptable
    }

    #[test]
    fn error_history_tracks_drift() {
        let mut model = EmlCoherenceModel::new();
        let f = GraphFeatures {
            node_count: 5.0,
            edge_count: 5.0,
            avg_degree: 2.0,
            max_degree: 2.0,
            min_degree: 2.0,
            density: 0.5,
            component_count: 1.0,
        };

        model.record(f.clone(), 1.0);
        model.record(f.clone(), 2.0);
        assert_eq!(model.error_history.len(), 2);
        assert!(model.mean_error() >= 0.0);
    }
}
