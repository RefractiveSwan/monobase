# Crate: lib/domain/ingestion - `refractive_swan_ingestion`

**Path:** `code/lib/domain/ingestion`  
**Depends on:** `refractive_swan_core`, `serde(_json)`.

## Responsibilities
- Normalize **FHIR -> staging -> domain** (`ServiceRequest`) with clear, typed errors.
- Provide **validation** utilities aligned with FHIR ingestion requirements.
- Keep behavior predictable; **strict** vs **lenient** modes available.

## Public API (re‑exports in `lib.rs`)
- `reference::{reference_id, reference_id_from_str}` - parse `"Type/id"` from `Reference`.
- `transforms::{ sr_to_staging, sr_to_domain, bundle_to_staging(_with_validation), bundle_to_domain(_with_validation), IngestionError }`
- `validation::{ validate_bundle, validate_sr, ValidationMode, ValidationReport, ValidationIssue, ValidationSeverity, RequirementRef, Validated }`

## Key rules
- `IngestionError` surfaces missing/invalid fields, invalid resource types, invalid status/intent, decode failures, and **validation** failures.
- `ValidationMode::Strict` blocks bundles with errors; `Lenient` returns a report alongside values.
- `description_from_sr` falls back: `ServiceRequest.description` -> `code.text` -> first `coding.display` -> `"unspecified service request"`.

## Tests
- Unit tests cover invalid resource types, invalid status/intent, strict/lenient validation, and relationship checks (missing Patient/Encounter in Bundle).

## Cross‑links
- FHIR ingestion MVP: `docs/kanban/feature/002-fhir-pipeline-mvp.md`
- FHIR behavior & requirements: `docs/system-design/fhir/**`
