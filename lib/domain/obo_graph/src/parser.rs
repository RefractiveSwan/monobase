use std::collections::{BTreeSet, HashMap};

use crate::error::OboError;
use crate::types::{Edge, Node, OntologyGraph, Relation};

#[derive(Default)]
struct NodeBuilder {
    id: Option<String>,
    label: Option<String>,
    synonyms: Vec<String>,
    xref_ncit_ids: Vec<String>,
}

impl NodeBuilder {
    fn finish(self) -> Result<Node, OboError> {
        let id = self
            .id
            .ok_or_else(|| OboError::Parse("missing id in [Term] stanza".to_string()))?;
        let label = self
            .label
            .ok_or_else(|| OboError::Parse(format!("missing label for term {}", id)))?;

        let mut synonyms: Vec<String> = self.synonyms.into_iter().collect();
        dedup_and_sort(&mut synonyms);

        let mut xref_ncit_ids: Vec<String> = self.xref_ncit_ids.into_iter().collect();
        dedup_and_sort(&mut xref_ncit_ids);

        Ok(Node {
            iri: id,
            label,
            synonyms,
            xref_ncit_ids,
        })
    }
}

fn dedup_and_sort(values: &mut Vec<String>) {
    let mut set = BTreeSet::new();
    for value in values.drain(..) {
        if !value.is_empty() {
            set.insert(value);
        }
    }
    values.extend(set.into_iter());
}

pub fn parse_obo(id: &str, source: &str) -> Result<OntologyGraph, OboError> {
    let mut graph_version: Option<String> = None;
    let mut nodes: HashMap<String, Node> = HashMap::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut builder: Option<NodeBuilder> = None;

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('!') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("data-version:") {
            graph_version = Some(rest.trim().to_string());
            continue;
        }

        if line == "[Term]" {
            if let Some(prev) = builder.take() {
                let node = prev.finish()?;
                nodes.insert(node.iri.clone(), node);
            }
            builder = Some(NodeBuilder::default());
            continue;
        }

        if line.starts_with('[') {
            // Ignore other stanzas (e.g., Typedef, Header).
            continue;
        }

        let Some(current) = builder.as_mut() else {
            // Ignore header lines outside of [Term] stanzas.
            continue;
        };

        if let Some(rest) = line.strip_prefix("id:") {
            current.id = Some(rest.trim().to_string());
            continue;
        }

        if let Some(rest) = line.strip_prefix("name:") {
            current.label = Some(rest.trim().to_string());
            continue;
        }

        if let Some(rest) = line.strip_prefix("synonym:") {
            if let Some(value) = parse_synonym(rest) {
                current.synonyms.push(value);
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("xref:") {
            if let Some(value) = parse_ncit_xref(rest) {
                current.xref_ncit_ids.push(value);
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("is_a:") {
            if let Some(target) = parse_edge_target(rest) {
                if let Some(source_id) = current.id.clone() {
                    edges.push(Edge {
                        from: source_id,
                        to: target,
                        relation: Relation::IsA,
                    });
                }
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("relationship:") {
            if let Some((relation, target)) = parse_relationship(rest) {
                if let Some(source_id) = current.id.clone() {
                    edges.push(Edge {
                        from: source_id,
                        to: target,
                        relation,
                    });
                }
            }
            continue;
        }
    }

    if let Some(last) = builder.take() {
        let node = last.finish()?;
        nodes.insert(node.iri.clone(), node);
    }

    let mut nodes: Vec<Node> = nodes.into_values().collect();
    nodes.sort_by(|a, b| a.iri.cmp(&b.iri));

    edges.sort_by(|a, b| match a.from.cmp(&b.from) {
        std::cmp::Ordering::Equal => match a.to.cmp(&b.to) {
            std::cmp::Ordering::Equal => a.relation.cmp(&b.relation),
            other => other,
        },
        other => other,
    });

    Ok(OntologyGraph {
        id: id.to_string(),
        version: graph_version,
        nodes,
        edges,
    })
}

fn parse_synonym(line: &str) -> Option<String> {
    let mut parts = line.split('"');
    parts.next()?;
    let value = parts.next()?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn parse_edge_target(line: &str) -> Option<String> {
    let mut parts = line.trim().split_whitespace();
    parts.next().map(|token| token.to_string())
}

fn parse_relationship(line: &str) -> Option<(Relation, String)> {
    let mut parts = line.trim().split_whitespace();
    let relation = parts.next()?;
    let target = parts.next()?;
    let relation = match relation {
        "part_of" => Relation::PartOf,
        "is_a" => Relation::IsA,
        _ => return None,
    };
    Some((relation, target.to_string()))
}

fn parse_ncit_xref(line: &str) -> Option<String> {
    let token = line.trim().split_whitespace().next()?;
    if let Some(stripped) = token.strip_prefix("NCIT:") {
        return Some(stripped.trim().to_string());
    }
    if let Some(stripped) = token.strip_prefix("NCIT_") {
        return Some(stripped.trim().to_string());
    }
    if token.starts_with('C') {
        return Some(token.trim().to_string());
    }
    None
}
