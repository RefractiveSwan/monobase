//! Mapping engine skeleton that composes lexical/vector heuristics with rule
//! reranking to emit NCIt-aligned `MappingResult`s.
//!
//! Roles:
//! - `Mapper`/`MappingEngine` orchestrate lexical + vector rankers with rule tweaks
//! - `CandidateRanker` is the trait for rankers (`LexicalRanker`, `VectorRankerBackend`)
//! - Compliance/policy is injected via `MappingConfig` (pure `Policy`, no env reads)
//! - Terminology/vector backends are trait-based (`TerminologyClient`, `VectorStore`)
//!
//! See:
//! - docs/system-design/clinical/ncit/architecture.md
//! - docs/system-design/clinical/ncit/models/data-model-er.md
//! - docs/system-design/clinical/fhir/overview.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-06--domain-mapping-engine--ncit-integration
//!
//! This crate stays deterministic and self-contained so golden/property tests can
//! run without external services or environment configuration.

pub mod config;
mod data;
pub mod engine;
pub mod eval;
pub mod pipelines;
pub mod rankers;
pub mod traits;
pub mod types;

pub use config::MappingConfig;
pub use data::{
    NCIT_DATA_VERSION, UMLS_DATA_VERSION, UmlsXref, load_ncit_concepts, load_umls_xrefs,
};
pub use dfps_eval::{EvalCase, EvalResult, EvalSummary};
pub use engine::{
    MappingEngine, MappingExplanation, RuleReranker, default_engine, explain_staging_code,
    vector_engine, vector_engine_from_config,
};
#[allow(deprecated)]
pub use eval::run_eval;
pub use pipelines::{
    map_staging_codes, map_staging_codes_with_summary, map_staging_codes_with_summary_and_policy,
    map_staging_codes_with_summary_with_client, map_staging_codes_with_summary_with_config,
    map_staging_codes_with_summary_with_policy, map_staging_codes_with_vector,
    map_staging_codes_with_vector_and_config, map_staging_codes_with_vector_and_policy,
};
pub use rankers::{
    DeterministicEmbeddingProvider, LexicalRanker, VectorRankerBackend, VectorRankerError,
    VectorRankerMock,
};
pub use traits::{CandidateRanker, Mapper};
pub use types::{FusionWeights, MappingSummary};
