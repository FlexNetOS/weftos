# Adaptive HNSW: Tiered Dimensional Search with Tree Calculus Strategy Selection

**Date**: 2026-04-17
**Version**: v0.6.13
**Files changed**: `hnsw_store.rs`, `hnsw_service.rs`, `hnsw_eml.rs`, `chain.rs`, `a2a.rs`, `lib.rs`

## Problem

The HNSW vector store used static parameters (ef=100, single full-dimensional
index). The four EML models in `hnsw_eml.rs` (ef, path, distance, rebuild)
existed but were never wired into the search path. Initial attempts to tune
ef adaptively showed that:

- Single-head ef model collapsed to reproducing the current ef (circular signal)
- 2-head ef+recall model with Score strategy achieved parity but couldn't beat static
- Tuning ef on a single index is the wrong lever — the real gain is structural

## Solution: Multi-Index Tiered Search

Three-tier dimensional search where each tier operates at a different resolution:

```
Query arrives
  ↓
[Probe] Analyze corpus variance spectrum (one-time, at build)
  ↓
[Triage] Tree calculus classifies spectrum shape:
  Atom     → uniform data, use flat HNSW
  Sequence → gradual decay, moderate tiering
  Branch   → sharp clusters, aggressive tiering
  ↓
[EML] Compute tier parameters:
  coarse_dims, coarse_keep, medium_dims, medium_keep, ef
  ↓
[Build] Construct separate HNSW indexes per tier:
  Coarse: 20-dim projected embeddings (cheap graph traversal)
  Full:   128-dim store for re-ranking
  ↓
[Search] Tiered query:
  1. Coarse HNSW → 120 candidates (20-dim cosine, fast)
  2. Medium re-rank → 40 survivors (40-dim cosine)
  3. Fine re-rank → top-10 (128-dim cosine, exact)
```

## Key Insight

The coarse HNSW index is built on **projected** embeddings (e.g., 20 dims
that capture 80% of the corpus variance). The graph connectivity is different
from the full-dimensional graph because "nearby" in 20-dim space differs from
"nearby" in 128-dim space. This means the graph traversal itself is cheap
(20-float distance instead of 128-float), not just the re-scoring.

On uniform random data this fails catastrophically (recall 0.04) because
all dimensions are equally informative. On structured data (clustered
embeddings where early dims carry signal), the coarse index captures
cluster membership and the fine tiers resolve within-cluster order.

## Tree Calculus Integration

Following the ADR pattern from `treecalc.rs` (graphify) and
`adr-treecalc-eml-architecture.md`:

- **Tree calculus** handles structural dispatch: triage on spectrum form
  (Atom/Sequence/Branch) determines which search architecture to use
- **EML** handles continuous parameters: exp-ln scoring computes tier
  widths and keep counts from steepness and knee position

The `TriageRecord` captures the full decision chain (form, steepness,
concentration, knee, strategy, parameters) and is serializable for
ExoChain logging via `EVENT_KIND_HNSW_EML_TRIAGE`.

## Benchmark Results (5000 vecs, 128 dims, k=10, structured data)

```
                          recall@10    mean (ns)    p99 (ns)
  Control (flat HNSW):     0.6937        95,239      149,613
  Tiered (coarse→fine):    0.7937        58,978       97,034

  Tiered: 1.61× faster, +10% recall
```

The probe detected steepness=1986, concentration=92.5%, and chose
`Branch` form with coarse=20d/keep=120, medium=40d/keep=40, ef=50.

## What Didn't Work (and why)

1. **Single-head ef tuning**: model learned to reproduce current ef, not
   find a better one. Circular training signal.
2. **2-head ef+recall with Score strategy**: correctly avoided collapse but
   settled near the starting point. The ef knob has limited range on a
   single full-dimensional index.
3. **Single-index tiered re-rank**: fetched candidates from full-dim HNSW
   then re-ranked with partial dims. Can only remove good candidates,
   never add missed ones. No speedup because graph traversal was already
   full-dim.
4. **Uniform random test data**: tiered search on i.i.d. random vectors
   gives recall=0.04. All dimensions are equally informative, so
   projecting to a subset loses signal. Structured data is required.

## ExoChain Events Added

- `hnsw.eml.observe` — per-search multi-signal payload (ef, latency, recall)
- `hnsw.eml.recall` — periodic recall measurement checkpoint
- `hnsw.eml.trained` — EML model training cycle complete
- `hnsw.eml.triage` — tree calculus strategy decision record

## Files

- `crates/clawft-core/src/embeddings/hnsw_store.rs` — `TieredSearch`,
  `cosine_partial`, `set_ef_search`, `brute_force_topk`
- `crates/clawft-kernel/src/hnsw_eml.rs` — 2-head ef model, probe,
  triage, benchmark harness, structured data generator
- `crates/clawft-kernel/src/hnsw_service.rs` — EML integration into
  search path, `build_tiered`, `measure_recall`
- `crates/clawft-kernel/src/chain.rs` — ExoChain event constants
- `crates/clawft-kernel/src/a2a.rs` — Rust 2024 edition fix
