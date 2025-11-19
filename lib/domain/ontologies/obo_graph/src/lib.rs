//! OBO graph loader and lightweight reasoning utilities for NCIt and companion ontologies.
//! See:
//! - docs/system-design/clinical/ncit/concepts/obo-graph.md
//! - docs/system-design/clinical/fhir/concepts/terminology-layer.md
//! - lib/domain/ontologies/obo_graph/README.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md (REFR-08)

mod error;
mod loader;
mod parser;
mod reasoner;
mod types;

pub use error::OboError;
pub use loader::{SUPPORTED_GRAPHS, load_ontology_graph};
pub use reasoner::CachedOntologyGraph;
pub use types::{Edge, Node, OntologyGraph, Relation};

#[cfg(test)]
mod tests;
