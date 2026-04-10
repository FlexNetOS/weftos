//! HNSW-backed [`VectorBackend`] implementation.
//!
//! Wraps the existing [`HnswService`] behind the unified
//! [`VectorBackend`] trait so that it can be used standalone or as the
//! hot tier inside [`HybridBackend`](super::vector_hybrid::HybridBackend).
//!
//! Compiled only when the `ecc` feature is enabled.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::hnsw_service::{HnswService, HnswServiceConfig};
use crate::vector_backend::{SearchResult, VectorBackend, VectorError, VectorResult};

// ── ID ↔ key mapping ────────────────────────────────────────────────────

/// Internal mapping between numeric IDs (used by `VectorBackend`) and
/// the string keys used by `HnswService`.
struct IdMap {
    id_to_key: HashMap<u64, String>,
    key_to_id: HashMap<String, u64>,
}

impl IdMap {
    fn new() -> Self {
        Self {
            id_to_key: HashMap::new(),
            key_to_id: HashMap::new(),
        }
    }

    fn insert(&mut self, id: u64, key: String) {
        // Remove old key mapping if this id was previously used.
        if let Some(old_key) = self.id_to_key.insert(id, key.clone()) {
            if old_key != key {
                self.key_to_id.remove(&old_key);
            }
        }
        self.key_to_id.insert(key, id);
    }

    fn contains_id(&self, id: u64) -> bool {
        self.id_to_key.contains_key(&id)
    }

    fn key_for_id(&self, id: u64) -> Option<&str> {
        self.id_to_key.get(&id).map(|s| s.as_str())
    }

    fn remove(&mut self, id: u64) -> Option<String> {
        if let Some(key) = self.id_to_key.remove(&id) {
            self.key_to_id.remove(&key);
            Some(key)
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.id_to_key.len()
    }
}

// ── Backend ─────────────────────────────────────────────────────────────

/// HNSW vector backend wrapping [`HnswService`].
pub struct HnswBackend {
    inner: HnswService,
    id_map: Mutex<IdMap>,
}

impl HnswBackend {
    /// Create a new HNSW backend with the given configuration.
    pub fn new(config: HnswServiceConfig) -> Self {
        Self {
            inner: HnswService::new(config),
            id_map: Mutex::new(IdMap::new()),
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(HnswServiceConfig::default())
    }

    /// Access the underlying [`HnswService`] for legacy code paths.
    pub fn inner(&self) -> &HnswService {
        &self.inner
    }
}

impl VectorBackend for HnswBackend {
    fn insert(
        &self,
        id: u64,
        key: &str,
        vector: &[f32],
        metadata: serde_json::Value,
    ) -> VectorResult<()> {
        let mut map = self.id_map.lock().expect("IdMap lock poisoned");
        map.insert(id, key.to_owned());
        self.inner.insert(key.to_owned(), vector.to_vec(), metadata);
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        let results = self.inner.search(query, k);
        let map = self.id_map.lock().expect("IdMap lock poisoned");
        results
            .into_iter()
            .filter_map(|r| {
                // Reverse-lookup the numeric id from the string key.
                // If we can't find one (e.g., inserted via raw HnswService), skip.
                map.key_to_id.get(&r.id).map(|&numeric_id| {
                    // HnswService returns cosine similarity (1.0 = identical).
                    // Convert to distance: distance = 1.0 - similarity.
                    let distance = 1.0 - r.score;
                    SearchResult::new(numeric_id, r.id, distance, r.metadata)
                })
            })
            .collect()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn contains(&self, id: u64) -> bool {
        let map = self.id_map.lock().expect("IdMap lock poisoned");
        map.contains_id(id)
    }

    fn remove(&self, id: u64) -> bool {
        let mut map = self.id_map.lock().expect("IdMap lock poisoned");
        // HnswService doesn't support removal, so we just clear our mapping.
        // The vector remains in the HNSW graph but won't appear in results
        // because we filter by known ids.
        map.remove(id).is_some()
    }

    fn flush(&self) -> VectorResult<()> {
        // HNSW is in-memory only; flush is a no-op.
        Ok(())
    }

    fn backend_name(&self) -> &str {
        "hnsw"
    }
}

impl std::fmt::Debug for HnswBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HnswBackend")
            .field("len", &self.len())
            .finish()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_backend() -> HnswBackend {
        HnswBackend::with_defaults()
    }

    #[test]
    fn insert_and_search() {
        let b = make_backend();
        b.insert(1, "a", &[1.0, 0.0, 0.0], serde_json::json!({}))
            .unwrap();
        b.insert(2, "b", &[0.0, 1.0, 0.0], serde_json::json!({}))
            .unwrap();
        b.insert(3, "c", &[0.0, 0.0, 1.0], serde_json::json!({}))
            .unwrap();

        assert_eq!(b.len(), 3);
        assert!(!b.is_empty());

        let results = b.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[0].key, "a");
        assert!(results[0].distance < 0.01);
    }

    #[test]
    fn contains_and_remove() {
        let b = make_backend();
        b.insert(10, "x", &[1.0, 0.0], serde_json::json!({}))
            .unwrap();
        assert!(b.contains(10));
        assert!(!b.contains(99));

        assert!(b.remove(10));
        assert!(!b.contains(10));
        assert!(!b.remove(10));
    }

    #[test]
    fn flush_is_noop() {
        let b = make_backend();
        b.flush().unwrap();
    }

    #[test]
    fn backend_name() {
        let b = make_backend();
        assert_eq!(b.backend_name(), "hnsw");
    }

    #[test]
    fn empty_search() {
        let b = make_backend();
        let results = b.search(&[1.0, 0.0], 5);
        assert!(results.is_empty());
    }
}
