# Phase 1A Development Notes

**Date:** 2026-04-04
**Tasks:** GRAPH-001, GRAPH-002, GRAPH-003, GRAPH-012, GRAPH-013, GRAPH-014, GRAPH-015

## Crate Setup

- Created `crates/clawft-graphify/` with Cargo.toml matching the master plan.
- Added to workspace members and workspace.dependencies in root Cargo.toml.
- Added `petgraph`, `rayon`, `walkdir` to workspace.dependencies for consistency.
- Default features set to `["code-domain"]` only for Phase 1A (tree-sitter features deferred to Phase 1B when extractors are implemented).

## Design Decisions

### GRAPH-001: Core Data Model

- **EntityId:** 32-byte BLAKE3 hash of (domain_byte, entity_type_discriminant, name, source_file). Also provides `from_legacy_string()` which hashes a Python-style string ID for backward compatibility with JSON import.
- **DomainTag:** Enum with Code=0x20, Forensic=0x21, Custom(u8). Single-byte discriminator embedded in hash.
- **EntityType:** 26 variants (12 code + 12 forensic + File + Concept + Custom). Discriminant strings are frozen for ID stability -- `Struct` maps to `"struct_"`, `Enum` to `"enum_"` (trailing underscore avoids Rust keyword collision while staying stable).
- **FileType:** 6 variants. Uses `#[serde(rename_all = "lowercase")]` for Python compatibility. Added `config` and `unknown` as Rust extensions beyond the Python 4 types.
- **Confidence:** Serde as UPPERCASE to match Python output. `to_weight()` returns 1.0/0.7/0.4 (graph algorithms), `to_score()` returns 1.0/0.5/0.2 (JSON export confidence_score field, matching Python defaults).
- **RelationType:** 23 variants (10 code + 11 forensic + RelatedTo + CaseOf + Custom). Serde as snake_case.

### GRAPH-002: KnowledgeGraph

- Wraps `petgraph::Graph<Entity, Relationship, Directed>` with `HashMap<EntityId, NodeIndex>` for O(1) lookup.
- `add_entity()` is idempotent: last-write-wins (matches Python NetworkX behavior).
- `add_relationship()` returns `None` silently when source/target missing (matches Python skip-external behavior).
- `subgraph()` creates a new KnowledgeGraph with edges between included nodes only.
- Scale test: 1000 entities + 3000 edges passes.

### GRAPH-003: Error Types

- `GraphifyError` with 9 variants, all carrying String messages.
- `From<io::Error>` maps to CacheError.
- `From<serde_json::Error>` maps to ValidationError.

### GRAPH-012: Graph Assembly (build.rs)

- `build()` merges ExtractionResults in order, accumulating stats.
- `build_from_json()` parses Python-compatible JSON format, using `EntityId::from_legacy_string()` for node IDs.
- Dangling edge references (stdlib/external imports) are filtered as warnings, not hard errors.
- RelationType parsing from string covers all 10 code variants + shared + Custom fallback.

### GRAPH-013: Content Cache (cache.rs)

- Uses BLAKE3 for content hashing (not SHA256 as in Python, per plan).
- Cache location: `.weftos/graphify-cache/` (not `graphify-out/cache/`).
- Atomic writes via write-to-tmp + rename.
- `EXTRACTOR_VERSION` constant for cache invalidation on extractor changes.
- GC removes entries whose content hash no longer matches any live file.

### GRAPH-014: Schema Validation (validation.rs)

- Port of Python's `validate.py` with identical checks.
- Accepts both Python-compatible file types (code, document, paper, image) and Rust extensions (config, unknown).
- Dangling edge references produce warnings (included in error list but filtered by `build_from_json`).

### GRAPH-015: JSON Export (export/json.rs)

- Outputs Python-compatible `node_link_data` format: `directed: false`, `multigraph: false`, `graph: {}`, `nodes: [...]`, `links: [...]`, `hyperedges: [...]`.
- Uses legacy string IDs when available, falls back to BLAKE3 hex.
- Adds `community` field to nodes from KnowledgeGraph communities.
- Adds `confidence_score` using `Confidence::to_score()` matching Python defaults.
- `_src` and `_tgt` fields preserved from relationship metadata for edge direction fidelity.

## Test Coverage

37 unit tests across all modules:
- entity.rs: 8 tests (determinism, serde, display, legacy IDs)
- relationship.rs: 5 tests (weights, scores, serde roundtrip)
- model.rs: 5 tests (idempotent add, skip missing, neighbors, subgraph, 1K scale)
- build.rs: 4 tests (merge, dedup, external edges, JSON import)
- cache.rs: 4 tests (roundtrip, content change miss, GC, clear)
- validation.rs: 7 tests (valid, missing keys, invalid types, dangling refs, extended types)
- export/json.rs: 4 tests (structure, confidence scores, communities, file write)

## Files Created

- `crates/clawft-graphify/Cargo.toml`
- `crates/clawft-graphify/src/lib.rs`
- `crates/clawft-graphify/src/entity.rs`
- `crates/clawft-graphify/src/relationship.rs`
- `crates/clawft-graphify/src/model.rs`
- `crates/clawft-graphify/src/build.rs`
- `crates/clawft-graphify/src/cache.rs`
- `crates/clawft-graphify/src/validation.rs`
- `crates/clawft-graphify/src/export/mod.rs`
- `crates/clawft-graphify/src/export/json.rs`

## Files Modified

- `Cargo.toml` (workspace root): added crate to members, added workspace deps for petgraph/rayon/walkdir/clawft-graphify
