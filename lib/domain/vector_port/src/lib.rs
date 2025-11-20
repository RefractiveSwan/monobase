//! Shared vector-store port definitions for domain crates.
//! Keeps traits/types (backend enum, config, embeddings, mock) environment-free
//! so platform adapters (Qdrant, PGVector, etc.) can implement the trait without
//! leaking env logic into mapping/pipeline crates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;

/// Supported vector backends. Platform adapters map these to real services.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorBackend {
    Qdrant,
    PgVector,
    Milvus,
    Mock,
}

impl FromStr for VectorBackend {
    type Err = ();

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.to_ascii_lowercase().as_str() {
            "qdrant" => Ok(Self::Qdrant),
            "pgvector" => Ok(Self::PgVector),
            "milvus" => Ok(Self::Milvus),
            "mock" => Ok(Self::Mock),
            _ => Err(()),
        }
    }
}

/// Configuration shared with vector adapters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub backend: VectorBackend,
    pub url: Option<String>,
    pub namespace: String,
    pub pool_max: u32,
    pub health_timeout_ms: u64,
    pub enabled: bool,
}

impl VectorStoreConfig {
    pub fn validate(&self) -> Result<(), VectorStoreConfigError> {
        if self.enabled && self.namespace.trim().is_empty() {
            return Err(VectorStoreConfigError::MissingNamespace);
        }
        if self.enabled && !matches!(self.backend, VectorBackend::Mock) && self.url.is_none() {
            return Err(VectorStoreConfigError::MissingUrl);
        }
        if self.pool_max == 0 {
            return Err(VectorStoreConfigError::InvalidPoolMax);
        }
        if self.health_timeout_ms == 0 {
            return Err(VectorStoreConfigError::InvalidTimeout);
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VectorStoreConfigError {
    #[error("vector store namespace must not be empty when enabled")]
    MissingNamespace,
    #[error("vector store URL is required when backend is enabled")]
    MissingUrl,
    #[error("pool_size must be greater than zero")]
    InvalidPoolMax,
    #[error("health_timeout_ms must be greater than zero")]
    InvalidTimeout,
    #[error("invalid environment value: {0}")]
    InvalidEnv(String),
}

/// Aggregate capacity metrics surfaced by a backend.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorCapacitySnapshot {
    pub geom_rm: Option<f32>,
    pub geom_dm: Option<f32>,
    pub geom_rm_sqrt_dm: Option<f32>,
    pub cap_alpha_sim: Option<f32>,
}

/// Usage counters + optional capacity snapshot.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorUsageSnapshot {
    pub queries: usize,
    pub hits: usize,
    pub fallbacks: usize,
    pub capacity: Option<VectorCapacitySnapshot>,
}

/// Embedding metadata (version + dimension).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub embedding_version: String,
    pub dim: usize,
}

/// Embedding payload passed to vector stores.
#[derive(Clone, Debug, PartialEq)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub metadata: EmbeddingMetadata,
}

/// Optional trait for deterministic embedding providers.
pub trait EmbeddingProvider<I>: Send + Sync {
    fn embed(&self, input: &I) -> Embedding;
}

/// Item to be added to the index.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorItem {
    pub ref_id: String,
    pub embedding: Embedding,
}

/// Individual search hit returned by a backend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VectorSearchHit {
    pub ref_id: String,
    pub score: f32,
}

/// Search outcome with optional capacity signals.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VectorSearchResult {
    pub hits: Vec<VectorSearchHit>,
    pub capacity: Option<CapacityProxies>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CapacityProxies {
    pub geom_rm: Option<f32>,
    pub geom_dm: Option<f32>,
    pub geom_rm_sqrt_dm: Option<f32>,
    pub cap_alpha_sim: Option<f32>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VectorStoreError {
    #[error("vector store disabled")]
    Disabled,
    #[error("invalid namespace")]
    InvalidNamespace,
    #[error("backend unavailable")]
    BackendUnavailable,
    #[error("search failed: {0}")]
    SearchFailed(String),
    #[error("indexing failed: {0}")]
    IndexFailed(String),
}

/// Vector store interface implemented by platform adapters.
pub trait VectorStore: Send + Sync {
    fn backend(&self) -> VectorBackend;
    fn health(&self, namespace: &str) -> Result<(), VectorStoreError>;
    fn index_items(&self, namespace: &str, items: &[VectorItem]) -> Result<(), VectorStoreError>;
    fn search(
        &self,
        namespace: &str,
        query_vec: &[f32],
        top_k: usize,
    ) -> Result<VectorSearchResult, VectorStoreError>;
}

/// Simple counter set to collect observability signals.
#[derive(Debug, Default)]
pub struct VectorUsageCounters {
    vector_queries: AtomicUsize,
    vector_hits: AtomicUsize,
    vector_fallbacks: AtomicUsize,
}

impl VectorUsageCounters {
    pub fn inc_query(&self) {
        self.vector_queries.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_hits(&self, count: usize) {
        self.vector_hits.fetch_add(count, Ordering::Relaxed);
    }

    pub fn inc_fallback(&self) {
        self.vector_fallbacks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> VectorUsageSnapshot {
        VectorUsageSnapshot {
            queries: self.vector_queries.load(Ordering::Relaxed),
            hits: self.vector_hits.load(Ordering::Relaxed),
            fallbacks: self.vector_fallbacks.load(Ordering::Relaxed),
            capacity: None,
        }
    }
}

#[derive(Clone)]
pub struct VectorUsageHandle {
    counters: Arc<VectorUsageCounters>,
    capacity: Arc<Mutex<Option<CapacityProxies>>>,
}

impl VectorUsageHandle {
    pub fn new(
        counters: Arc<VectorUsageCounters>,
        capacity: Arc<Mutex<Option<CapacityProxies>>>,
    ) -> Self {
        Self { counters, capacity }
    }

    pub fn snapshot(&self) -> VectorUsageSnapshot {
        let mut snapshot = self.counters.snapshot();
        let capacity = self.capacity.lock().expect("capacity lock").clone();
        snapshot.capacity = capacity.map(|proxies| VectorCapacitySnapshot {
            geom_rm: proxies.geom_rm,
            geom_dm: proxies.geom_dm,
            geom_rm_sqrt_dm: proxies.geom_rm_sqrt_dm,
            cap_alpha_sim: proxies.cap_alpha_sim,
        });
        snapshot
    }
}

/// Deterministic in-memory store used in domain tests and offline mode.
#[derive(Clone)]
pub struct MockVectorStore {
    namespace: String,
    healthy: Arc<AtomicBool>,
    responses: Arc<Mutex<HashMap<String, Vec<VectorSearchHit>>>>,
    counters: Arc<VectorUsageCounters>,
    capacity: Option<CapacityProxies>,
    simulated_latency: Option<Duration>,
    last_indexed: Arc<Mutex<Vec<VectorItem>>>,
}

impl MockVectorStore {
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            healthy: Arc::new(AtomicBool::new(true)),
            responses: Arc::new(Mutex::new(HashMap::new())),
            counters: Arc::new(VectorUsageCounters::default()),
            capacity: None,
            simulated_latency: None,
            last_indexed: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_capacity(mut self, capacity: CapacityProxies) -> Self {
        self.capacity = Some(capacity);
        self
    }

    pub fn with_latency(mut self, duration: Duration) -> Self {
        self.simulated_latency = Some(duration);
        self
    }

    pub fn set_response(&self, namespace: &str, hits: Vec<VectorSearchHit>) {
        let mut guard = self.responses.lock().expect("mock responses lock");
        guard.insert(namespace.to_string(), hits);
    }

    pub fn counters(&self) -> Arc<VectorUsageCounters> {
        self.counters.clone()
    }

    pub fn last_indexed(&self) -> Vec<VectorItem> {
        self.last_indexed.lock().expect("last_indexed lock").clone()
    }

    pub fn set_healthy(&self, healthy: bool) {
        self.healthy.store(healthy, Ordering::Relaxed);
    }
}

impl VectorStore for MockVectorStore {
    fn backend(&self) -> VectorBackend {
        VectorBackend::Mock
    }

    fn health(&self, namespace: &str) -> Result<(), VectorStoreError> {
        if namespace != self.namespace {
            return Err(VectorStoreError::InvalidNamespace);
        }
        if !self.healthy.load(Ordering::Relaxed) {
            return Err(VectorStoreError::BackendUnavailable);
        }
        Ok(())
    }

    fn index_items(&self, namespace: &str, items: &[VectorItem]) -> Result<(), VectorStoreError> {
        if namespace != self.namespace {
            return Err(VectorStoreError::InvalidNamespace);
        }
        if let Some(delay) = self.simulated_latency {
            std::thread::sleep(delay);
        }
        let mut last = self.last_indexed.lock().expect("last_indexed lock");
        last.clear();
        last.extend_from_slice(items);
        Ok(())
    }

    fn search(
        &self,
        namespace: &str,
        query_vec: &[f32],
        top_k: usize,
    ) -> Result<VectorSearchResult, VectorStoreError> {
        if namespace != self.namespace {
            self.counters.inc_fallback();
            return Err(VectorStoreError::InvalidNamespace);
        }
        self.counters.inc_query();
        if let Some(delay) = self.simulated_latency {
            std::thread::sleep(delay);
        }
        if !self.healthy.load(Ordering::Relaxed) {
            self.counters.inc_fallback();
            return Err(VectorStoreError::BackendUnavailable);
        }
        let hits = self
            .responses
            .lock()
            .expect("mock responses lock")
            .get(namespace)
            .cloned()
            .unwrap_or_else(|| deterministic_hits(query_vec, top_k));
        self.counters.inc_hits(hits.len());
        Ok(VectorSearchResult {
            hits,
            capacity: self.capacity.clone(),
        })
    }
}

fn deterministic_hits(query_vec: &[f32], top_k: usize) -> Vec<VectorSearchHit> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for value in query_vec {
        value.to_bits().hash(&mut hasher);
    }
    let seed = hasher.finish();
    let mut hits = Vec::new();
    for i in 0..top_k {
        let ref_id = format!("C{:05}", (seed as usize + i) % 10_000);
        let score = 0.5 + ((seed as f32 * (i as f32 + 1.0) % 50.0) / 100.0);
        hits.push(VectorSearchHit { ref_id, score });
    }
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.ref_id.cmp(&b.ref_id))
    });
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_validation_blocks_invalid_state() {
        let config = VectorStoreConfig {
            backend: VectorBackend::Qdrant,
            url: None,
            namespace: "".into(),
            pool_max: 5,
            health_timeout_ms: 10,
            enabled: true,
        };
        assert!(matches!(
            config.validate(),
            Err(VectorStoreConfigError::MissingNamespace)
        ));
    }

    #[test]
    fn mock_store_returns_deterministic_hits() {
        let store = MockVectorStore::new("demo");
        let hits1 = store.search("demo", &[0.1, 0.2], 3).unwrap();
        let hits2 = store.search("demo", &[0.1, 0.2], 3).unwrap();
        assert_eq!(hits1.hits, hits2.hits);
    }
}
