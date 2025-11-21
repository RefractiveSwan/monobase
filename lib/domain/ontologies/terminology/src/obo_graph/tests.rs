use std::path::Path;

use super::{
    CachedOntologyGraph, DEFAULT_CACHE_CAPACITY, OntologyGraph, Relation, load_ontology_graph,
    load_ontology_graph_from_path,
};

#[test]
fn parses_ncit_mini_fixture() {
    let graph = load_ontology_graph("ncit-mini").expect("parse ncit-mini");
    assert_eq!(graph.id, "ncit-mini");
    assert_eq!(graph.version.as_deref(), Some("ncit-mini-0.1.1"));
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);

    let pet_ct = graph
        .nodes
        .iter()
        .find(|node| node.iri == "NCIT:C19951")
        .expect("pet/ct node");
    assert_eq!(pet_ct.label, "Positron Emission Tomography");
    assert!(pet_ct.synonyms.contains(&"PET scan".to_string()));
}

#[test]
fn ancestors_descendants_and_synonyms_are_cached() {
    let cached = CachedOntologyGraph::new(load_ontology_graph("ncit-mini").unwrap());

    let ancestors = cached.ancestors("NCIT:C116746");
    assert_eq!(
        ancestors,
        vec!["NCIT:C19951".to_string(), "NCIT:C80219".to_string()]
    );

    let descendants = cached.descendants("NCIT:C19951");
    assert!(descendants.contains(&"NCIT:C116746".to_string()));

    let synonyms = cached.synonym_set("C19951").expect("synonyms present");
    assert!(synonyms.contains(&"positron emission tomography".to_string()));
    assert!(synonyms.contains(&"pet scan".to_string()));

    let synonyms_again = cached
        .synonym_set("NCIT:C116746")
        .expect("synonyms present");
    assert!(synonyms_again.contains(&"pet/ct".to_string()));
}

#[test]
fn related_concepts_are_bounded() {
    let cached = CachedOntologyGraph::new(load_ontology_graph("ncit-mini").unwrap());

    let one_hop = cached.related_concepts("NCIT:C19951", 1);
    assert!(one_hop.contains(&"NCIT:C116746".to_string()));
    assert!(!one_hop.contains(&"NCIT:C80219".to_string()));

    let two_hop = cached.related_concepts("NCIT:C19951", 2);
    assert!(two_hop.contains(&"NCIT:C80219".to_string()));
}

#[test]
fn mondo_synonyms_bridge_via_xref() {
    let cached = CachedOntologyGraph::new(load_ontology_graph("mondo-mini").unwrap());
    let synonyms = cached
        .synonym_set("NCIT:C19951")
        .expect("xref provides bridge");
    assert!(synonyms.contains(&"example disease".to_string()));
}

#[test]
fn pet_ct_synonyms_align_with_mapping_use_cases() {
    let cached = CachedOntologyGraph::new(load_ontology_graph("ncit-mini").unwrap());
    let pet_ct = cached
        .synonym_set("NCIT:C116746")
        .expect("pet/ct synonym set present");
    assert!(
        pet_ct.iter().any(|syn| syn.contains("pet/ct")),
        "mapping surfaces rely on pet/ct synonyms"
    );
    let ancestors = cached.related_concepts("NCIT:C116746", 1);
    assert!(
        ancestors.contains(&"NCIT:C19951".to_string()),
        "PET code should connect back to PET parent for ranking"
    );
}

#[test]
fn runtime_loader_reads_external_obo_files() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../data/clinical/ontologies/ncit-mini.example.obo");
    let graph =
        load_ontology_graph_from_path("ncit-mini-example", path).expect("parse runtime graph");
    assert_eq!(graph.id, "ncit-mini-example");
    assert!(
        graph.nodes.len() >= 3,
        "expected at least 3 nodes in runtime graph"
    );
    assert!(graph.nodes.iter().any(|node| node.iri == "NCIT:C19951"));
}

#[test]
fn cache_evicts_entries_for_large_graphs() {
    let graph = synthetic_graph(DEFAULT_CACHE_CAPACITY * 3);
    let cached = CachedOntologyGraph::new(graph);
    for idx in 0..(DEFAULT_CACHE_CAPACITY * 3) {
        let iri = format!("NCIT:C{idx:05}");
        cached.ancestors(&iri);
        cached.descendants(&iri);
    }
    let stats = cached.cache_stats();
    assert!(
        stats.ancestors <= DEFAULT_CACHE_CAPACITY,
        "ancestors cache should evict old entries"
    );
    assert!(
        stats.descendants <= DEFAULT_CACHE_CAPACITY,
        "descendants cache should evict old entries"
    );
}

fn synthetic_graph(size: usize) -> OntologyGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for idx in 0..size {
        let iri = format!("NCIT:C{idx:05}");
        nodes.push(super::types::Node {
            iri: iri.clone(),
            label: format!("Node {idx}"),
            synonyms: Vec::new(),
            xref_ncit_ids: Vec::new(),
        });
        if idx > 0 {
            edges.push(super::types::Edge {
                from: format!("NCIT:C{idx:05}"),
                to: format!("NCIT:C{prev:05}", prev = idx - 1),
                relation: Relation::IsA,
            });
        }
    }

    OntologyGraph {
        id: "synthetic".into(),
        version: Some("test".into()),
        nodes,
        edges,
    }
}
