use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "obo-graph")]
use crate::obo_graph::{CachedOntologyGraph, load_ontology_graph};
#[cfg(feature = "obo-graph")]
use std::sync::OnceLock;

/// Minimal metadata for an OBO Foundry ontology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OboOntology {
    pub id: &'static str,
    pub name: &'static str,
    pub iri: &'static str,
    pub description: &'static str,
}

pub const NCIT_OBO: OboOntology = OboOntology {
    id: "NCIT",
    name: "NCI Thesaurus OBO",
    iri: "http://purl.obolibrary.org/obo/ncit.owl",
    description: "National Cancer Institute Thesaurus (OBO distribution).",
};

pub const MONDO_OBO: OboOntology = OboOntology {
    id: "MONDO",
    name: "Mondo Disease Ontology",
    iri: "http://purl.obolibrary.org/obo/mondo.owl",
    description: "Unified disease ontology combining multiple sources.",
};

static ONTOLOGIES: [OboOntology; 2] = [NCIT_OBO, MONDO_OBO];

pub fn list_ontologies() -> &'static [OboOntology] {
    &ONTOLOGIES
}

pub fn lookup_ontology(value: &str) -> Option<&'static OboOntology> {
    ONTOLOGIES.iter().find(|ont| {
        ont.id.eq_ignore_ascii_case(value)
            || ont.iri.eq_ignore_ascii_case(value)
            || value.eq_ignore_ascii_case(ont.name)
    })
}

#[derive(Debug, Error)]
pub enum OboGraphError {
    #[error("obo graph load failed: {0}")]
    Load(String),
}

#[cfg(feature = "obo-graph")]
#[derive(Debug, Clone)]
pub struct GraphContext {
    id: &'static str,
    version: String,
    graph: CachedOntologyGraph,
}

#[cfg(feature = "obo-graph")]
impl GraphContext {
    pub fn id(&self) -> &str {
        self.id
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn synonym_set(&self, ncit_id: &str) -> Option<Vec<String>> {
        self.graph.synonym_set(ncit_id)
    }

    pub fn ancestors(&self, iri: &str) -> Vec<String> {
        self.graph.ancestors(iri)
    }

    pub fn related_concepts(&self, ncit_id: &str, max_hops: usize) -> Vec<String> {
        self.graph.related_concepts(ncit_id, max_hops)
    }
}

#[cfg(feature = "obo-graph")]
static NCIT_GRAPH: OnceLock<Result<GraphContext, OboGraphError>> = OnceLock::new();
#[cfg(feature = "obo-graph")]
static MONDO_GRAPH: OnceLock<Result<GraphContext, OboGraphError>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphVersion {
    pub id: &'static str,
    pub version: String,
}

#[cfg(feature = "obo-graph")]
pub fn graph_versions() -> Vec<GraphVersion> {
    [ncit_graph(), mondo_graph()]
        .into_iter()
        .filter_map(Result::ok)
        .map(|ctx| GraphVersion {
            id: ctx.id,
            version: ctx.version.clone(),
        })
        .collect()
}

#[cfg(feature = "obo-graph")]
pub fn synonym_set(ncit_id: &str) -> Option<Vec<String>> {
    let mut merged = Vec::new();

    if let Some(set) = ncit_graph().ok().and_then(|ctx| ctx.synonym_set(ncit_id)) {
        merged.extend(set);
    }
    if let Some(set) = mondo_graph().ok().and_then(|ctx| ctx.synonym_set(ncit_id)) {
        merged.extend(set);
    }

    if merged.is_empty() {
        None
    } else {
        merged.sort();
        merged.dedup();
        Some(merged)
    }
}

#[cfg(feature = "obo-graph")]
pub fn related_concepts(ncit_id: &str, max_hops: usize) -> Vec<String> {
    let mut merged = Vec::new();
    if let Some(ids) = ncit_graph()
        .ok()
        .map(|ctx| ctx.related_concepts(ncit_id, max_hops))
    {
        merged.extend(ids);
    }
    merged.sort();
    merged.dedup();
    merged
}

#[cfg(feature = "obo-graph")]
pub fn ncit_graph() -> Result<&'static GraphContext, OboGraphError> {
    NCIT_GRAPH
        .get_or_init(|| load_graph("ncit-mini"))
        .as_ref()
        .map_err(|err| OboGraphError::Load(err.to_string()))
}

#[cfg(feature = "obo-graph")]
pub fn mondo_graph() -> Result<&'static GraphContext, OboGraphError> {
    MONDO_GRAPH
        .get_or_init(|| load_graph("mondo-mini"))
        .as_ref()
        .map_err(|err| OboGraphError::Load(err.to_string()))
}

#[cfg(feature = "obo-graph")]
fn load_graph(id: &'static str) -> Result<GraphContext, OboGraphError> {
    let graph = load_ontology_graph(id).map_err(|err| OboGraphError::Load(err.to_string()))?;
    let version = graph
        .version
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    Ok(GraphContext {
        id,
        version,
        graph: CachedOntologyGraph::new(graph),
    })
}
