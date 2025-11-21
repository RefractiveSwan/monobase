use refractive_swan_core::mapping::{CodeElement, MappingCandidate};
use refractive_swan_vector_port::{Embedding, EmbeddingMetadata, EmbeddingProvider};

use crate::traits::CandidateRanker;

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
        use std::hash::{Hash, Hasher};
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

#[derive(Debug, Default, Clone)]
pub struct VectorRankerMock;

impl CandidateRanker for VectorRankerMock {
    fn rank(&self, code: &CodeElement) -> Vec<MappingCandidate> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
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
