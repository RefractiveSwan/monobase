# Kanban — feature/fhir-profiles-and-structuredefinition (018)

**Epic:** FHIR-018 – FHIR profiles & StructureDefinition  
**Branch:** `feature/domain/FHI-018-fhir-profiles-structuredefinition` | **Target version:** `v0.1.0`  
**Status:** INPROGRESS | **Introduced:** `v0.1.0` | **Last updated:** `v0.1.0`  
**Theme:** Spec-complete semantics - FHIR profiling & conformance metadata  
**Goal:** Represent core FHIR `StructureDefinition` metadata in code and wire it into ingestion validation, so requirements and profile constraints stay aligned.

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
- _Empty_

---

## DONE

### FHIR-PROF-01 – Profiles crate & data model

- [x] Introduce a new domain crate `lib/domain/fhir_profiles` (`dfps_fhir_profiles`):

  - [x] Types mirroring a subset of FHIR `StructureDefinition`:

    - `ProfileMeta { url, name, type, base_definition }`
    - `ElementDefinition { id, path, min, max, type_codes, must_support, binding? }`

  - [x] A thin `FhirProfile` struct that holds `ProfileMeta` + `Vec<ElementDefinition>`.

- [x] Provide loader functions:

  - [x] `load_profile(url: &str) -> Option<FhirProfile>` from embedded JSON (`include_str!`) for:

    - `Patient`, `Encounter`, `ServiceRequest` profiles used by DFPS.

### FHIR-PROF-02 – Linking requirements to profile constraints

- [x] Extend `RequirementRef` or add a mapping table:

  - [x] Map `R_Subject`, `R_Status`, `R_Trace` to specific `ElementDefinition.path` values.

- [x] Introduce helper:

  - [x] `profile_requirement_links(profile: &FhirProfile) -> Vec<(RequirementRef, ElementDefinition)>`.

- [x] Document this linkage in `docs/system-design/clinical/fhir/requirements/ingestion-requirements.md` under a “Profile mapping” section.

### FHIR-PROF-03 – Structural validation helpers

- [x] Under `dfps_ingestion::validation`, add functions that use `dfps_fhir_profiles`:

  - [x] `validate_sr_profile(sr, profile: &FhirProfile) -> Vec<ValidationIssue>` checking:

    - Cardinalities (`min`/`max`).
    - Required element presence for your subset (e.g., `subject`, `status`).

- [x] Integrate these checks into `validate_sr` / `validate_bundle` behind a feature flag:

  - [x] Ensure they coexist cleanly with existing hand-written validation logic.

### FHIR-PROF-04 – Fixtures & tests

- [x] Add example `StructureDefinition` JSON files for the relevant profiles under `data/fhir/profiles/`.

- [x] Tests in `dfps_fhir_profiles`:

  - [x] Parse embedded profiles.
  - [x] Validate that key paths (`ServiceRequest.subject`, `status`, etc.) are present and mapped to `RequirementRef`.

- [x] Tests in `dfps_test_suite`:

  - [x] A “profile-violating” bundle that passes basic validation but fails profile-based rules (min/max, unsupported elements).
  - [x] Confirm `validate_bundle` surfaces these as distinct `ValidationIssue`s.

### FHIR-PROF-05 – Docs

- [x] Extend `docs/system-design/clinical/fhir/overview.md` with:

  - [x] A “Profiles & conformance” markdown section summarizing:

    - Which profiles DFPS understands.
    - How they are enforced.

- [x] Add a short runbook `docs/runbook/fhir-profiles-quickstart.md` for:

  - [x] Updating embedded profiles.
  - [x] Regenerating any derived metadata.

---

## Acceptance Criteria

- Core FHIR profiles for Patient/Encounter/ServiceRequest are represented in code and can be loaded.
- Validation APIs can optionally enforce a subset of profile constraints, with issues mapped back to requirements.
- Existing regression fixtures continue to pass under the intended profiles.

## Out of Scope

- Modeling or enforcing the entire FHIR specification.
- Implementing a generic profile engine for arbitrary FHIR resources beyond DFPS’s narrow slice.
