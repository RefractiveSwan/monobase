# dfps_obo_graph

Embedded OBO graph loader plus lightweight reasoning helpers (ancestors,
descendants, synonym sets, related concepts). Keeps ontology parsing and graph
traversal firmly inside the domain layer while higher-level crates decide how
to source `.obo` files. See:

- `docs/system-design/clinical/ncit/concepts/obo-graph.md`
- `docs/system-design/clinical/fhir/concepts/terminology-layer.md`

## Data & versions

- `SUPPORTED_GRAPHS` documents the embedded fixtures (currently `ncit-mini` and
  `mondo-mini`). Each `.obo` lives under `data/clinical/ontologies/` and is kept
  intentionally small so tests stay fast.
- `load_ontology_graph(id)` parses the baked-in `.obo` text and carries through
  the `graph.version` metadata so downstream surfaces can display provenance.
- Future full-graph integrations (NCIt, MONDO, SNOMED, etc.) should add new IDs
  to `SUPPORTED_GRAPHS` or provide alternate loaders that feed
  `CachedOntologyGraph` with the same `OntologyGraph` struct.

## Cached reasoning

- `CachedOntologyGraph` exposes **pure** operations (no IO/env access): it
  caches ancestors/descendants/synonym sets/related concepts in memory so
  `dfps_terminology` and `dfps_mapping` can enrich candidates without touching
  files or HTTP.
- Lookups operate on NCIt IDs (with or without the `NCIT:` prefix) and leverage
  xrefs to bridge across ontologies (e.g., MONDO synonyms for NCIt PET/CT terms).

## Tests & mapping use cases

Unit tests assert that the embedded PET/CT examples expose the synonym/related
concept sets consumed by the terminology/mapping layers. If you extend NCIt or
add new modalities, add similar tests tying the ontology edges back to the
observability or scoring signals they unlock.
