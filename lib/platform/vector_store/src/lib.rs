//! Vector-backed candidate retrieval abstraction.
//!
//! This crate defines the `VectorStore` trait plus config and mock helpers so
//! `dfps_mapping` can toggle vector-enabled ranking without breaking offline
//! determinism. It corresponds to the vector layer described in
//! `docs/system-design/clinical/ncit/concepts/vector-layer.md`.

use std::{
    collections::HashMap,
    env,
    hash::{Hash, Hasher},
    str::FromStr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_POOL_MAX: u32 = 5;
const DEFAULT_HEALTH_TIMEOUT_MS: u64 = 500;

/// Supported backends for the vector store.
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

/// Configuration derived from environment variables.
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
    pub fn from_env() -> Result<Self, VectorStoreConfigError> {
        // Best-effort env load; errors bubble only when parsing specific vars.
        let _ = dfps_configuration::load_env("platform.vector_store");

        let enabled = dfps_configuration::bool_var("DFPS_VECTOR_ENABLED")
            .map_err(|err| VectorStoreConfigError::InvalidEnv(err.to_string()))?
            .unwrap_or(false);
        let backend = env::var("DFPS_VECTOR_BACKEND")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(VectorBackend::Mock);
        let url = env::var("DFPS_VECTOR_URL").ok();
        let namespace = env::var("DFPS_VECTOR_NAMESPACE").unwrap_or_else(|_| "default".into());
        let pool_max = dfps_configuration::u32_var("DFPS_VECTOR_POOL_MAX")
            .map_err(|err| VectorStoreConfigError::InvalidEnv(err.to_string()))?
            .unwrap_or(DEFAULT_POOL_MAX);
        let health_timeout_ms = dfps_configuration::u64_var("DFPS_VECTOR_HEALTH_TIMEOUT_MS")
            .map_err(|err| VectorStoreConfigError::InvalidEnv(err.to_string()))?
            .unwrap_or(DEFAULT_HEALTH_TIMEOUT_MS);

        let config = Self {
            backend,
            url,
            namespace,
            pool_max,
            health_timeout_ms,
            enabled,
        };
        config.validate()?;
        Ok(config)
    }

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
pub struct CapacityProxies {
    pub geom_rm: Option<f32>,
    pub geom_dm: Option<f32>,
    pub geom_rm_sqrt_dm: Option<f32>,
    pub cap_alpha_sim: Option<f32>,
}

/// Metadata for an embedding function.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub embedding_version: String,
    pub dim: usize,
}

/// Embedding payload passed to vector backends.
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

/// Search outcome with optional capacity/correlation signals.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VectorSearchResult {
    pub hits: Vec<VectorSearchHit>,
    pub capacity: Option<CapacityProxies>,
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

/// Vector store interface. Implementations should be Send + Sync to plug into
/// ranking pipelines.
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorUsageSnapshot {
    pub queries: usize,
    pub hits: usize,
    pub fallbacks: usize,
    pub capacity: Option<CapacityProxies>,
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
        snapshot.capacity = self.capacity.lock().expect("capacity lock").clone();
        snapshot
    }
}

/// In-memory mock store for tests and offline mode.
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

    /// For tests: return the last batch passed to `index_items`.
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
        if self.healthy.load(Ordering::Relaxed) {
            Ok(())
        } else {
            Err(VectorStoreError::BackendUnavailable)
        }
    }

    fn index_items(&self, namespace: &str, _items: &[VectorItem]) -> Result<(), VectorStoreError> {
        if namespace != self.namespace {
            return Err(VectorStoreError::InvalidNamespace);
        }
        if let Some(first) = _items.first() {
            let dim = first.embedding.vector.len();
            if !_items.iter().all(|item| item.embedding.vector.len() == dim) {
                return Err(VectorStoreError::IndexFailed(
                    "dimension mismatch in batch".into(),
                ));
            }
        }
        *self.last_indexed.lock().expect("last index lock") = _items.to_vec();
        Ok(())
    }

    fn search(
        &self,
        namespace: &str,
        query_vec: &[f32],
        top_k: usize,
    ) -> Result<VectorSearchResult, VectorStoreError> {
        if self.simulated_latency.is_some() {
            std::thread::sleep(self.simulated_latency.unwrap());
        }

        if namespace != self.namespace {
            self.counters.inc_fallback();
            return Err(VectorStoreError::InvalidNamespace);
        }

        self.counters.inc_query();

        if !self.healthy.load(Ordering::Relaxed) {
            self.counters.inc_fallback();
            return Err(VectorStoreError::BackendUnavailable);
        }

        let hits = {
            let guard = self.responses.lock().expect("mock responses lock");
            guard
                .get(namespace)
                .cloned()
                .unwrap_or_else(|| deterministic_hits(query_vec, top_k))
        };

        let mut hits = hits;
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.ref_id.cmp(&b.ref_id))
        });

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

#[cfg(feature = "backend-qdrant")]
mod qdrant;
#[cfg(feature = "backend-qdrant")]
pub use qdrant::QdrantVectorStore;

#[cfg(feature = "backend-pgvector")]
mod pgvector_backend;
#[cfg(feature = "backend-pgvector")]
pub use pgvector_backend::PgVectorStore;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    fn reset_env() {
        for key in [
            "DFPS_VECTOR_ENABLED",
            "DFPS_VECTOR_BACKEND",
            "DFPS_VECTOR_URL",
            "DFPS_VECTOR_NAMESPACE",
            "DFPS_VECTOR_POOL_MAX",
            "DFPS_VECTOR_HEALTH_TIMEOUT_MS",
        ] {
            unsafe {
                env::remove_var(key);
            }
        }
    }

    #[test]
    fn config_defaults_to_mock_when_disabled() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        unsafe {
            env::set_var("DFPS_VECTOR_NAMESPACE", "ncit_dev");
        }
        let config = VectorStoreConfig::from_env().expect("config");
        assert_eq!(config.backend, VectorBackend::Mock);
        assert_eq!(config.namespace, "ncit_dev");
        assert!(!config.enabled);
    }

    #[test]
    fn config_rejects_empty_namespace_when_enabled() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        unsafe {
            env::set_var("DFPS_VECTOR_ENABLED", "true");
            env::set_var("DFPS_VECTOR_NAMESPACE", "   ");
        }
        let err = VectorStoreConfig::from_env().unwrap_err();
        assert_eq!(err, VectorStoreConfigError::MissingNamespace);
    }

    #[test]
    fn mock_store_enforces_namespace_and_records_fallbacks() {
        let store = MockVectorStore::new("ncit_dev");
        let result = store.search("other", &[0.1, 0.2], 2);
        assert!(matches!(result, Err(VectorStoreError::InvalidNamespace)));
        let counters = store.counters();
        let snapshot = counters.snapshot();
        assert_eq!(snapshot.queries, 0); // queries
        assert_eq!(snapshot.hits, 0); // hits
        assert_eq!(snapshot.fallbacks, 1); // fallbacks
    }

    #[test]
    fn deterministic_hits_ordered_on_ties() {
        let store = MockVectorStore::new("ncit_dev");
        store.set_response(
            "ncit_dev",
            vec![
                VectorSearchHit {
                    ref_id: "C10002".into(),
                    score: 0.75,
                },
                VectorSearchHit {
                    ref_id: "C01000".into(),
                    score: 0.75,
                },
            ],
        );
        let result = store.search("ncit_dev", &[0.3, 0.4], 2).expect("search");
        assert_eq!(result.hits[0].ref_id, "C01000");
        assert_eq!(result.hits[1].ref_id, "C10002");
    }

    #[test]
    fn mock_index_is_idempotent_for_same_batch() {
        let store = MockVectorStore::new("ncit_dev");
        let items = vec![
            VectorItem {
                ref_id: "C1".into(),
                embedding: Embedding {
                    vector: vec![0.1, 0.2, 0.3],
                    metadata: EmbeddingMetadata {
                        embedding_version: "v1".into(),
                        dim: 3,
                    },
                },
            },
            VectorItem {
                ref_id: "C2".into(),
                embedding: Embedding {
                    vector: vec![0.4, 0.5, 0.6],
                    metadata: EmbeddingMetadata {
                        embedding_version: "v1".into(),
                        dim: 3,
                    },
                },
            },
        ];
        store
            .index_items("ncit_dev", &items)
            .expect("initial index");
        let first = store.last_indexed();
        store.index_items("ncit_dev", &items).expect("second index");
        let second = store.last_indexed();
        assert_eq!(first, second);
    }
}
