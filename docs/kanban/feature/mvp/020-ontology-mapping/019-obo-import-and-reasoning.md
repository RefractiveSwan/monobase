# Kanban - feature/obo-import-and-reasoning (019)

**Theme:** Spec-complete semantics - full(er) OBO import & lightweight reasoning  
**Branch:** `feature/domain/TERM-019-obo-import-and-reasoning`  
**Branch target version:** Unreleased  
> Status: **REVIEW**  
> Introduced in: `v0.1.0`  
> Last updated in: `v0.1.0`  
**Goal:** Import NCIt OBO (and at least one companion OBO ontology) into a graph representation and expose simple reasoning utilities (transitive closure, synonym expansion) for mapping and analytics.

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / refactor / docs polish  
* **DONE** – Completed  

---

## TODO
- _Empty_

---

## INPROGRESS
- _Empty_

---

## REVIEW

### TERM-01 – OBO graph crate

- [x] Add a new crate `lib/domain/ontologies/terminology (obo_graph module)` (`dfps_terminology::obo_graph`) with a focused API for NCIt + one companion ontology (e.g., MONDO) and deterministic parsing.

  - [x] Types (serde-friendly, deterministic ordering; error type captures missing labels/xrefs):

    - `OntologyGraph { id, nodes: Vec<Node>, edges: Vec<Edge> }`
    - `Node { iri, label, synonyms: Vec<String>, xref_ncit_ids: Vec<String> }`
    - `Edge { from, to, relation }` with focus on `is_a`/`part_of` and a small enum of supported relations.

  - [x] Parsers for NCIt OBO and MONDO:

    - Either via an OBO/OWL parsing library or a minimal line-based parser for a curated slice with invariant checks (required fields, deduped synonyms, stable ID ordering).

- [x] Provide helpers:

  - [x] `load_ontology_graph(id: &str) -> Result<OntologyGraph, OboError>` using embedded or on-disk `.obo` / `.owl`, plus a light registry of supported IDs and data versions.

#### Cross-Cohesion

- **Engineering Targets:** A1, C
- **Crates & Paths:**
  - `lib/domain/ontologies/terminology (obo_graph module)` (`dfps_terminology::obo_graph`)
  - `data/clinical/ontologies` (fixtures referenced by dfps_terminology::obo_graph)
- **Shared Metrics & Signals:**
  - `graph_communities_count`, `graph_modularity`, `graph_conductance_mean`
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/concepts/obo-graph.md`
  - `docs/kanban/feature/mvp/019-obo-import-and-reasoning.md`
- **Experiments / CI Hooks:**
  - `dfps_terminology::obo_graph` parser/round-trip unit tests
  - Fixture validation task for embedded `.obo` slices
- **Interfaces & Contracts:**
  - API: `load_ontology_graph`, `OntologyGraph`, `Node`, `Edge`
  - Env/feature: `obo-graph` feature flag for consumers

### TERM-02 – Reasoning utilities

- [x] Implement basic utilities on `OntologyGraph` with deterministic traversal order and bounded fanout:

  - [x] `ancestors(node_iri)`, `descendants(node_iri)` via transitive closure with cycle guards.
  - [x] `synonym_set(ncit_id)` giving canonical label + normalized synonyms from OBO (trim + lowercase rules).
  - [x] `related_concepts(ncit_id)` limited to a few hops in the graph with relation filters.

- [x] Add a small caching layer so repeated queries are fast (hash-based memoization scoped to graph ID/version).

#### Cross-Cohesion

- **Engineering Targets:** A1, B
- **Crates & Paths:**
  - `lib/domain/ontologies/terminology (obo_graph module)` (`dfps_terminology::obo_graph`)
  - `lib/domain/ontologies/terminology` (`dfps_terminology`)
- **Shared Metrics & Signals:**
  - `mapping_precision`, `mapping_recall`, `mapping_f1`
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/concepts/obo-graph.md`
  - `docs/system-design/clinical/fhir/concepts/terminology-layer.md`
- **Experiments / CI Hooks:**
  - Memoization benchmarks in `dfps_terminology::obo_graph`
  - Reasoning utilities unit tests (ancestor/descendant/synonym)
- **Interfaces & Contracts:**
  - APIs: `ancestors`, `descendants`, `synonym_set`, `related_concepts`
  - Cache behavior: version-aware memoization keyed by graph ID

### TERM-03 – Integration with terminology & mapping

- [x] Extend `dfps_terminology::obo` to bridge `DimNCITConcept.ncit_id` → `OntologyGraph` node IRIs and expose version metadata for downstream caches.

- [x] In `dfps_mapping`:

  - [x] Optionally call `synonym_set` / `ancestors` when building lexical features:

    - Expand candidate synonyms used by `LexicalRanker`.
    - Optionally adjust mapping scores based on ontology distance.

- [x] Ensure behavior is gated behind a feature flag (`obo-graph`) and remains deterministic when disabled (same ordering/scores when off).

#### Cross-Cohesion

- **Engineering Targets:** A1, B, D
- **Crates & Paths:**
  - `lib/domain/ontologies/terminology` (`dfps_terminology`)
  - `lib/domain/mapping` (`dfps_mapping`)
  - `lib/app/frontend/cli` (`dfps_cli`)
- **Shared Metrics & Signals:**
  - `mapping_precision`, `mapping_recall`, `mapping_f1`
  - `auto_mapped`, `needs_review`, `no_match`
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/fhir/concepts/terminology-layer.md`
  - `docs/kanban/feature/mvp/013-mapping-vector-backend.md`
  - `docs/kanban/feature/mvp/019-obo-import-and-reasoning.md`
- **Experiments / CI Hooks:**
  - `dfps_cli map_codes` + `dfps_cli eval_mapping` smoke tests with/without `obo-graph`
  - `dfps_test_suite/tests/integration/vector_mapping.rs` (coverage for synonym expansion)
- **Interfaces & Contracts:**
  - Feature flag: `obo-graph`
  - APIs: terminology bridge helpers; `LexicalRanker` synonym expansion hooks
  - Env: `DFPS_EVAL_DATA_ROOT` for eval datasets when testing effects

### TERM-04 – Tests & fixtures

- [x] Add small clipped NCIt OBO fixtures under `data/clinical/ontologies/ncit-mini.obo` with deterministic ordering and comments on provenance:

  - [x] Include nodes for PET/CT and immediate neighbors; include at least one MONDO slice for cross-ontology validation.

- [x] Tests in `dfps_terminology::obo_graph`:

  - [x] Parse the mini graph.
  - [x] Verify ancestor/descendant relationships for known NCIt IDs (e.g., PET/CT lineage).
  - [x] Round-trip serialization retains node/synonym ordering.

- [x] Mapping tests in `dfps_test_suite`:

  - [x] Confirm synonym expansion from OBO does not reduce scores (monotonicity) and improves coverage for close variants of PET/CT codes.

#### Cross-Cohesion

- **Engineering Targets:** A1, B, D
- **Crates & Paths:**
  - `data/clinical/ontologies` (fixtures)
  - `lib/domain/ontologies/terminology (obo_graph module)` (`dfps_terminology::obo_graph`)
  - `lib/platform/test_suite` (`dfps_test_suite`)
- **Shared Metrics & Signals:**
  - `mapping_precision`, `mapping_recall`, `mapping_f1`
  - `graph_leiden_bad_communities`, `graph_communities_count`
- **Docs & Kanbans Touched:**
  - `docs/runbook/terminology-apis-quickstart.md`
  - `docs/kanban/feature/mvp/019-obo-import-and-reasoning.md`
- **Experiments / CI Hooks:**
  - Integration tests: `lib/platform/test_suite/tests/integration/vector_mapping.rs`
  - Fixture validation job for `data/clinical/ontologies/ncit-mini.obo`
- **Interfaces & Contracts:**
  - CLI/data: fixtures consumed by `dfps_cli map_codes`/`eval_mapping` smoke tests
  - API: `load_ontology_graph` fixture loading contract

### TERM-05 – Docs

- [x] Add `docs/system-design/clinical/ncit/concepts/obo-graph.md` describing:

  - [x] How NCIt OBO / MONDO graphs are loaded (source, versioning, feature flags).
  - [x] What reasoning capabilities are supported and how they influence mapping and eval.

- [x] Update `docs/system-design/clinical/fhir/concepts/terminology-layer.md` to reflect that TERM-backed concepts now have graph context, not just static metadata; include notes on cache invalidation/versioning.

#### Cross-Cohesion

- **Engineering Targets:** A1, C, D
- **Crates & Paths:**
  - `lib/domain/ontologies/terminology (obo_graph module)` (`dfps_terminology::obo_graph`)
  - `lib/domain/ontologies/terminology` (`dfps_terminology`)
  - `lib/domain/mapping` (`dfps_mapping`)
- **Shared Metrics & Signals:**
  - `mapping_precision`, `mapping_recall`
  - `graph_modularity`, `graph_conductance_mean`
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/concepts/obo-graph.md`
  - `docs/system-design/clinical/fhir/concepts/terminology-layer.md`
  - `docs/kanban/feature/mvp/019-obo-import-and-reasoning.md`
- **Experiments / CI Hooks:**
  - `cargo make docs` (mdBook build) including new OBO graph doc
  - Doc examples synced with `dfps_cli`/`dfps_terminology::obo_graph` APIs
- **Interfaces & Contracts:**
  - Documentation links for `obo-graph` feature flag and `OntologyGraph` API
  - References to `dfps_cli map_codes`/`eval_mapping` usage with OBO context

---

## DONE
- _Empty_

---

## Acceptance Criteria

- NCIt OBO (and at least one other OBO ontology) can be imported into a graph representation.
- Mapping code can optionally use TERM-derived synonyms/relations to improve lexical matching.
- Tests prove that OBO integration is deterministic and does not break existing golden mappings.

## Out of Scope

- Full DL reasoning (e.g., complete OWL2 DL reasoners).
- Supporting arbitrary OBO ontologies beyond the curated set used by DFPS.
