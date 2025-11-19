use super::{CachedOntologyGraph, load_ontology_graph};

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
