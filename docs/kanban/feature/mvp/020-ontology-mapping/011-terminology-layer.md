# Kanban - feature/terminology-layer (011)

**Branch:** `feature/domain/terminology-layer`  
**Goal:** Introduce an explicit terminology layer that:
- differentiates **licensed** vs **unlicensed/open** vocabularies, and  
- integrates **OBO Foundry** sources (e.g., NCIt OBO, MONDO) alongside FHIR CodeSystems, introducing an explicit FHIR terminology layer (CodeSystem / ValueSet metadata and registries) between staging codes and the NCIt mapping engine.

### Columns
* **TODO** - Not started yet
* **INPROGRESS** - In progress
* **REVIEW** - Needs code review / refactor / docs polish
* **DONE** - Completed

---

## TODO
- _Empty_

---

## INPROGRESS
- _Empty_

---

## REVIEW
- [ ] Confirm license tiers and source kinds are modeled correctly for all seeded systems.
- [ ] Ensure mapping behavior is stable and existing golden tests (`dfps_mapping`, `dfps_test_suite`) remain valid.
- [ ] Sanity check docs so they match the actual licensed/unlicensed split and OBO integration points.

---

## DONE

### TERM-01 - Terminology crate scaffold
- [x] Create `lib/domain/terminology` crate (e.g., `dfps_terminology`).
- [x] Wire into `Cargo.toml` workspace members.
- [x] Initial modules:
  - [x] `codesystem` - FHIR / code system metadata.
  - [x] `obo` - OBO Foundry ontology metadata (NCIt OBO, MONDO, etc.).
  - [x] `registry` - unified registries and lookup APIs.

### TERM-02 - CodeSystem metadata with license tier
- [x] Implement:
  - [x] `LicenseTier` enum (e.g., `Licensed`, `Open`, `InternalOnly`).
  - [x] `SourceKind` enum (e.g., `FHIR`, `UMLS`, `OBOFoundry`, `Local`).
  - [x] `CodeSystemMeta { url, name, version, description, license_tier, source_kind }`.
- [x] Seed registry entries for core systems, with **license-aware** classification:
  - [x] `http://www.ama-assn.org/go/cpt` -> `Licensed`.
  - [x] `http://snomed.info/sct` -> `Licensed`.
  - [x] `http://loinc.org` -> appropriate tier (e.g., `Licensed` or `Open`, per policy).
  - [x] NCIt OBO IRI(s) -> `Open`, `source_kind = OBOFoundry`.
- [x] Add helper APIs:
  - [x] `lookup_codesystem(url: &str) -> Option<CodeSystemMeta>`
  - [x] `is_licensed(url: &str) -> bool`
  - [x] `is_open(url: &str) -> bool`.

### TERM-03 - OBO Foundry integration
- [x] Add `obo` module with:
  - [x] `OntologyMeta { id, iri, preferred_prefix, version, description }`.
  - [x] seed entries for NCIt OBO and at least one additional OBO Foundry ontology (e.g., MONDO) referenced in docs.
- [x] Provide helper APIs:
  - [x] `lookup_ontology(prefix_or_iri: &str) -> Option<OntologyMeta>`.
  - [x] mapping between NCIt IDs in `dfps_core::mapping::DimNCITConcept` and OBO IRIs when available.
- [x] Ensure OBO ontologies are recorded as **unlicensed/open** in metadata and never treated as “licensed-protected” in downstream flows.

### TERM-04 - Staging ↔ terminology bridge (license-aware)
- [x] Introduce an adapter type:
  - [x] `EnrichedCode { staging: StgSrCodeExploded, codesystem: Option<CodeSystemMeta>, license_tier: Option<LicenseTier>, source_kind: Option<SourceKind> }`.
- [x] Decide and document URL normalization rules (lowercasing, trailing slashes, canonical SNOMED/LOINC URLs).
- [x] Provide classification:
  - [x] `CodeKind` enum that distinguishes:
    - [x] `KnownLicensedSystem`
    - [x] `KnownOpenSystem`
    - [x] `OBOBacked` (where an OBO mapping/ontology is known)
    - [x] `UnknownSystem`
    - [x] `MissingSystemOrCode`.

### TERM-05 - Mapping integration (reason codes, policy hooks)
- [x] Integrate terminology checks into `dfps_mapping::map_staging_codes`:
  - [x] For `UnknownSystem` ?+' `MappingResult.state = NoMatch`, `reason = "unknown_code_system"`.
  - [x] For `MissingSystemOrCode` ?+' `reason = "missing_system_or_code"` (existing behavior).
- [x] Add **license-aware** hooks (no hard policy yet, but wiring in the data):
  - [x] Ensure `MappingResult` can optionally surface `license_tier` / `source_kind` via `reason` or a reserved metadata field if needed in the future.
  - [x] Keep behavior deterministic and non-breaking for existing tests.
- [x] Optional: add helper to aggregate counts by `CodeKind` and `LicenseTier` for observability.

### TERM-06 - Tests
- [x] Unit tests for registries:
  - [x] Known URLs (CPT/SNOMED/LOINC/NCIt OBO) resolve with correct `LicenseTier` and `SourceKind`.
  - [x] Bogus/non-canonical URLs resolve as `UnknownSystem`.
- [x] Unit tests for `EnrichedCode`:
  - [x] correct classification into `CodeKind` variants.
- [x] Integration tests with `dfps_mapping`:
  - [x] Known systems behave as before for mapping outcomes.
  - [x] Unknown systems produce `reason = "unknown_code_system"`.
  - [x] Ensure OBO-backed concepts are still treated as `Open` and do not flip any licensed flags.

### TERM-07 - Docs (licensed vs unlicensed + OBO)
- [x] Add `docs/system-design/clinical/fhir/concepts/terminology-layer.md` describing:
  - [x] the **licensed vs unlicensed/open** terminology split,
  - [x] the role of OBO Foundry ontologies,
  - [x] where this sits between staging and NCIt mapping.
- [x] Update:
  - [x] `docs/system-design/clinical/fhir/overview.md` to reference the terminology layer and license split.
  - [x] `docs/system-design/clinical/ncit/architecture.md` to explicitly call out:
    - FHIR CodeSystems,
    - UMLS/NCIm,
    - OBOFoundry (NCIt OBO, MONDO) as distinct but connected sources.


---

## Acceptance Criteria
- `dfps_terminology` exists and exposes:
  - license-aware `CodeSystemMeta` lookups,
  - OBO Foundry `OntologyMeta` lookups,
  - a classification of staging codes into `CodeKind` with license/source context.
- Mapping engine can distinguish:
  - missing identifiers,
  - unknown systems,
  - known licensed systems,
  - open/OBO-backed systems.
- Docs clearly reflect:
  - how licensed vs unlicensed vocabularies are handled, and
  - where OBO Foundry ontologies plug into the FHIR -> NCIt pipeline.

## Out of Scope
- Actual license enforcement or distribution logic (legal/compliance layer).
- Full OBO import/parse pipelines or ontology reasoners.
