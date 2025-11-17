//! Mapping engine skeleton that composes lexical/vector heuristics with rule
//! reranking to emit NCIt-aligned `MappingResult`s.
//!
//! This crate intentionally keeps the logic deterministic and self-contained so
//! it can power golden/property tests without external services.

use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use dfps_core::{
    mapping::{
        CodeElement, DimNCITConcept, MappingCandidate, MappingResult, MappingSourceVersion,
        MappingState, MappingStrategy, MappingThresholds,
    },
    staging::StgSrCodeExploded,
};
use dfps_terminology::{CodeKind, EnrichedCode, TerminologyClient, TerminologyResult};
use dfps_vector_store::{
    CapacityProxies, Embedding, EmbeddingMetadata, EmbeddingProvider, VectorStore,
    VectorStoreConfig, VectorStoreError, VectorUsageCounters, VectorUsageHandle,
};

mod data;
pub mod eval;

pub use data::{
    NCIT_DATA_VERSION, UMLS_DATA_VERSION, UmlsXref, load_ncit_concepts, load_umls_xrefs,
};
pub use dfps_eval::{EvalCase, EvalResult, EvalSummary};
#[allow(deprecated)]
pub use eval::run_eval;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MappingSummary {
    pub total: usize,
    pub by_code_kind: BTreeMap<String, usize>,
    pub by_license_tier: BTreeMap<String, usize>,
    pub extern_lookup_success: usize,
    pub extern_lookup_miss: usize,
    pub extern_lookup_error: usize,
}

const DEFAULT_VECTOR_TOP_K: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FusionWeights {
    pub lexical: f32,
    pub vector: f32,
}

impl FusionWeights {
    pub fn new(lexical: f32, vector: f32) -> Self {
        let mut weights = Self { lexical, vector };
        weights.ensure_valid();
        weights
    }

    fn ensure_valid(&mut self) {
        if self.lexical <= 0.0 && self.vector <= 0.0 {
            self.lexical = 1.0;
            self.vector = 1.0;
        } else {
            if self.lexical <= 0.0 {
                self.lexical = 0.01;
            }
            if self.vector <= 0.0 {
                self.vector = 0.01;
            }
        }
    }
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self {
            lexical: 1.0,
            vector: 1.0,
        }
    }
}

impl MappingSummary {
    pub fn record(&mut self, kind: CodeKind, license_label: Option<&str>) {
        self.total += 1;
        let kind_key = kind.as_str().to_string();
        *self.by_code_kind.entry(kind_key).or_default() += 1;

        let license_key = license_label.unwrap_or("unknown").to_string();
        *self.by_license_tier.entry(license_key).or_default() += 1;
    }

    pub fn record_external_success(&mut self) {
        self.extern_lookup_success += 1;
    }

    pub fn record_external_miss(&mut self) {
        self.extern_lookup_miss += 1;
    }

    pub fn record_external_error(&mut self) {
        self.extern_lookup_error += 1;
    }
}

pub trait Mapper {
    fn map(&self, code: &CodeElement) -> MappingResult;
}

pub trait CandidateRanker {
    fn rank(&self, code: &CodeElement) -> Vec<MappingCandidate>;
}

#[derive(Debug, Default)]
pub struct LexicalRanker;

impl CandidateRanker for LexicalRanker {
    fn rank(&self, code: &CodeElement) -> Vec<MappingCandidate> {
        let mut candidates = Vec::new();

        if let Some(display) = &code.display {
            let display_lower = display.to_ascii_lowercase();
            if display_lower.contains("pet") || display_lower.contains("ct") {
                candidates.push(MappingCandidate {
                    target_system: "NCIT".into(),
                    target_code: "C19951".into(),
                    cui: Some("C19951".into()),
                    score: 0.92,
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
        let search = self
            .store
            .search(&self.namespace, &embedding.vector, self.top_k);

        match search {
            Ok(result) => {
                self.counters.inc_query();
                self.counters.inc_hits(result.hits.len());
                if let Some(capacity) = result.capacity {
                    if let Ok(mut guard) = self.capacity.lock() {
                        *guard = Some(capacity);
                    }
                }
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

#[derive(Debug, Default)]
pub struct RuleReranker;

impl RuleReranker {
    pub fn apply(&self, candidates: &mut [MappingCandidate]) {
        for candidate in candidates {
            if candidate.target_system == "NCIT" {
                candidate.score = (candidate.score + 0.05).min(1.0);
            } else if candidate.target_system.contains("SNOMED")
                || candidate.target_system.contains("CPT")
            {
                candidate.score = (candidate.score + 0.02).min(1.0);
            }
        }
    }
}

pub struct MappingEngine<L, V> {
    lexical: L,
    vector: V,
    rules: RuleReranker,
    fusion_weights: FusionWeights,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MappingExplanation {
    pub code_element: CodeElement,
    pub candidates: Vec<MappingCandidate>,
}

impl<L, V> MappingEngine<L, V>
where
    L: CandidateRanker,
    V: CandidateRanker,
{
    pub fn new(lexical: L, vector: V, rules: RuleReranker) -> Self {
        Self::new_with_weights(lexical, vector, rules, FusionWeights::default())
    }

    pub fn new_with_weights(
        lexical: L,
        vector: V,
        rules: RuleReranker,
        fusion_weights: FusionWeights,
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

    /// Expose ranked candidates for diagnostics/tests.
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
}

impl<L, V> Mapper for MappingEngine<L, V>
where
    L: CandidateRanker,
    V: CandidateRanker,
{
    fn map(&self, code: &CodeElement) -> MappingResult {
        let candidates = self.collect_candidates(code);
        let top = candidates.first().cloned().unwrap_or(MappingCandidate {
            target_system: "NCIT".into(),
            target_code: "C00000".into(),
            cui: None,
            score: 0.0,
        });

        build_result_with_score(
            code,
            top.cui.clone(),
            Some(normalize_ncit_code(&top.target_code)),
            top.score,
            MappingStrategy::Composite,
            None,
        )
    }
}

fn normalize_ncit_code(code: &str) -> String {
    if code.starts_with("NCIT:") {
        code.to_string()
    } else if code.starts_with('C') {
        format!("NCIT:{code}")
    } else {
        format!("NCIT:C{code}")
    }
}

pub fn default_engine() -> MappingEngine<LexicalRanker, VectorRankerMock> {
    MappingEngine::new(LexicalRanker, VectorRankerMock, RuleReranker)
}

/// Build a vector-enabled engine from a provided store/config, keeping lexical as a baseline.
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

/// Build a vector-enabled engine using the shared VectorStoreConfig environment.
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

fn default_thresholds() -> MappingThresholds {
    MappingThresholds::default()
}

fn classify(score: f32, thresholds: &MappingThresholds) -> MappingState {
    if score >= thresholds.auto_map_min {
        MappingState::AutoMapped
    } else if score >= thresholds.needs_review_min {
        MappingState::NeedsReview
    } else {
        MappingState::NoMatch
    }
}

fn source_versions() -> MappingSourceVersion {
    MappingSourceVersion::new(NCIT_DATA_VERSION, UMLS_DATA_VERSION)
}

fn build_result_with_score(
    code: &CodeElement,
    cui: Option<String>,
    ncit_id: Option<String>,
    score: f32,
    strategy: MappingStrategy,
    reason: Option<String>,
) -> MappingResult {
    let thresholds = default_thresholds();
    let state = classify(score, &thresholds);
    let mut final_ncit = ncit_id;
    let final_reason = if state == MappingState::NoMatch {
        final_ncit = None;
        Some(reason.unwrap_or_else(|| "score_below_threshold".into()))
    } else {
        reason
    };
    MappingResult {
        code_element_id: code.id.clone(),
        cui,
        ncit_id: final_ncit,
        score,
        strategy,
        state,
        thresholds,
        source_version: source_versions(),
        reason: final_reason,
        license_tier: None,
        source_kind: None,
    }
}

fn attach_license_metadata(result: &mut MappingResult, enriched: &EnrichedCode) {
    if let Some(label) = enriched.license_label() {
        result.license_tier = Some(label.to_string());
    }
    if let Some(label) = enriched.source_label() {
        result.source_kind = Some(label.to_string());
    }
}

pub fn map_staging_codes<I>(codes: I) -> (Vec<MappingResult>, Vec<DimNCITConcept>)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let (results, dims, _) = map_staging_codes_with_summary(codes);
    (results, dims)
}

pub fn map_staging_codes_with_summary<I>(
    codes: I,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    map_staging_codes_with_summary_with_client(codes, None)
}

pub fn map_staging_codes_with_summary_with_client<I>(
    codes: I,
    client: Option<&dyn TerminologyClient>,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let dim_concepts = dim_concepts();
    let xrefs = load_umls_xrefs();
    let engine = default_engine();
    let (results, summary) = map_with_engine(codes.into_iter(), &engine, &xrefs, client);
    (results, dim_concepts, summary)
}

pub fn map_staging_codes_with_vector<I, S, E>(
    codes: I,
    store: Arc<S>,
    config: VectorStoreConfig,
    embedding: E,
    top_k: usize,
) -> Result<
    (
        Vec<MappingResult>,
        Vec<DimNCITConcept>,
        MappingSummary,
        dfps_vector_store::VectorUsageSnapshot,
    ),
    VectorRankerError,
>
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    let dim_concepts = dim_concepts();
    let xrefs = load_umls_xrefs();
    let vector_ranker = VectorRankerBackend::from_config(store, embedding, config, top_k)?;
    let usage_handle = vector_ranker.usage_handle();
    let engine = MappingEngine::new(LexicalRanker, vector_ranker, RuleReranker);
    let (results, summary) = map_with_engine(
        codes.into_iter(),
        &engine,
        &xrefs,
        None as Option<&dyn TerminologyClient>,
    );
    let snapshot = usage_handle.snapshot();
    Ok((results, dim_concepts, summary, snapshot))
}

fn dim_concepts() -> Vec<DimNCITConcept> {
    let concepts = load_ncit_concepts();
    let mut seen = HashSet::new();
    let mut dim_concepts = Vec::new();
    for (_, dim) in concepts {
        if seen.insert(dim.ncit_id.clone()) {
            dim_concepts.push(dim);
        }
    }
    dim_concepts
}

fn map_with_engine<I, L, V>(
    codes: I,
    engine: &MappingEngine<L, V>,
    xrefs: &std::collections::HashMap<(String, String), UmlsXref>,
    client: Option<&dyn TerminologyClient>,
) -> (Vec<MappingResult>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    L: CandidateRanker,
    V: CandidateRanker,
{
    let mut results = Vec::new();
    let mut summary = MappingSummary::default();

    for staging in codes {
        let enriched = EnrichedCode::from_staging(staging.clone());
        let code_kind = enriched.code_kind();
        let element = CodeElement::from(staging);
        let system_value = enriched.staging.system.clone().unwrap_or_default();
        let code_value = enriched.staging.code.clone().unwrap_or_default();
        let key = (system_value.clone(), code_value.clone());

        summary.record(code_kind, enriched.license_label());

        let mut result = match code_kind {
            CodeKind::MissingSystemOrCode => build_result_with_score(
                &element,
                None,
                None,
                0.0,
                MappingStrategy::Unmapped,
                Some("missing_system_or_code".into()),
            ),
            CodeKind::UnknownSystem => {
                let base = build_result_with_score(
                    &element,
                    None,
                    None,
                    0.0,
                    MappingStrategy::Unmapped,
                    Some("unknown_code_system".into()),
                );
                if let Some(client) = client {
                    match external_lookup(client, &system_value, &code_value) {
                        Ok(Some((cui, ncit_id))) => {
                            summary.record_external_success();
                            build_result_with_score(
                                &element,
                                cui,
                                ncit_id,
                                0.95,
                                MappingStrategy::Rule,
                                Some("external_terminology_lookup".into()),
                            )
                        }
                        Ok(None) => {
                            summary.record_external_miss();
                            base
                        }
                        Err(_) => {
                            summary.record_external_error();
                            base
                        }
                    }
                } else {
                    base
                }
            }
            _ => {
                if let Some(xref) = xrefs.get(&key) {
                    build_result_with_score(
                        &element,
                        Some(xref.cui.clone()),
                        Some(xref.ncit_id.clone()),
                        0.99,
                        MappingStrategy::Rule,
                        Some("umls_direct_xref".into()),
                    )
                } else {
                    engine.map(&element)
                }
            }
        };

        attach_license_metadata(&mut result, &enriched);
        results.push(result);
    }

    (results, summary)
}

fn external_lookup(
    client: &dyn TerminologyClient,
    system: &str,
    code: &str,
) -> TerminologyResult<Option<(Option<String>, Option<String>)>> {
    let mut cui = None;
    if let Some(record) = client.lookup_cui(system, code)? {
        cui = Some(record.cui);
    }

    let ncit_id = client
        .lookup_ncit(cui.as_deref().unwrap_or(code))?
        .map(|record| record.ncit_id);

    if cui.is_some() || ncit_id.is_some() {
        Ok(Some((cui, ncit_id)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dfps_core::staging::StgSrCodeExploded;
    use dfps_terminology::{MockTerminologyClient, TerminologyClient};
    use dfps_vector_store::{MockVectorStore, VectorBackend, VectorSearchHit, VectorStoreConfig};
    use std::sync::Arc;

    #[test]
    fn engine_returns_deterministic_result() {
        let staging = StgSrCodeExploded {
            sr_id: "SR-1".into(),
            system: Some("http://snomed.info/sct".into()),
            code: Some("123".into()),
            display: Some("PET CT staging".into()),
        };
        let code = CodeElement::from(staging);
        let engine = MappingEngine::new(LexicalRanker, VectorRankerMock, RuleReranker);

        let result = engine.map(&code);
        assert!(result.score > 0.5);
        assert!(result.ncit_id.unwrap().starts_with("NCIT:"));
        assert_ne!(result.state, MappingState::NoMatch);
    }

    #[test]
    fn summary_tracks_code_kind_and_license_counts() {
        let codes = vec![
            StgSrCodeExploded {
                sr_id: "SR-1".into(),
                system: Some("http://www.ama-assn.org/go/cpt".into()),
                code: Some("78815".into()),
                display: None,
            },
            StgSrCodeExploded {
                sr_id: "SR-2".into(),
                system: Some("http://example.org/custom".into()),
                code: Some("A1".into()),
                display: None,
            },
            StgSrCodeExploded {
                sr_id: "SR-3".into(),
                system: None,
                code: Some("B1".into()),
                display: None,
            },
        ];

        let (_, _, summary) = map_staging_codes_with_summary(codes);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_code_kind.get("known_licensed_system"), Some(&1));
        assert_eq!(summary.by_code_kind.get("unknown_system"), Some(&1));
        assert_eq!(summary.by_code_kind.get("missing_system_or_code"), Some(&1));
        assert_eq!(summary.by_license_tier.get("licensed"), Some(&1));
        assert_eq!(summary.by_license_tier.get("unknown"), Some(&2));
    }

    #[test]
    fn unknown_system_uses_external_client() {
        let codes = vec![StgSrCodeExploded {
            sr_id: "SR-1".into(),
            system: Some("http://unknown.test/system".into()),
            code: Some("X1".into()),
            display: Some("Unknown".into()),
        }];
        let mut mock = MockTerminologyClient::default();
        mock = mock.with_cui(
            "http://unknown.test/system",
            "X1",
            dfps_terminology::CuiRecord {
                cui: "CEXTERNAL".into(),
                preferred_name: "External".into(),
            },
        );
        mock = mock.with_ncit(
            "C9999",
            dfps_terminology::NcitRecord {
                ncit_id: "C9999".into(),
                preferred_name: "External NCIt".into(),
                synonyms: vec![],
            },
        );
        let (results, _dims, summary) =
            map_staging_codes_with_summary_with_client(codes, Some(&mock));
        assert_eq!(summary.extern_lookup_success, 1);
        let result = &results[0];
        assert_eq!(result.strategy, MappingStrategy::Rule);
        assert_eq!(
            result.reason.as_deref(),
            Some("external_terminology_lookup")
        );
    }

    #[test]
    fn vector_ranker_uses_backend_hits_and_normalizes_codes() {
        let store = Arc::new(MockVectorStore::new("ncit_dev"));
        store.set_response(
            "ncit_dev",
            vec![
                VectorSearchHit {
                    ref_id: "C12345".into(),
                    score: 0.72,
                },
                VectorSearchHit {
                    ref_id: "NCIT:C00010".into(),
                    score: 0.81,
                },
            ],
        );
        let config = VectorStoreConfig {
            backend: VectorBackend::Mock,
            url: None,
            namespace: "ncit_dev".into(),
            pool_max: 4,
            health_timeout_ms: 250,
            enabled: true,
        };
        let ranker = VectorRankerBackend::from_config(
            store.clone(),
            DeterministicEmbeddingProvider::new(),
            config,
            3,
        )
        .expect("vector ranker");
        let usage = ranker.usage_handle();
        let engine = MappingEngine::new(LexicalRanker, ranker, RuleReranker);
        let staging = StgSrCodeExploded {
            sr_id: "SR-vec".into(),
            system: Some("http://ncit.nci.nih.gov".into()),
            code: Some("C19951".into()),
            display: Some("PET CT staging".into()),
        };
        let code = CodeElement::from(staging);
        let candidates = engine.ranked_candidates(&code);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.target_code == "NCIT:C00010")
        );
        assert!(candidates[0].score >= candidates[1].score);
        let snapshot = usage.snapshot();
        assert_eq!(snapshot.capacity, None);
    }

    #[test]
    fn vector_ranker_captures_capacity_proxies_from_backend() {
        let store = Arc::new(
            MockVectorStore::new("ncit_dev").with_capacity(CapacityProxies {
                geom_rm_sqrt_dm: Some(1.25),
                cap_alpha_sim: Some(0.92),
                ..CapacityProxies::default()
            }),
        );
        let config = VectorStoreConfig {
            backend: VectorBackend::Mock,
            url: None,
            namespace: "ncit_dev".into(),
            pool_max: 2,
            health_timeout_ms: 200,
            enabled: true,
        };
        let ranker = VectorRankerBackend::from_config(
            store.clone(),
            DeterministicEmbeddingProvider::new(),
            config,
            2,
        )
        .expect("vector ranker");
        let usage = ranker.usage_handle();
        let engine = MappingEngine::new(LexicalRanker, ranker, RuleReranker);
        let code = CodeElement::new(
            "SR-capacity",
            Some("http://loinc.org".into()),
            Some("1111-1".into()),
            Some("Example display".into()),
        );
        let _ = engine.ranked_candidates(&code);
        let snapshot = usage.snapshot();
        assert!(snapshot.capacity.is_some());
        let capacity = snapshot.capacity.unwrap();
        assert_eq!(capacity.geom_rm_sqrt_dm, Some(1.25));
        assert_eq!(capacity.cap_alpha_sim, Some(0.92));
    }

    #[test]
    fn vector_ranker_disabled_falls_back_to_mock() {
        let store = Arc::new(MockVectorStore::new("ncit_dev"));
        let config = VectorStoreConfig {
            backend: VectorBackend::Mock,
            url: None,
            namespace: "ncit_dev".into(),
            pool_max: 1,
            health_timeout_ms: 100,
            enabled: false,
        };
        let ranker = VectorRankerBackend::from_config(
            store.clone(),
            DeterministicEmbeddingProvider::new(),
            config,
            0,
        )
        .expect("vector ranker");
        let usage = ranker.usage_handle();
        let engine = MappingEngine::new(LexicalRanker, ranker, RuleReranker);
        let code = CodeElement::new(
            "SR-raw",
            Some("http://example.org/system".into()),
            Some("A1".into()),
            Some("Example display".into()),
        );
        let candidates = engine.ranked_candidates(&code);
        assert!(!candidates.is_empty());
        let snapshot = usage.snapshot();
        assert_eq!(snapshot.fallbacks, 1);
    }

    #[derive(Clone)]
    struct FixedRanker {
        candidates: Vec<MappingCandidate>,
    }

    impl CandidateRanker for FixedRanker {
        fn rank(&self, _code: &CodeElement) -> Vec<MappingCandidate> {
            self.candidates.clone()
        }
    }

    #[test]
    fn fusion_weights_shift_scores_between_lexical_and_vector() {
        let lexical = FixedRanker {
            candidates: vec![MappingCandidate {
                target_system: "NCIT".into(),
                target_code: "LEX".into(),
                cui: None,
                score: 0.9,
            }],
        };
        let vector = FixedRanker {
            candidates: vec![MappingCandidate {
                target_system: "NCIT".into(),
                target_code: "VEC".into(),
                cui: None,
                score: 0.8,
            }],
        };
        let engine = MappingEngine::new_with_weights(
            lexical.clone(),
            vector.clone(),
            RuleReranker,
            FusionWeights::new(0.1, 1.0),
        );
        let code = CodeElement::new("id", Some("sys".into()), Some("code".into()), None);
        let ranked = engine.ranked_candidates(&code);
        assert_eq!(ranked[0].target_code, "VEC");
    }
}
