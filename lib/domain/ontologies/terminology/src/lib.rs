//! License-aware terminology registries and bridges for staging/mapping flows.
//! See:
//! - docs/system-design/clinical/ncit/architecture.md
//! - lib/domain/ontologies/terminology/README.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md (REFR-08)

pub mod bridge;
pub mod client;
pub mod codesystem;
pub mod obo;
#[cfg(feature = "obo-graph")]
pub mod obo_graph;
pub mod registry;
pub mod valueset;

pub use bridge::{CodeKind, EnrichedCode, canonicalize_system, canonicalize_system_url};
#[cfg(feature = "http-client")]
pub use client::HttpTerminologyClient;
pub use client::{
    CompositeTerminologyClient, CuiRecord, MockTerminologyClient, NcitRecord, TerminologyClient,
    TerminologyClientConfig, TerminologyClientError, TerminologyMode, TerminologyResult,
};
pub use codesystem::{CodeSystemMeta, LicenseTier, SourceKind};
#[cfg(feature = "obo-graph")]
pub use obo::{
    GraphContext, GraphVersion, OboGraphError, graph_versions, related_concepts, synonym_set,
};
pub use obo::{OboOntology, list_ontologies, lookup_ontology};
pub use registry::{is_licensed, is_open, list_code_systems, lookup_codesystem};
pub use valueset::{ValueSetMeta, list_value_sets, lookup_value_set};
