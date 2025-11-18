# dfps_mapping – Mapping engine and NCIt integration

`dfps_mapping` provides a deterministic mapping engine that combines lexical and vector rankers with rule tweaks to produce `MappingResult`s backed by NCIt/UMLS data. Compliance/policy, terminology clients, and vector stores are injected via traits/config so the crate stays pure and testable.

## System-design links
- docs/system-design/clinical/ncit/architecture.md
- docs/system-design/clinical/ncit/models/data-model-er.md
- docs/system-design/clinical/fhir/overview.md
- docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-06--domain-mapping-engine--ncit-integration

## Modules & roles
- `MappingEngine` (`Mapper`) – orchestrates `CandidateRanker`s (lexical + vector) with a `RuleReranker` and fusion weights.
- `CandidateRanker` impls – `LexicalRanker`, `VectorRankerBackend` (trait-based over `VectorStore`/`EmbeddingProvider`), `VectorRankerMock`.
- `MappingConfig` – injects `Policy`, threshold defaults, and source versions (no env reads). Helpers accept an explicit policy or use `Policy::default_for_mode(Internal)`.
- Data loaders – embedded NCIt/UMLS snapshots (`load_ncit_concepts`, `load_umls_xrefs`) with version constants.
- Eval helpers – thin wrappers over `dfps_eval` (deprecated `run_eval` kept for compatibility).

## Usage notes
- Prefer `map_staging_codes_with_summary_and_policy`/`map_staging_codes_with_vector_and_policy` to pass an explicit `Policy`; env access has been removed.
- Mapping thresholds and source versions reuse `dfps_core` types via `MappingConfig`.
- Vector wiring is trait-based; backends (Qdrant/pgvector) remain outside this crate.
