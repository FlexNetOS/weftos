//! Hybrid vector backend: hot HNSW cache + cold DiskANN store.
//!
//! Implements the "map of maps" pattern: HNSW holds the frequently-accessed
//! vectors for fast lookup, while DiskANN holds the full corpus. Vectors
//! are promoted from cold to hot based on access frequency, and evicted
//! from the hot tier using LRU when it reaches capacity.
//!
//! Compiled only when the `ecc` feature is enabled.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::vector_backend::{SearchResult, VectorBackend, VectorResult};
use crate::vector_diskann::{DiskAnnBackend, DiskAnnConfig};
use crate::vector_hnsw::HnswBackend;
use crate::hnsw_service::HnswServiceConfig;

// ── Configuration ────────────────────────────────────────────────────────

/// Eviction policy for the hot tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EvictionPolicy {
    /// Least Recently Used — evict the vector that was accessed longest ago.
    #[default]
    Lru,
}

/// Hybrid backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridConfig {
    /// Maximum number of vectors in the hot (HNSW) tier.
    #[serde(default = "default_hot_capacity")]
    pub hot_capacity: usize,

    /// Access count threshold before a cold vector is promoted to hot.
    #[serde(default = "default_promotion_threshold")]
    pub promotion_threshold: u32,

    /// Eviction policy for the hot tier when it is full.
    #[serde(default)]
    pub eviction_policy: EvictionPolicy,
}

fn default_hot_capacity() -> usize {
    50_000
}
fn default_promotion_threshold() -> u32 {
    3
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            hot_capacity: default_hot_capacity(),
            promotion_threshold: default_promotion_threshold(),
            eviction_policy: EvictionPolicy::default(),
        }
    }
}

// ── LRU tracker ──────────────────────────────────────────────────────────

/// Simple LRU tracker backed by a `VecDeque` of IDs.
///
/// When an ID is "touched", it moves to the back (most recent).
/// Eviction pops from the front (least recent).
struct LruTracker {
    order: VecDeque<u64>,
    positions: HashMap<u64, usize>,
}

impl LruTracker {
    fn new() -> Self {
        Self {
            order: VecDeque::new(),
            positions: HashMap::new(),
        }
    }

    /// Mark an ID as recently used. If new, push to back.
    fn touch(&mut self, id: u64) {
        if self.positions.contains_key(&id) {
            // Remove from current position and push to back.
            self.order.retain(|&x| x != id);
        }
        self.order.push_back(id);
        self.rebuild_positions();
    }

    /// Evict the least recently used ID. Returns `None` if empty.
    fn evict(&mut self) -> Option<u64> {
        let id = self.order.pop_front()?;
        self.positions.remove(&id);
        self.rebuild_positions();
        Some(id)
    }

    /// Remove a specific ID from the tracker.
    fn remove(&mut self, id: u64) {
        if self.positions.remove(&id).is_some() {
            self.order.retain(|&x| x != id);
            self.rebuild_positions();
        }
    }

    fn len(&self) -> usize {
        self.order.len()
    }

    fn rebuild_positions(&mut self) {
        self.positions.clear();
        for (i, &id) in self.order.iter().enumerate() {
            self.positions.insert(id, i);
        }
    }
}

// ── Stored vector snapshot (for promotion) ───────────────────────────────

/// Snapshot of a vector for transfer between tiers.
struct VectorSnapshot {
    key: String,
    vector: Vec<f32>,
    metadata: serde_json::Value,
}

// ── Cold-tier vector cache (needed for promotion) ────────────────────────

/// Cache of raw vectors stored alongside the DiskANN backend so that
/// we can promote vectors to the hot tier without re-embedding.
struct ColdVectorCache {
    vectors: HashMap<u64, VectorSnapshot>,
}

impl ColdVectorCache {
    fn new() -> Self {
        Self {
            vectors: HashMap::new(),
        }
    }

    fn insert(&mut self, id: u64, key: String, vector: Vec<f32>, metadata: serde_json::Value) {
        self.vectors.insert(
            id,
            VectorSnapshot {
                key,
                vector,
                metadata,
            },
        );
    }

    fn get(&self, id: u64) -> Option<&VectorSnapshot> {
        self.vectors.get(&id)
    }

    fn remove(&mut self, id: u64) {
        self.vectors.remove(&id);
    }
}

// ── Hybrid backend ──────────────────────────────────────────────────────

/// Hybrid vector backend combining HNSW (hot) + DiskANN (cold).
///
/// # Data flow
///
/// - **Insert**: vector goes to DiskANN (always) and HNSW (if under hot
///   capacity).
/// - **Search**: query both tiers, merge results by distance, deduplicate
///   by ID.
/// - **Promotion**: access counts are tracked per ID. When a cold-only
///   vector exceeds `promotion_threshold` accesses, it is copied into
///   the hot tier.
/// - **Eviction**: when HNSW exceeds `hot_capacity`, the LRU entry is
///   evicted (it remains in DiskANN).
pub struct HybridBackend {
    hot: HnswBackend,
    cold: DiskAnnBackend,
    config: HybridConfig,
    /// Per-ID access counters for promotion decisions.
    access_counts: DashMap<u64, u32>,
    /// LRU tracker for the hot tier (protected by Mutex for ordered ops).
    lru: Mutex<LruTracker>,
    /// Cold vector cache for promotion (protected by Mutex).
    cold_cache: Mutex<ColdVectorCache>,
    /// Set of IDs currently in the hot tier.
    hot_ids: DashMap<u64, ()>,
}

impl HybridBackend {
    /// Create a new hybrid backend.
    pub fn new(
        hnsw_config: HnswServiceConfig,
        diskann_config: DiskAnnConfig,
        hybrid_config: HybridConfig,
    ) -> Self {
        Self {
            hot: HnswBackend::new(hnsw_config),
            cold: DiskAnnBackend::new(diskann_config),
            config: hybrid_config,
            access_counts: DashMap::new(),
            lru: Mutex::new(LruTracker::new()),
            cold_cache: Mutex::new(ColdVectorCache::new()),
            hot_ids: DashMap::new(),
        }
    }

    /// Create with all-default configurations.
    pub fn with_defaults() -> Self {
        Self::new(
            HnswServiceConfig::default(),
            DiskAnnConfig::default(),
            HybridConfig::default(),
        )
    }

    /// Borrow the hybrid configuration.
    pub fn config(&self) -> &HybridConfig {
        &self.config
    }

    /// Return the number of vectors in the hot tier.
    pub fn hot_len(&self) -> usize {
        self.hot_ids.len()
    }

    /// Return the number of vectors in the cold tier.
    pub fn cold_len(&self) -> usize {
        self.cold.len()
    }

    /// Try to promote a vector from cold to hot tier.
    fn try_promote(&self, id: u64) {
        // Already in hot tier?
        if self.hot_ids.contains_key(&id) {
            return;
        }

        // Check access count.
        let count = self
            .access_counts
            .get(&id)
            .map(|v| *v)
            .unwrap_or(0);

        if count < self.config.promotion_threshold {
            return;
        }

        // Get the vector from cold cache.
        let snapshot = {
            let cache = self.cold_cache.lock().expect("cold_cache lock poisoned");
            cache.get(id).map(|s| VectorSnapshot {
                key: s.key.clone(),
                vector: s.vector.clone(),
                metadata: s.metadata.clone(),
            })
        };

        let Some(snap) = snapshot else { return };

        // Evict from hot tier if at capacity.
        self.ensure_hot_capacity();

        // Insert into hot tier.
        if self
            .hot
            .insert(id, &snap.key, &snap.vector, snap.metadata)
            .is_ok()
        {
            self.hot_ids.insert(id, ());
            let mut lru = self.lru.lock().expect("LRU lock poisoned");
            lru.touch(id);
        }
    }

    /// Ensure the hot tier has room for one more entry.
    fn ensure_hot_capacity(&self) {
        let mut lru = self.lru.lock().expect("LRU lock poisoned");
        while lru.len() >= self.config.hot_capacity {
            if let Some(evict_id) = lru.evict() {
                self.hot.remove(evict_id);
                self.hot_ids.remove(&evict_id);
            } else {
                break;
            }
        }
    }

    /// Increment the access counter for a given ID.
    fn record_access(&self, id: u64) {
        self.access_counts
            .entry(id)
            .and_modify(|c| {
                *c = c.saturating_add(1);
            })
            .or_insert(1);

        // Touch LRU if in hot tier.
        if self.hot_ids.contains_key(&id) {
            let mut lru = self.lru.lock().expect("LRU lock poisoned");
            lru.touch(id);
        }
    }

    /// Merge and deduplicate search results from hot and cold tiers.
    fn merge_results(hot_results: Vec<SearchResult>, cold_results: Vec<SearchResult>, k: usize) -> Vec<SearchResult> {
        let mut seen = std::collections::HashSet::new();
        let mut merged: Vec<SearchResult> = Vec::with_capacity(hot_results.len() + cold_results.len());

        // Collect all results, dedup by id.
        for r in hot_results.into_iter().chain(cold_results) {
            if seen.insert(r.id) {
                merged.push(r);
            }
        }

        // Sort by distance ascending.
        merged.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        merged.truncate(k);
        merged
    }
}

impl VectorBackend for HybridBackend {
    fn insert(
        &self,
        id: u64,
        key: &str,
        vector: &[f32],
        metadata: serde_json::Value,
    ) -> VectorResult<()> {
        // Always insert into cold tier.
        self.cold
            .insert(id, key, vector, metadata.clone())?;

        // Cache the raw vector for future promotion.
        {
            let mut cache = self.cold_cache.lock().expect("cold_cache lock poisoned");
            cache.insert(id, key.to_owned(), vector.to_vec(), metadata.clone());
        }

        // Insert into hot tier if under capacity.
        if self.hot_ids.len() < self.config.hot_capacity {
            self.hot.insert(id, key, vector, metadata)?;
            self.hot_ids.insert(id, ());
            let mut lru = self.lru.lock().expect("LRU lock poisoned");
            lru.touch(id);
        }

        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        // Search both tiers.
        let hot_results = self.hot.search(query, k);
        let cold_results = self.cold.search(query, k);

        let merged = Self::merge_results(hot_results, cold_results, k);

        // Record access for all returned IDs and potentially promote.
        for r in &merged {
            self.record_access(r.id);
            self.try_promote(r.id);
        }

        merged
    }

    fn len(&self) -> usize {
        // Total unique vectors = cold tier (which holds everything).
        self.cold.len()
    }

    fn contains(&self, id: u64) -> bool {
        self.cold.contains(id)
    }

    fn remove(&self, id: u64) -> bool {
        // Remove from both tiers.
        let hot_removed = self.hot.remove(id);
        if hot_removed {
            self.hot_ids.remove(&id);
            let mut lru = self.lru.lock().expect("LRU lock poisoned");
            lru.remove(id);
        }
        let cold_removed = self.cold.remove(id);
        self.access_counts.remove(&id);
        {
            let mut cache = self.cold_cache.lock().expect("cold_cache lock poisoned");
            cache.remove(id);
        }
        cold_removed
    }

    fn flush(&self) -> VectorResult<()> {
        self.hot.flush()?;
        self.cold.flush()?;
        Ok(())
    }

    fn backend_name(&self) -> &str {
        "hybrid"
    }
}

impl std::fmt::Debug for HybridBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridBackend")
            .field("hot_len", &self.hot_len())
            .field("cold_len", &self.cold_len())
            .field("config", &self.config)
            .finish()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn small_hybrid() -> HybridBackend {
        HybridBackend::new(
            HnswServiceConfig::default(),
            DiskAnnConfig {
                max_points: 1000,
                ..DiskAnnConfig::default()
            },
            HybridConfig {
                hot_capacity: 3,
                promotion_threshold: 2,
                eviction_policy: EvictionPolicy::Lru,
            },
        )
    }

    #[test]
    fn insert_populates_both_tiers_under_capacity() {
        let h = small_hybrid();
        h.insert(1, "a", &[1.0, 0.0, 0.0], serde_json::json!({}))
            .unwrap();
        h.insert(2, "b", &[0.0, 1.0, 0.0], serde_json::json!({}))
            .unwrap();

        assert_eq!(h.len(), 2);
        assert_eq!(h.hot_len(), 2);
        assert_eq!(h.cold_len(), 2);
    }

    #[test]
    fn insert_beyond_hot_capacity_stays_cold_only() {
        let h = small_hybrid();
        // Fill hot tier (capacity = 3).
        h.insert(1, "a", &[1.0, 0.0, 0.0], serde_json::json!({}))
            .unwrap();
        h.insert(2, "b", &[0.0, 1.0, 0.0], serde_json::json!({}))
            .unwrap();
        h.insert(3, "c", &[0.0, 0.0, 1.0], serde_json::json!({}))
            .unwrap();

        // 4th vector exceeds hot capacity.
        h.insert(4, "d", &[0.5, 0.5, 0.0], serde_json::json!({}))
            .unwrap();

        assert_eq!(h.len(), 4);
        assert_eq!(h.hot_len(), 3); // Still 3 (not promoted yet).
        assert_eq!(h.cold_len(), 4);
    }

    #[test]
    fn search_returns_merged_results() {
        let h = small_hybrid();
        h.insert(1, "a", &[1.0, 0.0, 0.0], serde_json::json!({}))
            .unwrap();
        h.insert(2, "b", &[0.0, 1.0, 0.0], serde_json::json!({}))
            .unwrap();

        let results = h.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        // Closest should be id=1.
        assert_eq!(results[0].id, 1);
        assert!(results[0].distance < 0.01);
    }

    #[test]
    fn promotion_after_threshold() {
        let h = small_hybrid();
        // Fill hot tier.
        h.insert(1, "a", &[1.0, 0.0, 0.0], serde_json::json!({}))
            .unwrap();
        h.insert(2, "b", &[0.0, 1.0, 0.0], serde_json::json!({}))
            .unwrap();
        h.insert(3, "c", &[0.0, 0.0, 1.0], serde_json::json!({}))
            .unwrap();

        // Insert cold-only vector.
        h.insert(4, "d", &[0.5, 0.5, 0.0], serde_json::json!({}))
            .unwrap();
        assert!(!h.hot_ids.contains_key(&4));

        // Access id=4 enough times to trigger promotion.
        // promotion_threshold = 2, so we need 2+ accesses.
        h.record_access(4);
        h.record_access(4);
        h.try_promote(4);

        // After promotion, hot tier should evict LRU (id=1, first inserted)
        // and add id=4.
        assert!(h.hot_ids.contains_key(&4));
        assert_eq!(h.hot_len(), 3); // Still at capacity.
    }

    #[test]
    fn remove_cleans_both_tiers() {
        let h = small_hybrid();
        h.insert(1, "a", &[1.0, 0.0], serde_json::json!({}))
            .unwrap();
        assert!(h.contains(1));
        assert!(h.remove(1));
        assert!(!h.contains(1));
        assert_eq!(h.hot_len(), 0);
        assert_eq!(h.cold_len(), 0);
    }

    #[test]
    fn merge_results_deduplicates() {
        let hot = vec![
            SearchResult::new(1, "a".into(), 0.1, serde_json::json!({})),
            SearchResult::new(2, "b".into(), 0.3, serde_json::json!({})),
        ];
        let cold = vec![
            SearchResult::new(1, "a".into(), 0.1, serde_json::json!({})), // dup
            SearchResult::new(3, "c".into(), 0.2, serde_json::json!({})),
        ];

        let merged = HybridBackend::merge_results(hot, cold, 3);
        assert_eq!(merged.len(), 3);
        // Should be sorted by distance: 0.1, 0.2, 0.3
        assert_eq!(merged[0].id, 1);
        assert_eq!(merged[1].id, 3);
        assert_eq!(merged[2].id, 2);
    }

    #[test]
    fn backend_name() {
        let h = small_hybrid();
        assert_eq!(h.backend_name(), "hybrid");
    }

    #[test]
    fn flush_both_tiers() {
        let h = small_hybrid();
        h.flush().unwrap();
    }

    #[test]
    fn eviction_policy_default() {
        assert_eq!(EvictionPolicy::default(), EvictionPolicy::Lru);
    }

    #[test]
    fn lru_tracker_basics() {
        let mut lru = LruTracker::new();
        lru.touch(1);
        lru.touch(2);
        lru.touch(3);
        assert_eq!(lru.len(), 3);

        // Touch 1 again — moves to back.
        lru.touch(1);
        // Evict should return 2 (now least recently used).
        assert_eq!(lru.evict(), Some(2));
        assert_eq!(lru.len(), 2);
    }
}
