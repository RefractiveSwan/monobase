use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use super::types::{Edge, Node, OntologyGraph};

#[derive(Debug, Default)]
struct GraphCache {
    ancestors: Mutex<HashMap<String, Vec<String>>>,
    descendants: Mutex<HashMap<String, Vec<String>>>,
    synonyms: Mutex<HashMap<String, Vec<String>>>,
    related: Mutex<HashMap<(String, usize), Vec<String>>>,
}

#[derive(Debug, Clone)]
pub struct CachedOntologyGraph {
    graph: Arc<OntologyGraph>,
    cache: Arc<GraphCache>,
}

impl CachedOntologyGraph {
    pub fn new(graph: OntologyGraph) -> Self {
        Self {
            graph: Arc::new(graph),
            cache: Arc::new(GraphCache::default()),
        }
    }

    pub fn graph(&self) -> &OntologyGraph {
        &self.graph
    }

    pub fn ancestors(&self, iri: &str) -> Vec<String> {
        cached_lookup(&self.cache.ancestors, iri.to_string(), || {
            compute_ancestors(self.graph.as_ref(), iri)
        })
    }

    pub fn descendants(&self, iri: &str) -> Vec<String> {
        cached_lookup(&self.cache.descendants, iri.to_string(), || {
            compute_descendants(self.graph.as_ref(), iri)
        })
    }

    pub fn synonym_set(&self, ncit_id: &str) -> Option<Vec<String>> {
        let normalized = normalize_ncit_id(ncit_id);
        let values = cached_lookup(&self.cache.synonyms, normalized.clone(), || {
            let nodes = matching_nodes_for_ncit(self.graph.as_ref(), &normalized);
            let mut synonyms = BTreeSet::new();
            for node in nodes {
                synonyms.insert(node.label.to_lowercase());
                for syn in &node.synonyms {
                    synonyms.insert(sanitize_synonym(syn));
                }
            }
            synonyms.into_iter().collect()
        });

        if values.is_empty() {
            None
        } else {
            Some(values)
        }
    }

    pub fn related_concepts(&self, ncit_id: &str, max_hops: usize) -> Vec<String> {
        let normalized = normalize_ncit_id(ncit_id);
        cached_lookup(&self.cache.related, (normalized.clone(), max_hops), || {
            compute_related(self.graph.as_ref(), &normalized, max_hops)
        })
    }
}

fn cached_lookup<K, F>(cache: &Mutex<HashMap<K, Vec<String>>>, key: K, compute: F) -> Vec<String>
where
    K: Eq + Hash + Clone,
    F: FnOnce() -> Vec<String>,
{
    if let Some(existing) = cache.lock().unwrap().get(&key) {
        return existing.clone();
    }

    let computed = compute();
    cache.lock().unwrap().insert(key, computed.clone());
    computed
}

fn compute_ancestors(graph: &OntologyGraph, iri: &str) -> Vec<String> {
    let mut stack = vec![iri.to_string()];
    let mut visited: HashSet<String> = HashSet::new();
    let mut ancestors = BTreeSet::new();

    while let Some(current) = stack.pop() {
        for Edge {
            from,
            to,
            relation: _,
        } in graph.edges.iter()
        {
            if from == &current && visited.insert(to.clone()) {
                ancestors.insert(to.clone());
                stack.push(to.clone());
            }
        }
    }

    ancestors.into_iter().collect()
}

fn compute_descendants(graph: &OntologyGraph, iri: &str) -> Vec<String> {
    let mut stack = vec![iri.to_string()];
    let mut visited: HashSet<String> = HashSet::new();
    let mut descendants = BTreeSet::new();

    while let Some(current) = stack.pop() {
        for Edge {
            from,
            to,
            relation: _,
        } in graph.edges.iter()
        {
            if to == &current && visited.insert(from.clone()) {
                descendants.insert(from.clone());
                stack.push(from.clone());
            }
        }
    }

    descendants.into_iter().collect()
}

fn compute_related(graph: &OntologyGraph, ncit_id: &str, max_hops: usize) -> Vec<String> {
    if max_hops == 0 {
        return Vec::new();
    }

    let start_nodes = matching_nodes_for_ncit(graph, ncit_id);
    if start_nodes.is_empty() {
        return Vec::new();
    }
    let start_ids: Vec<String> = start_nodes.iter().map(|n| n.iri.clone()).collect();

    let mut queue: VecDeque<(String, usize)> =
        start_ids.clone().into_iter().map(|id| (id, 0)).collect();
    let mut visited: HashSet<String> = start_ids.iter().cloned().collect();
    let mut related = BTreeSet::new();

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_hops {
            continue;
        }

        for neighbor in neighbors(graph, &current) {
            if visited.insert(neighbor.clone()) {
                related.insert(neighbor.clone());
                queue.push_back((neighbor, depth + 1));
            }
        }
    }

    related.into_iter().collect()
}

fn neighbors(graph: &OntologyGraph, iri: &str) -> Vec<String> {
    let mut related = Vec::new();
    for Edge {
        from,
        to,
        relation: _,
    } in graph.edges.iter()
    {
        if from == iri {
            related.push(to.clone());
        } else if to == iri {
            related.push(from.clone());
        }
    }
    related
}

fn matching_nodes_for_ncit<'a>(graph: &'a OntologyGraph, ncit_id: &str) -> Vec<&'a Node> {
    graph
        .nodes
        .iter()
        .filter(|node| {
            normalize_ncit_id(&node.iri) == ncit_id
                || node
                    .xref_ncit_ids
                    .iter()
                    .any(|xref| normalize_ncit_id(xref) == ncit_id)
        })
        .collect()
}

fn normalize_ncit_id(raw: &str) -> String {
    let mut value = raw.trim();

    for prefix in &[
        "NCIT:",
        "ncit:",
        "NCIT_",
        "http://purl.obolibrary.org/obo/NCIT_",
    ] {
        if let Some(stripped) = value.strip_prefix(prefix) {
            value = stripped;
            break;
        }
    }

    value.to_uppercase()
}

fn sanitize_synonym(raw: &str) -> String {
    raw.trim().to_lowercase()
}
