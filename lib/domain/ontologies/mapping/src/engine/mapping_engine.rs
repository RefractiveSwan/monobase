use std::sync::Arc;

use refractive_swan_core::mapping::{
    CodeElement, MappingCandidate, MappingResult, MappingStrategy,
};
use refractive_swan_core::staging::StgSrCodeExploded;
use refractive_swan_vector_port::{EmbeddingProvider, VectorStore, VectorStoreConfig};

use crate::config::MappingConfig;
use crate::engine::reranker::RuleReranker;
use crate::rankers::{LexicalRanker, VectorRankerBackend, VectorRankerError, VectorRankerMock};
use crate::traits::{CandidateRanker, Mapper};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MappingExplanation {
    pub code_element: CodeElement,
    pub candidates: Vec<MappingCandidate>,
}

pub struct MappingEngine<L, V> {
    lexical: L,
    vector: V,
    rules: RuleReranker,
    fusion_weights: crate::types::FusionWeights,
}

impl<L, V> MappingEngine<L, V>
where
    L: CandidateRanker,
    V: CandidateRanker,
{
    pub fn new(lexical: L, vector: V, rules: RuleReranker) -> Self {
        Self::new_with_weights(
            lexical,
            vector,
            rules,
            crate::types::FusionWeights::default(),
        )
    }

    pub fn new_with_weights(
        lexical: L,
        vector: V,
        rules: RuleReranker,
        fusion_weights: crate::types::FusionWeights,
    ) -> Self {
        Self {
            lexical,
            vector,
            rules,
            fusion_weights,
        }
    }

    fn collect_candidates(&self, code: &CodeElement) -> Vec<MappingCandidate> {
        let mut combined = Vec::new();
        for mut candidate in self.lexical.rank(code) {
            candidate.score *= self.fusion_weights.lexical;
            combined.push(candidate);
        }
        for mut candidate in self.vector.rank(code) {
            candidate.score *= self.fusion_weights.vector;
            combined.push(candidate);
        }
        self.rules.apply(&mut combined);
        combined.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        combined
    }

    pub fn ranked_candidates(&self, code: &CodeElement) -> Vec<MappingCandidate> {
        self.collect_candidates(code)
    }

    pub fn explain(&self, code: &CodeElement, top_n: usize) -> MappingExplanation {
        let mut candidates = self.collect_candidates(code);
        if candidates.len() > top_n {
            candidates.truncate(top_n);
        }
        MappingExplanation {
            code_element: code.clone(),
            candidates,
        }
    }

    pub fn map_with_config(&self, code: &CodeElement, config: &MappingConfig) -> MappingResult {
        let candidates = self.collect_candidates(code);
        let top = candidates.first().cloned().unwrap_or(MappingCandidate {
            target_system: "NCIT".into(),
            target_code: "C00000".into(),
            cui: None,
            score: 0.0,
        });

        crate::pipelines::build_result_with_score(
            code,
            top.cui.clone(),
            Some(crate::rankers::normalize_ncit_code(&top.target_code)),
            top.score,
            MappingStrategy::Composite,
            None,
            config,
        )
    }
}

impl<L, V> Mapper for MappingEngine<L, V>
where
    L: CandidateRanker,
    V: CandidateRanker,
{
    fn map(&self, code: &CodeElement) -> MappingResult {
        self.map_with_config(code, &MappingConfig::default())
    }
}

pub fn default_engine() -> MappingEngine<LexicalRanker, VectorRankerMock> {
    MappingEngine::new(LexicalRanker, VectorRankerMock, RuleReranker)
}

pub fn vector_engine<S, E>(
    store: Arc<S>,
    namespace: String,
    top_k: usize,
    embedding: E,
    enabled: bool,
) -> Result<MappingEngine<LexicalRanker, VectorRankerBackend<S, E>>, VectorRankerError>
where
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    let ranker = VectorRankerBackend::new(store, embedding, namespace, top_k, enabled)?;
    Ok(MappingEngine::new(LexicalRanker, ranker, RuleReranker))
}

pub fn vector_engine_from_config<S, E>(
    store: Arc<S>,
    config: VectorStoreConfig,
    embedding: E,
    top_k: usize,
) -> Result<MappingEngine<LexicalRanker, VectorRankerBackend<S, E>>, VectorRankerError>
where
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    let ranker = VectorRankerBackend::from_config(store, embedding, config, top_k)?;
    Ok(MappingEngine::new(LexicalRanker, ranker, RuleReranker))
}

pub fn explain_staging_code(staging: &StgSrCodeExploded, top_n: usize) -> MappingExplanation {
    let engine = default_engine();
    let code = CodeElement::from(staging);
    engine.explain(&code, top_n)
}
