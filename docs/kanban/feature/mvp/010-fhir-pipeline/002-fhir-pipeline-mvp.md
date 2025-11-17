# Kanban - feature/fhir-pipeline-mvp

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
- [ ] Validate serde field names (`resourceType`, `type`) & JSON shapes
- [ ] Check seed determinism across fake-data + ingestion tests

---

## DONE

### FP-01 – Core: minimal typed FHIR + staging models
- [x] Add `dfps_core::fhir` (typed, minimal R4) with:
  - [x] `Coding`, `CodeableConcept`, `Reference`
  - [x] `Patient`, `Encounter`, `ServiceRequest`
  - [x] `Bundle` + `BundleEntry` (JSON passthrough entry)
  - [x] `Bundle::iter_servicerequests()` iterator
- [x] Add `dfps_core::staging`:
  - [x] `StgServiceRequestFlat { sr_id, patient_id, encounter_id, status, intent, description }`
  - [x] `StgSrCodeExploded { sr_id, system, code, display }`
- [x] Module docs (`//!`) stating scope & invariants; align terms with diagrams in `docs/system-design/fhir/*`.

### FP-02 – Ingestion crate: transforms
- [x] New crate `dfps_ingestion`
  - [x] `sr_to_staging(sr)` -> `(StgServiceRequestFlat, Vec<StgSrCodeExploded>)`
  - [x] `sr_to_domain(sr)` -> `dfps_core::order::ServiceRequest` (normalize `status`/`intent`)
  - [x] `bundle_to_staging(bundle)` + `bundle_to_domain(bundle)`
  - [x] Helper to parse FHIR `Reference` `"Type/ID"` -> `ID`

### FP-03 – Fake raw FHIR generators
- [x] Extend `dfps_fake_data` with `raw_fhir` module:
  - [x] `fake_fhir_patient[_with_seed]`
  - [x] `fake_fhir_encounter_for[_with_seed]`
  - [x] `fake_fhir_servicerequest[_with_seed]` (compose 2–3 codings from CPT/SNOMED/LOINC)
  - [x] `fake_fhir_bundle_scenario[_with_seed]` (Patient + Encounter + ServiceRequest)
- [x] CLI: `generate_fhir_bundle` emitting NDJSON `Bundle`s (count + optional seed)

### FP-04 – Tests: e2e, properties, regression
- [x] Add `dfps_test_suite` dependency on `dfps_ingestion`
- [x] E2E: `fhir_ingest_flow.rs`
  - [x] bundle -> staging rows (1 flat per SR; N exploded rows = `coding.len()`)
  - [x] bundle -> domain aggregate matches IDs & normalized status/intent
- [x] Property test: for random seeds, `exploded.len() == sum(coding.len())`
- [x] Regression fixture: `lib/domain/fake_data/data/regression/fhir_bundle_sr.json` (1 SR with 2 codings)

### FP-05 – Workspace & CI wiring
- [x] Add `"lib/ingestion"` to root `[workspace].members`
- [x] Ensure CI runs `cargo fmt`, `clippy`, and `test` across all crates

### FP-06 – Docs alignment
- [x] Cross-link modules to:
  - [x] `docs/system-design/fhir/architecture/system-architecture.md`
  - [x] `docs/system-design/fhir/models/data-model-er.md`
  - [x] `docs/system-design/fhir/behavior/sequence-servicerequest.md`
- [x] Short “ingestion MVP” note in `docs/system-design/fhir/index.md`

### E2E-01 – Bundle -> mapped concepts facade
- [x] Add a public entrypoint (new crate `dfps_pipeline` or module in `dfps_mapping`) that composes `bundle_to_staging` and `map_staging_codes`:
  - [x] `bundle_to_mapped_sr(bundle) -> (Vec<StgServiceRequestFlat>, Vec<MappingResult>, Vec<DimNCITConcept>)`
  - [x] Return structured errors for ingestion/mapping failures.

### E2E-02 – End-to-end test: FHIR -> staging -> NCIt
- [x] In `dfps_test_suite`, load the regression bundle fixture, run the facade, and assert:
  - [x] Flat/exploded counts match the staging invariants.
  - [x] PET codes resolve to the expected NCIt IDs and mapping states.
  - [x] Mapping state distribution (AutoMapped / NeedsReview / NoMatch) is stable.

### E2E-03 – Pipeline CLI
- [x] Add a binary (e.g. `dfps_pipeline::bin::map_bundles`) that:
  - [x] Reads NDJSON FHIR Bundles.
  - [x] Emits NDJSON staging rows, mapping results, and NCIt dims.
  - [x] Supports deterministic seeds / sample data for demos.


---

## Acceptance Criteria
- `cargo test --all` passes.
- `dfps_fake_data::generate_fhir_bundle` prints valid FHIR `Bundle` NDJSON.
- `bundle_to_staging` yields exactly one flat row per SR and one exploded row per `code.coding[]`.
- Domain aggregate fields (IDs, status, intent, description) match the source FHIR semantics.

## Out of Scope (deferred)
- NCIt/UMLS mapping, vector search, warehouse loads.
- [x] Short “ingestion MVP” note in `docs/system-design/fhir/index.md`

### FP-07 – FHIR validation & error handling
- [x] Extend `IngestionError` coverage (missing `resourceType`, malformed `Reference`, bad `status/intents`).
- [x] Decide behavior (skip vs. accumulate vs. fail) and expose structured errors.
- [x] Unit/property tests around each error scenario.

### FP-08 – Messy FHIR regression fixtures
- [x] Add fixtures covering missing `subject`, extra codings, unknown systems, casing variants.
- [x] Regression tests asserting deterministic error/skip behavior.

### FP-09 – Quickstart docs & CLI usage
- [x] Update `docs/system-design/fhir/index.md` (or crate README) with copy/paste snippets:
  - [x] Example: load bundle JSON -> call `bundle_to_staging`.
  - [x] Document `generate_fhir_bundle` CLI usage for devs.
