use std::hash::{Hash, Hasher};
use std::sync::Arc;

use dfps_core::mapping::{CodeElement, MappingCandidate};
use dfps_vector_store::{
    CapacityProxies, Embedding, EmbeddingMetadata, EmbeddingProvider, VectorStore,
    VectorStoreConfig, VectorStoreError, VectorUsageCounters, VectorUsageHandle,
};
#[cfg(feature = "obo-graph")]
use once_cell::sync::Lazy;

use crate::traits::CandidateRanker;
use crate::types::DEFAULT_VECTOR_TOP_K;

#[derive(Debug, Default)]
pub struct LexicalRanker;

#[cfg(feature = "obo-graph")]
const PET_PRIMARY_NCIT_ID: &str = "C19951";

#[cfg(feature = "obo-graph")]
fn contains_any(haystack: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

#[cfg(feature = "obo-graph")]
fn pet_synonyms() -> &'static Vec<String> {
    static SYNONYMS: Lazy<Vec<String>> =
        Lazy::new(|| dfps_terminology::synonym_set(PET_PRIMARY_NCIT_ID).unwrap_or_default());
    &SYNONYMS
}

#[cfg(feature = "obo-graph")]
fn pet_related_synonyms() -> &'static Vec<String> {
    use std::collections::BTreeSet;

    static RELATED: Lazy<Vec<String>> = Lazy::new(|| {
        let mut set = BTreeSet::new();
        for related in dfps_terminology::related_concepts(PET_PRIMARY_NCIT_ID, 2) {
            if let Some(syns) = dfps_terminology::synonym_set(&related) {
                for syn in syns {
                    set.insert(syn);
                }
            }
        }
        set.into_iter().collect()
    });
    &RELATED
}

impl CandidateRanker for LexicalRanker {
    fn rank(&self, code: &CodeElement) -> Vec<MappingCandidate> {
        let mut candidates = Vec::new();

        if let Some(display) = &code.display {
            let display_lower = display.to_ascii_lowercase();
            let mut direct_match = display_lower.contains("pet") || display_lower.contains("ct");
            #[cfg(feature = "obo-graph")]
            let mut related_match = false;

            #[cfg(feature = "obo-graph")]
            {
                if !direct_match && contains_any(&display_lower, pet_synonyms()) {
                    direct_match = true;
                }
                if !direct_match && contains_any(&display_lower, pet_related_synonyms()) {
                    related_match = true;
                }
            }

            if direct_match {
                candidates.push(MappingCandidate {
                    target_system: "NCIT".into(),
                    target_code: "C19951".into(),
                    cui: Some("C19951".into()),
                    score: 0.97,
                });
            }

            #[cfg(feature = "obo-graph")]
            if !direct_match && related_match {
                candidates.push(MappingCandidate {
                    target_system: "NCIT".into(),
                    target_code: "C19951".into(),
                    cui: Some("C19951".into()),
                    score: 0.95,
                });
            }

            if display_lower.contains("loinc") {
                candidates.push(MappingCandidate {
                    target_system: "LOINC".into(),
                    target_code: code.code.clone().unwrap_or_default(),
                    cui: None,
                    score: 0.6,
                });
            }
        }

        if candidates.is_empty() {
            candidates.push(MappingCandidate {
                target_system: code.system.clone().unwrap_or_else(|| "local-system".into()),
                target_code: code.code.clone().unwrap_or_else(|| "local-code".into()),
                cui: None,
                score: 0.4,
            });
        }

        candidates
    }
}

#[derive(Debug, Default)]
pub struct VectorRankerMock;

impl CandidateRanker for VectorRankerMock {
    fn rank(&self, code: &CodeElement) -> Vec<MappingCandidate> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        code.id.hash(&mut hasher);
        code.system.hash(&mut hasher);
        code.code.hash(&mut hasher);
        let hash = hasher.finish();
        let score = ((hash % 100) as f32) / 100.0;

        vec![MappingCandidate {
            target_system: "NCIT".into(),
            target_code: format!("C{:05}", hash % 10_000),
            cui: Some(format!("CUI{:05}", hash % 10_000)),
            score: 0.5 + (score / 2.0),
        }]
    }
}

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
pub struct DeterministicEmbeddingProvider {
    metadata: EmbeddingMetadata,
}

impl Default for DeterministicEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DeterministicEmbeddingProvider {
    pub fn new() -> Self {
        Self {
            metadata: EmbeddingMetadata {
                embedding_version: "deterministic-hash-v1".into(),
                dim: 8,
            },
        }
    }
}

impl EmbeddingProvider<CodeElement> for DeterministicEmbeddingProvider {
    fn embed(&self, input: &CodeElement) -> Embedding {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        input.id.hash(&mut hasher);
        input.system.hash(&mut hasher);
        input.code.hash(&mut hasher);
        input.display.hash(&mut hasher);
        let seed = hasher.finish();

        let mut vector = Vec::with_capacity(self.metadata.dim);
        for i in 0..self.metadata.dim {
            let component_seed = seed ^ ((i as u64 + 1) * 31);
            let component = ((component_seed % 10_000) as f32) / 10_000.0;
            vector.push(component);
        }

        Embedding {
            vector,
            metadata: self.metadata.clone(),
        }
    }
}

pub struct VectorRankerBackend<S, E> {
    store: Arc<S>,
    namespace: String,
    top_k: usize,
    embedding: E,
    enabled: bool,
    counters: Arc<VectorUsageCounters>,
    fallback: VectorRankerMock,
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
            fallback: VectorRankerMock,
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
