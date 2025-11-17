# Kanban - feature/mapping-ncit-skeleton

### Columns
* **TODO** – Not started yet
* **INPROGRESS** – In progress
* **REVIEW** – Needs code review / refactor / docs polish
* **DONE** – Completed

---

## TODO


---

## INPROGRESS
- _Empty_

---

## REVIEW
- [ ] Cross-check mock tables cover codes used by `fake_data::raw_fhir`
- [ ] Verify mapping states align to `docs/system-design/ncit/behavior/state-servicerequest.md`

---

## DONE

### MAP-01 – Core: mapping & concept types
- [x] Add `dfps_core::mapping` module:
  - [x] `CodeElement { id, system, code, display }`
  - [x] `MappingCandidate { target_system, target_code, cui, score }`
  - [x] `MappingResult { code_element_id, cui, ncit_id, score, strategy, license_tier?, source_kind? }`
  - [x] `NCItConcept { ncit_id, preferred_name, synonyms[] }`
  - [x] `DimNCITConcept { ncit_id, preferred_name, semantic_group }`
- [x] From-staging conversion:
  - [x] `impl From<StgSrCodeExploded> for CodeElement`

### MAP-02 – Mapping crate skeleton
- [x] New crate `dfps_mapping`
  - [x] Traits
    - [x] `trait Mapper { fn map(&self, code: &CodeElement) -> MappingResult; }`
    - [x] `trait CandidateRanker { fn rank(&self, code: &CodeElement) -> Vec<MappingCandidate>; }`
  - [x] Implementations (stubs)
    - [x] `LexicalRanker` (string contains / Jaccard-ish heuristic)
    - [x] `VectorRankerMock` (deterministic “embedding” via hashing; no external deps)
    - [x] `RuleReranker` (prefers NCIt-linked CUIs, bumps SNOMED/CPT priority)
  - [x] `MappingEngine` composing rankers + rules -> `MappingResult`

### MAP-03 – Minimal NCIt/UMLS mock data
- [x] Tiny embedded tables for tests:
  - [x] `ncit_concepts.json` (e.g., `NCIT:C19951` “Positron Emission Tomography”)
  - [x] `umls_xrefs.json` linking SNOMED/CPT/LOINC -> CUIs -> NCIt
- [x] Loaders + lookups with stable version identifiers (for provenance)

### MAP-04 – Pipelines & integration points
- [x] Function: `map_staging_codes(iter<StgSrCodeExploded>) -> (Vec<MappingResult>, Vec<DimNCITConcept>)`
- [x] Optional CLI: `map_codes` (reads staging NDJSON, writes mappings NDJSON)

### MAP-05 – Tests
- [x] Golden tests (deterministic):
  - [x] Known input code “78815” -> expected NCIt concept
  - [x] SNOMED PET concept -> expected NCIt
- [x] Property tests:
  - [x] `score` monotonicity under synonym augmentation
  - [x] Top-1 candidate always ≥ all others
- [x] Regression fixtures under `lib/domain/fake_data/data/regression/mapping_*`

### MAP-07 – Provenance & thresholds
- [x] Add `strategy`, `source_version`, `thresholds` notes to `MappingResult`
- [x] Configurable cutoffs: `AUTO_MAP >= 0.95`, `NEEDS_REVIEW >= 0.60`

### MAP-09 – Mapping state machine
- [x] Introduce `MappingState` enum (`AutoMapped`, `NeedsReview`, `NoMatch`).
- [x] Wire thresholds from MAP-07 into `MappingResult` (state + reasoning).
- [x] Tests ensuring threshold tweaks move codes between states deterministically.

### MAP-06 – Docs & diagrams
- [x] Update/expand:
  - [x] `docs/system-design/ncit/architecture.md`
  - [x] `docs/system-design/ncit/behavior/sequence-servicerequest.md`
  - [x] `docs/system-design/ncit/models/class-model.md`
- [x] Document thresholds & states (AutoMapped / NeedsReview / NoMatch)

### MAP-08 – Workspace & CI
- [x] Add `"lib/mapping"` to workspace members
- [x] Ensure CI runs tests for mapping crate

### MAP-10 – Unknown code handling
- [x] Define behavior for codes not covered by mock tables or low-scoring rankers.
- [x] Emit `NoMatch` + provenance, ensure CLI/pipeline surfaces it cleanly.
- [x] Regression tests with synthetic “unknown” codes.

### MAP-11 – Mapping explainability helpers
- [x] Add helper APIs (and docs) to inspect top-N candidates + contributing rankers.
- [x] CLI option or debug output summarizing why a code mapped to a given NCIt.
- [x] Short doc snippet walking through the CPT 78815 example.

---

## Acceptance Criteria
- Mapping crate compiles; traits + engine wired.
- Golden tests pass with deterministic outputs for the seeded sample codes.
- End-to-end: `StgSrCodeExploded` -> `CodeElement` -> `MappingResult (NCIT:Cxxxx)` with provenance.
- Threshold behavior produces expected states (AutoMapped / NeedsReview / NoMatch).

## Out of Scope (deferred)
- Real vector DB, external UMLS/NCIt APIs, warehouse loads.
