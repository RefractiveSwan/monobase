# OBO Graph for NCIt + Companion Ontologies

Lightweight OBO graph ingestion and reasoning utilities that expose NCIt (and a small MONDO slice) as a deterministic graph for terminology enrichment and mapping.

## Responsibilities
- Parse curated OBO slices into `OntologyGraph { nodes, edges }` with stable ordering and explicit relations (`is_a`, `part_of`).
- Surface cached graph queries: `ancestors`, `descendants`, `synonym_set`, and bounded `related_concepts`.
- Keep graph IDs/version metadata visible to downstream caches (`obo-graph` feature flag gated).

## Data sources
- Embedded fixtures under `data/clinical/ontologies/` (mini NCIt + MONDO slices for PET/CT and xref coverage).
- Runtime loader: `load_ontology_graph_from_path(id, path)` parses arbitrary `.obo` files shipped next to deployments (used for research drops or NCIt previews).
- Loader registry: `load_ontology_graph("ncit-mini" | "mondo-mini")`; `SUPPORTED_GRAPHS` enumerates the embedded set.
- Normalizes NCIt IDs across forms (`NCIT:C123`, `NCIT_C123`, `C123`); captures `data-version` headers for downstream cache/version plumbing.

## APIs & contracts
- Module: `lib/domain/ontologies/terminology::obo_graph`
  - Types: `OntologyGraph`, `Node { iri, label, synonyms, xref_ncit_ids }`, `Edge { from, to, relation }`, `Relation`.
  - Loaders: `load_ontology_graph(id)` (embedded) and `load_ontology_graph_from_path(id, path)` (runtime .obo) → `Result<OntologyGraph, OboError>`.
  - Reasoner: `CachedOntologyGraph::ancestors`, `descendants`, `synonym_set`, `related_concepts(max_hops)`.
- `synonym_set(ncit_id)` includes canonical label + normalized synonyms across nodes whose `iri` or `xref_ncit_ids` match.
- `related_concepts` uses an undirected, hop-bounded walk; results are deterministic and cached.

## Feature flag & caching
- Graph reasoning is behind the `obo-graph` feature flag for consumers (terminology/mapping) to preserve deterministic behavior when disabled.
- `CachedOntologyGraph` memoizes query results per graph ID/version with bounded LRU caches (512 entries per query type) so long-running services don't accumulate unbounded state.
- Caches are keyed by normalized NCIt IDs and hop count for related queries; eviction happens transparently as new IDs are requested.

## Fixtures & tests
- Fixtures: `data/obo/ncit-mini.obo`, `data/obo/mondo-mini.obo` (PET/CT lineage + MONDO cross-xref).
- Tests (`dfps_terminology::obo_graph`):
  - Parse fixtures, verify ancestor/descendant relationships (PET-CT → PET, CT).
  - Synonym expansion (label + PET/CT variants) and hop-bounded related concept queries.
  - Runtime loader integration (mini `.obo` fixtures) + cache eviction behavior via synthetic large graphs.
  - Xref-driven synonym discovery via MONDO slice.
- Version surfacing: graph versions are exposed via `GraphContext::version` and `dfps_terminology::obo::graph_versions()` for cache invalidation and metrics tags.

## Cross-links
- Terminology layer with graph context: `../../fhir/concepts/terminology-layer.md`
- Mapping vector layer (synonym expansion interplay): `vector-layer.md`
- Kanban: `docs/kanban/feature/mvp/019-obo-import-and-reasoning.md`
