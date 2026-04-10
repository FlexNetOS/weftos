# Hybrid Vector Search Backend

Sprint 16 — Vector backend abstraction and hybrid HNSW+DiskANN tier.

## What Changed

### New files (all in `crates/clawft-kernel/src/`)

| File | Purpose |
|------|---------|
| `vector_backend.rs` | `VectorBackend` trait, `SearchResult`, `VectorError`, `VectorBackendKind` enum |
| `vector_hnsw.rs` | `HnswBackend` — wraps existing `HnswService` behind `VectorBackend` trait |
| `vector_diskann.rs` | `DiskAnnBackend` — brute-force stub (no real `ruvector-diskann` dep yet), same API |
| `vector_hybrid.rs` | `HybridBackend` — hot HNSW + cold DiskANN with promotion/eviction |

### Modified files

| File | Change |
|------|--------|
| `crates/clawft-types/src/config/kernel.rs` | Added `VectorConfig`, `VectorBackendKind`, `VectorHnswConfig`, `VectorDiskAnnConfig`, `VectorHybridConfig`, `VectorEvictionPolicy`; added `vector: Option<VectorConfig>` to `KernelConfig` |
| `crates/clawft-kernel/src/lib.rs` | Registered 4 new modules (`vector_backend`, `vector_hnsw`, `vector_diskann`, `vector_hybrid`); added re-exports |
| `crates/clawft-kernel/src/boot.rs` | Added `vector_backend` field to `EccSubsystem`; constructs backend based on config at boot; added `ecc_vector_backend()` accessor |
| `crates/clawft-kernel/src/config.rs` | Added `vector: None` to test KernelConfig literal |
| `crates/clawft-kernel/tests/e2e_integration.rs` | Added `vector: None` to test KernelConfig literals |
| `crates/clawft-kernel/tests/feature_composition.rs` | Added `vector: None` to test KernelConfig literals |

## Architecture

```
                    ┌─────────────────────┐
                    │   VectorBackend     │  (trait)
                    │   trait object      │
                    └──────┬──────────────┘
                           │
           ┌───────────────┼───────────────┐
           │               │               │
    ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
    │ HnswBackend │ │DiskAnnBackend│ │HybridBackend│
    │ (in-memory) │ │  (stub/SSD) │ │  hot + cold  │
    └─────────────┘ └─────────────┘ └──────────────┘
```

### Hybrid data flow

- **Insert**: always goes to DiskANN (cold); also goes to HNSW (hot) if under `hot_capacity`
- **Search**: queries both tiers, merges by distance, deduplicates by ID
- **Promotion**: access-counted per ID; when count exceeds `promotion_threshold`, vector is copied from cold to hot
- **Eviction**: LRU eviction from hot tier when at capacity (vector remains in cold)

## Configuration

```toml
[kernel.vector]
backend = "hybrid"   # "hnsw" | "diskann" | "hybrid"

[kernel.vector.hnsw]
ef_construction = 200
m = 16
max_elements = 100000

[kernel.vector.diskann]
max_points = 10000000
dimensions = 384
data_path = ".weftos/diskann"

[kernel.vector.hybrid]
hot_capacity = 50000
promotion_threshold = 3
eviction_policy = "lru"
```

## DiskANN status

The `ruvector-diskann` crate does not exist on crates.io yet. The current `DiskAnnBackend` is a brute-force stub using a `HashMap<u64, StoredEntry>` with linear-scan cosine distance. It has the same API surface that a real DiskANN integration would use. When `ruvector-diskann` ships, swap the internals behind the `diskann` feature flag.

## Test coverage

35 new tests across the 4 modules:
- `vector_backend`: 3 tests (SearchResult, BackendKind serde, default)
- `vector_hnsw`: 5 tests (insert/search, contains/remove, flush, name, empty)
- `vector_diskann`: 7 tests (insert/search, remove, capacity, upsert, cosine, name, flush)
- `vector_hybrid`: 9 tests (tiers, promotion, eviction, merge/dedup, remove, LRU)
- `clawft-types::config::kernel`: 4 new tests (VectorConfig defaults, hybrid deser, diskann deser, kernel+vector)

## Next steps

- [ ] Wire `VectorBackend` into `DemocritusLoop` (currently uses raw `HnswService`)
- [ ] Add `ecc.vector-config` RPC endpoint to show active backend
- [ ] Implement real DiskANN backend when `ruvector-diskann` publishes
- [ ] Add `diskann` feature flag gating for the real implementation
- [ ] Benchmark hybrid vs. pure HNSW for ECC workloads
