pub mod bridge;
pub mod client;
pub mod codesystem;
pub mod obo;
pub mod registry;
pub mod valueset;

pub use bridge::{CodeKind, EnrichedCode};
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
