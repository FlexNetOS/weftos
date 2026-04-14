# Sprint 17 — Knowledge Graph & Analysis Upgrade

**Source**: Paper surveys (22 papers), shaal PR #352 synergy, graphify gaps
**Version**: 0.6.x series (v0.7.0 = GUI complete)

---

## P0 — Implement Now

### KG-001: EML Score Fusion for graphify query
**Source**: Shaal PR #352 EmlScoreFusion + our query-time gap
**Target**: `clawft-graphify/src/analyze.rs`, `clawft-weave/src/commands/graphify_cmd.rs`
**What**: Non-linear fusion of vector similarity + keyword + graph-structural relevance for `weaver graphify query`. EML discovers the optimal weighting per domain.
**Effort**: M
**Impact**: Transforms query from keyword matching to semantic hybrid search
- [ ] Create `QueryFusion` EML model in graphify/eml_models.rs
- [ ] Wire into graphify_cmd.rs `run_query()`
- [ ] Combine HNSW results + keyword results + graph proximity
- [ ] Train from user query feedback
- [ ] Tests

### KG-002: Community Summary Generation (GraphRAG)
**Source**: Paper 1 — Microsoft GraphRAG
**Target**: `clawft-graphify/src/cluster.rs`, new `summary.rs`
**What**: After community detection, generate text summaries per community. Store in graph metadata. Use for answering global "what is this codebase about?" queries.
**Effort**: M
**Impact**: Major query quality improvement
- [ ] Add `generate_community_summary()` using community member labels
- [ ] Store summaries in KnowledgeGraph metadata
- [ ] Query matches against community summaries for global questions
- [ ] Tests

### KG-003: Causal Chain Tracing (CausalRAG)
**Source**: Paper 2 — CausalRAG (ACL 2025)
**Target**: `clawft-kernel/src/causal.rs`, `causal_predict.rs`
**What**: When answering questions, trace causal chains through the graph (A causes B enables C). Return the chain as context, not just matched nodes.
**Effort**: S
**Impact**: Adds "why" to answers — "auth fails BECAUSE config_service returns stale tokens BECAUSE cache TTL is 0"
- [ ] Add `trace_causal_chain(from, to, max_depth)` to CausalGraph
- [ ] Return typed edge path with explanations
- [ ] Wire into graphify query results
- [ ] Tests

### KG-004: Random Fourier Spectral Embedding (SASE)
**Source**: Paper 3 — SASE (CIKM 2024)
**Target**: `clawft-kernel/src/causal.rs` spectral_analysis()
**What**: Replace O(k*m) Lanczos with O(m) Random Fourier Feature approximation for large graphs. 3-6x speedup.
**Effort**: M
**Impact**: Enables spectral analysis on 100K+ node graphs
- [ ] Implement RFF spectral embedding
- [ ] Compare accuracy vs Lanczos on test graphs
- [ ] Use as fast path when graph > threshold size
- [ ] Benchmark and document

### KG-005: Information Gain Pruning
**Source**: Paper 4 — Information Gain Pruning (2026)
**Target**: `causal_predict.rs`, DEMOCRITUS loop
**What**: Filter redundant evidence by marginal information gain. If adding edge E gives Δλ₂=0.001 after edge D already gave Δλ₂=0.3, skip E.
**Effort**: S
**Impact**: 50-70% token reduction in dense graphs
- [ ] Add `is_redundant(candidate, recent_additions, threshold)` check
- [ ] Wire into DEMOCRITUS UPDATE phase
- [ ] Tests

### KG-006: BFS Dependency Graph Retrieval (SGKR)
**Source**: Paper 4 (Phase 2) — SGKR
**Target**: `clawft-graphify/src/extract/cross_file.rs`, `model.rs`
**What**: Trace data flow through function-call dependencies. "Where does this input go?" BFS over the AST dependency graph.
**Effort**: M
**Impact**: System understanding — maps data flow paths
- [ ] Add `trace_data_flow(entity_id, direction)` to KnowledgeGraph
- [ ] BFS over call edges following input→output
- [ ] Wire into graphify query
- [ ] Tests

### KG-007: MCTS Graph Exploration (RANGER)
**Source**: Paper 5 — RANGER (2025)
**Target**: `clawft-graphify/src/analyze.rs`
**What**: Monte Carlo Tree Search over the knowledge graph for code retrieval. Explores paths through the graph with learned priors.
**Effort**: L
**Impact**: Better code retrieval for complex multi-hop queries
- [ ] Implement MCTS graph walker
- [ ] Exploration prior from node type + edge type
- [ ] Wire into graphify query as alternative to keyword search
- [ ] Benchmark vs keyword + HNSW

---

## P1 — Next Sprint

### KG-008: Entity Dedup via HNSW (CodaRAG)
**Source**: Paper 5 (Phase 2) — CodaRAG
**Target**: `clawft-graphify/src/build.rs`
**What**: Check for near-duplicate entities via HNSW similarity before inserting. Reduces graph size 10-30%.
**Effort**: S
- [ ] Add dedup check in build.rs using HNSW near-neighbor
- [ ] Merge duplicates (keep best metadata)
- [ ] Tests

### KG-009: Geometric Shadowing for Memory Decay (RoMem)
**Source**: Paper 1 (Phase 2) — RoMem
**Target**: `clawft-kernel/src/causal.rs`
**What**: Replace hard age-pruning with geometric decay. Recent nodes cast "shadows" that suppress older redundant nodes. Semantic speed gate (per-edge-type volatility) as EML model.
**Effort**: M
- [ ] Implement geometric shadowing
- [ ] EML-learned volatility per edge type
- [ ] Tests

### KG-010: Multi-hop Traversal with Priors (TRACE)
**Source**: Paper 2 (Phase 2) — TRACE
**Target**: `clawft-graphify/src/analyze.rs`
**What**: Beam search over KG with reusable traversal patterns. 18% token savings.
**Effort**: M
- [ ] Beam search with exploration priors
- [ ] Pattern reuse across queries
- [ ] Tests

### KG-011: LogQuantized for DiskANN (Shaal)
**Source**: PR #352 — LogQuantized
**Target**: `clawft-kernel/src/vector_diskann.rs`
**What**: Replace scalar PQ codebooks with logarithmic quantization. 20-52% lower reconstruction error on skewed distributions.
**Effort**: M
- [ ] Integrate LogQuantized when ruvector-core merges shaal's PR
- [ ] Benchmark on our embedding distributions
- [ ] Wire into DiskANN backend

### KG-012: Unified SIMD Distance Kernel (Shaal)
**Source**: PR #352 — UnifiedDistanceParams
**Target**: `clawft-kernel/src/hnsw_service.rs`
**What**: Branch-free SIMD distance. +14% QPS on SIFT1M.
**Effort**: S
- [ ] Upgrade ruvector dependency after PR merges
- [ ] Benchmark on our workloads
- [ ] Configure for our embedding dimensions

### KG-013: Spatio-temporal GNN for Sonobuoy (K-STEMIT)
**Source**: Paper 7 (Phase 2) — K-STEMIT
**Target**: New `clawft-sensor/` crate or sonobuoy firmware
**What**: GraphSAGE spatial aggregation = learned beamforming for irregular arrays. Gated temporal convolution = learned matched filtering for species ID.
**Effort**: L
- [ ] Design sonobuoy graph topology (buoys as nodes, distances as edges)
- [ ] Implement GraphSAGE-style neighborhood aggregation
- [ ] Temporal feature extraction from acoustic time series
- [ ] Species classification head

### KG-014: Codebook Cold-Start (TransFIR)
**Source**: Paper 6 (Phase 2) — TransFIR
**Target**: `clawft-kernel/src/causal.rs`
**What**: VQ codebook for new entity embeddings when no training data exists. ~150 lines of Rust.
**Effort**: S
- [ ] Implement vector quantization codebook
- [ ] Map new entities to nearest codebook entry
- [ ] Tests

---

## P2 — Backlog

### KG-015: EA-Agent Entity Alignment
**Source**: Paper 11 — EA-Agent (ACL 2026, arxiv 2604.11686)
**What**: Entity alignment across KGs for multi-repo dedup
**Effort**: L

### KG-016: Conversational Graph Exploration
**Source**: Paper 12 — From Data to Dialogue
**What**: Structured dialogue patterns for graph exploration
**Effort**: M

### KG-017: Knowledge Distillation for Edge EML
**Source**: Paper 3 (Phase 2) — SevenNet-Nano
**What**: Distill depth-4 EML models into depth-2 for WASM/ESP32
**Effort**: M

### KG-018: Newman Modularity Scoring
**Source**: Paper 15 — Community Detection Review
**What**: Alternative to cohesion scoring in cluster.rs
**Effort**: S

---

## Summary

| Priority | Tasks | Key Theme |
|----------|-------|-----------|
| P0 | 7 tasks | Query-time retrieval + graph analysis depth |
| P1 | 7 tasks | Performance + domain-specific + shaal synergy |
| P2 | 4 tasks | Research / future capabilities |
| **Total** | **18 tasks** | |
