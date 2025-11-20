use std::path::Path;

use super::error::OboError;
use super::parser::parse_obo;
use super::types::OntologyGraph;

pub const SUPPORTED_GRAPHS: &[&str] = &["ncit-mini", "mondo-mini"];

pub fn load_ontology_graph(id: &str) -> Result<OntologyGraph, OboError> {
    match id {
        "ncit-mini" => parse_obo(
            "ncit-mini",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../../data/clinical/ontologies/ncit-mini.obo"
            )),
        ),
        "mondo-mini" => parse_obo(
            "mondo-mini",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../../data/clinical/ontologies/mondo-mini.obo"
            )),
        ),
        other => Err(OboError::UnsupportedGraph(other.to_string())),
    }
}

/// Load an ontology graph from a runtime `.obo` file path.
pub fn load_ontology_graph_from_path(
    id: impl Into<String>,
    path: impl AsRef<Path>,
) -> Result<OntologyGraph, OboError> {
    let id = id.into();
    let path_ref = path.as_ref();
    let contents = std::fs::read_to_string(path_ref).map_err(|source| OboError::Io {
        path: path_ref.display().to_string(),
        source,
    })?;
    parse_obo(&id, &contents)
}
