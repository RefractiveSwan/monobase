# Kanban - feature/fhir-external-conformance (015)

> Epic: 015 – External FHIR conformance  
> Branch: `feature/EVAL-015-fhir-external-conformance`  
> Branch target version: `Unreleased`  
> Status: DOING  
> Introduced in: `v0.1.0`  
> Last updated in: `v0.1.0`

**Themes:** External infra & heavy services; Spec-complete semantics (FHIR)  
**Goal:** Integrate an external FHIR validator/server into the ingestion flow, producing rich `ValidationIssue`s backed by FHIR `OperationOutcome` results.

### Columns
* **TODO** – Not started yet  
* **DOING** – In progress  
* **REVIEW** – Needs code review / refactor / docs polish  
* **DONE** – Completed  

---

## TODO

---

## DOING
- _Empty_

---

## REVIEW

### FHIR-CONF-01 – External validation model

- [x] Add a small `external` module under `dfps_ingestion::validation` or a new crate `lib/domain/fhir_validation` (`dfps_fhir_validation`):

  - [x] Represent a subset of FHIR `OperationOutcome`:

    - `OperationOutcomeIssue { severity, code, diagnostics, expression[] }`
    - `OperationOutcome { issues: Vec<OperationOutcomeIssue> }`

  - [x] Introduce `ExternalValidationReport` with:

    - `operation_outcome: Option<OperationOutcome>`
    - Mappings to `ValidationIssue` (reusing `RequirementRef` where possible).

### FHIR-CONF-02 – HTTP client & configuration

- [x] Expose an `ExternalValidator` port (no env/HTTP inside `dfps_ingestion`), plus a CLI adapter using `reqwest` + env config to hit `$validate`.

- [x] Add `.env.domain.fhir_validation.dev/example` documenting these keys.

### FHIR-CONF-03 – Blending internal & external validation

- [x] Extend `ValidationMode` with new variants:

  - `ExternalPreferred`, `ExternalStrict`.

- [x] Update `validate_bundle` to:

  - [x] Optionally call an injected `ExternalValidator` and merge results:

    - External `OperationOutcome` issues mapped into `ValidationIssue` with a new `RequirementRef` variant (e.g., `RExternal`), or tagged via a `source` field.
    - Ensure that IDs/requirements from `ingestion-requirements.md` remain stable.

  - [x] In `ExternalStrict` mode, treat any `OperationOutcomeIssue` with severity `error` or `fatal` as blocking; treat a missing validator as an `RExternal` issue.

- [x] Provide helpers:

- `validate_bundle_with_external(bundle, mode, ExternalValidationContext) -> ValidationReport`.

### FHIR-CONF-04 – CLI & developer ergonomics

- [x] Add a new CLI in `dfps_cli`:

  - `validate-fhir`:

    - [x] Reads Bundle JSON/NDJSON from stdin or file.
    - [x] Calls `validate_bundle_with_external_profile` with an injected validator context.
    - [x] Emits NDJSON `ValidationIssue` rows plus a summary line (counts by severity and source).

- [x] Update `docs/system-design/clinical/fhir/index.md` with:

  - [x] A “Validation (external)” subsection.
  - [x] Example `dfps_cli validate-fhir` commands.

### FHIR-CONF-05 – Tests & mocks

- [x] Add a test-only mock FHIR validator server in `dfps_test_suite`:

  - [x] Provides `/fhir/$validate` that returns canned `OperationOutcome` fixtures for:

    - Missing subject, invalid status, etc.
    - Valid bundles.

- [x] Write integration tests:

  - [x] Validate that external issues are merged with internal `validate_bundle` results.
  - [x] Ensure that `ExternalStrict` mode blocks ingestion for failing bundles but allows pass-through when `ExternalPreferred` is used.

---

## DONE
- _Empty_

---

## Acceptance Criteria

- Ingesting a Bundle can optionally consult an external FHIR validator and surface `OperationOutcome` issues through `ValidationIssue`.
- Internal validation remains usable without any external services (default).
- CLI + docs make it easy to toggle external validation on/off and understand how failures are reported.

## Out of Scope

- Hosting a FHIR server/validator within this repo.
- Modeling the full FHIR `OperationOutcome` specification (only the subset needed for ingestion diagnostics).
