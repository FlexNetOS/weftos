# Phase 3 Implementation Notes: Kernel Bridge + Domain Layers

**Date:** 2026-04-04
**Status:** Complete (79 tests passing, 0 warnings)

## Files Created / Modified

### New files
- `src/bridge.rs` -- Kernel bridge (feature-gated behind `kernel-bridge`)
- `src/domain/mod.rs` -- Domain trait and DomainRegistry
- `src/domain/code.rs` -- CodeDomainConfig (feature-gated behind `code-domain`)
- `src/domain/forensic.rs` -- ForensicDomainConfig + gap_analysis + coherence_score + counterfactual_delta (feature-gated behind `forensic-domain`)

### Modified files
- `src/lib.rs` -- Added `domain` and `bridge` module declarations + re-exports for GodNode, SurprisingConnection, etc.
- `src/model.rs` -- Added GodNode, SurprisingConnection, SuggestedQuestion, GraphDiff structs
- `src/ingest.rs` -- Fixed temporary value borrow bug (E0716)
- `Cargo.toml` -- Added `async-trait` dep, added `features = ["ecc"]` to clawft-kernel dep, added `dep:async-trait` to kernel-bridge feature

## Bridge Module (src/bridge.rs)

### GraphifyBridge
- Holds `Arc<CausalGraph>`, `Arc<HnswService>`, `Arc<CrossRefStore>` + entity-to-causal DashMap
- `ingest()` -- full KG ingestion: entities -> CausalNodes + HNSW embeddings + CrossRefs; relationships -> CausalEdges + CrossRefs
- `ingest_entity()` -- single entity: CausalGraph node + HNSW insert + CrossRef
- `ingest_relationship()` -- single edge: CausalEdge + CrossRef preserving original RelationType
- `export_from_causal()` -- reverse direction: reconstruct KnowledgeGraph from CausalGraph
- `causal_node_for()` -- reverse lookup EntityId -> CausalNodeId

### EmbeddingProvider trait
- Uses `#[async_trait]` for dyn-compatibility
- `NoOpEmbedder` returns zero vectors (for testing)

### RelationType -> CausalEdgeType mapping
- Calls/Imports/DependsOn -> Causes
- Contains/MethodOf/Implements/Extends/Configures -> Enables
- Contradicts -> Contradicts
- Corroborates/WitnessedBy -> EvidenceFor
- Precedes -> Follows
- SemanticallySimilarTo/RelatedTo/FoundAt/LocatedAt -> Correlates
- AlibiedBy -> Inhibits

### CrossRef discriminants
- Code domain: 0x20-0x29
- Forensic domain: 0x30-0x3C
- Custom/unknown: 0x3F
- StructureTag::Custom(0x20) for Graphify namespace

### GraphifyAnalyzer (9th analyzer)
- Implements `Analyzer` trait from clawft-kernel
- id = "graphify", categories = architecture/dependencies/complexity/knowledge-gaps
- Standalone `analyze_kg_to_findings()` function for pre-built KnowledgeGraphs
- Produces findings: god nodes -> complexity, surprising connections -> dependencies, singleton communities -> architecture

## Code Domain (src/domain/code.rs)
- 13 entity types: Module, Class, Function, Import, Config, Service, Endpoint, Interface, Struct, Enum, Constant, Package, File
- 11 edge types: Calls, Imports, ImportsFrom, DependsOn, Contains, Implements, Configures, Extends, MethodOf, Instantiates, RelatedTo

## Forensic Domain (src/domain/forensic.rs)
- 14 entity types: Person, Event, Evidence, Location, Timeline, Document, Hypothesis, Organization, PhysicalObject, DigitalArtifact, FinancialRecord, Communication, File, Concept
- 13 edge types: WitnessedBy, FoundAt, Contradicts, Corroborates, AlibiedBy, Precedes, DocumentedIn, OwnedBy, ContactedBy, LocatedAt, SemanticallySimilarTo, RelatedTo, CaseOf

### gap_analysis()
- Unlinked evidence: Evidence nodes with degree 0-1
- Timeline discontinuities: Event nodes without Precedes edges
- Unverified claims: Edges with Confidence::Ambiguous
- Missing connections: Person nodes not linked to any Event

### coherence_score()
- density * avg_confidence (both in [0,1])
- Single node returns 1.0, empty graph returns 0.0

### counterfactual_delta()
- Predicts coherence improvement if hypothetical edge added
- Analytical approximation (no mutation of original graph)

## Feature Gate Verification
- `--no-default-features` -- compiles (standalone mode, no bridge/domains)
- `--features kernel-bridge` -- compiles (bridge + CausalGraph integration)
- `--features code-domain` -- compiles (code entity/edge types)
- `--features forensic-domain` -- compiles (forensic types + gap analysis)
- `--features "kernel-bridge,code-domain,forensic-domain"` -- compiles (full)

## Design Decisions
1. Used `#[async_trait]` rather than native async fn in trait -- required for dyn-compatibility (`&dyn EmbeddingProvider`)
2. `export_from_causal()` is lossy (CausalEdgeType -> RelationType mapping is many-to-one) -- original RelationType preserved in CrossRef metadata
3. God nodes and surprising connections added to both model.rs (standalone) and bridge.rs (kernel-integrated) to support both modes
4. Forensic gap analysis uses O(n*m) edge iteration; acceptable for typical case sizes (<10K entities) but should be optimized with indexes for large graphs
