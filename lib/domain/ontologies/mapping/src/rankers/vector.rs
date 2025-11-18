use std::sync::Arc;

use dfps_core::mapping::{CodeElement, MappingCandidate};
use dfps_vector_store::{
    CapacityProxies, EmbeddingProvider, VectorStore, VectorStoreConfig, VectorStoreError,
    VectorUsageCounters, VectorUsageHandle,
};

use crate::traits::CandidateRanker;
use crate::types::DEFAULT_VECTOR_TOP_K;

#[derive(Debug)]
pub enum VectorRankerError {
    MissingNamespace,
    InvalidTopK,
    Store(VectorStoreError),
}

impl std::fmt::Display for VectorRankerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorRankerError::MissingNamespace => write!(f, "vector namespace required"),
            VectorRankerError::InvalidTopK => write!(f, "vector top_k must be greater than zero"),
            VectorRankerError::Store(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for VectorRankerError {}

impl From<VectorStoreError> for VectorRankerError {
    fn from(value: VectorStoreError) -> Self {
        Self::Store(value)
    }
}

#[derive(Debug, Clone)]
pub struct VectorRankerBackend<S, E> {
    store: Arc<S>,
    namespace: String,
    top_k: usize,
    embedding: E,
    enabled: bool,
    counters: Arc<VectorUsageCounters>,
    fallback: crate::rankers::VectorRankerMock,
    capacity: Arc<std::sync::Mutex<Option<CapacityProxies>>>,
}

impl<S, E> VectorRankerBackend<S, E> {
    pub fn new(
        store: Arc<S>,
        embedding: E,
        namespace: String,
        top_k: usize,
        enabled: bool,
    ) -> Result<Self, VectorRankerError> {
        if namespace.trim().is_empty() {
            return Err(VectorRankerError::MissingNamespace);
        }
        if top_k == 0 {
            return Err(VectorRankerError::InvalidTopK);
        }

        Ok(Self {
            store,
            namespace,
            top_k,
            embedding,
            enabled,
            counters: Arc::new(VectorUsageCounters::default()),
            fallback: crate::rankers::VectorRankerMock,
            capacity: Arc::new(std::sync::Mutex::new(None)),
        })
    }

    pub fn from_config(
        store: Arc<S>,
        embedding: E,
        config: VectorStoreConfig,
        top_k: usize,
    ) -> Result<Self, VectorRankerError> {
        let effective_top_k = if top_k == 0 {
            DEFAULT_VECTOR_TOP_K
        } else {
            top_k
        };
        Self::new(
            store,
            embedding,
            config.namespace,
            effective_top_k,
            config.enabled,
        )
    }

    pub fn usage_handle(&self) -> VectorUsageHandle {
        VectorUsageHandle::new(self.counters.clone(), self.capacity.clone())
    }
}

impl<S, E> CandidateRanker for VectorRankerBackend<S, E>
where
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    fn rank(&self, code: &CodeElement) -> Vec<MappingCandidate> {
        if !self.enabled {
            self.counters.inc_fallback();
            return self.fallback.rank(code);
        }

        let embedding = self.embedding.embed(code);
        self.counters.inc_query();
        match self
            .store
            .search(&self.namespace, &embedding.vector, self.top_k)
        {
            Ok(result) => {
                if result.capacity.is_some() {
                    let mut guard = self.capacity.lock().expect("capacity lock");
                    *guard = result.capacity;
                }
                self.counters.inc_hits(result.hits.len());
                let mut hits = result.hits;
                hits.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.ref_id.cmp(&b.ref_id))
                });
                hits.into_iter()
                    .map(|hit| MappingCandidate {
                        target_system: "NCIT".into(),
                        target_code: normalize_ncit_code(&hit.ref_id),
                        cui: None,
                        score: hit.score,
                    })
                    .collect()
            }
            Err(_) => {
                self.counters.inc_fallback();
                self.fallback.rank(code)
            }
        }
    }
}

pub fn normalize_ncit_code(code: &str) -> String {
    if code.starts_with("NCIT:") {
        code.to_string()
    } else if code.starts_with('C') {
        format!("NCIT:{code}")
    } else {
        format!("NCIT:C{code}")
    }
}
