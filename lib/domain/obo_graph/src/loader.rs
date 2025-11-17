use crate::error::OboError;
use crate::parser::parse_obo;
use crate::types::OntologyGraph;

pub const SUPPORTED_GRAPHS: &[&str] = &["ncit-mini", "mondo-mini"];

pub fn load_ontology_graph(id: &str) -> Result<OntologyGraph, OboError> {
    match id {
        "ncit-mini" => parse_obo(
            "ncit-mini",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../data/obo/ncit-mini.obo"
            )),
        ),
        "mondo-mini" => parse_obo(
            "mondo-mini",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../data/obo/mondo-mini.obo"
            )),
        ),
        other => Err(OboError::UnsupportedGraph(other.to_string())),
    }
}
