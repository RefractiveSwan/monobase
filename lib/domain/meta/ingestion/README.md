# dfps_ingestion – FHIR → staging/domain + validation

`dfps_ingestion` keeps the pure domain logic for normalizing minimal FHIR Bundles into staging rows and ServiceRequest aggregates, plus validation helpers (hand-written + optional profile/external hooks). HTTP/env concerns are left to app/platform layers.

## System-design links
- docs/system-design/clinical/fhir/index.md
- docs/system-design/clinical/fhir/requirements/ingestion-requirements.md
- docs/runbook/020-fhir/fhir-profiles-quickstart.md
- docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-05--domain-ingestion--fhir-profiles

## Modules
- `transforms` – `sr_to_staging` / `bundle_to_staging` → `StgServiceRequestFlat` + `StgSrCodeExploded`; `sr_to_domain` / `bundle_to_domain` → `ServiceRequest`.
- `validation` – requirement-linked checks (`ValidationIssue`, `ValidationReport`, `ValidationMode`), bundle/ServiceRequest validators, and the `ExternalValidator` port + `ExternalValidationContext` for injecting `$validate` calls.
- `profiles` (feature `profile_validation`) – embedded `StructureDefinition` snapshots (`FhirProfile`) for Patient/Encounter/ServiceRequest; loaded via `profiles::load_profile` and applied within validation.

## Notes & invariants
- No env/file IO inside the crate; external validation is injected via `ExternalValidator`.
- Profile snapshots stay colocated with ingestion for deterministic validation; update via the runbook.
- Defaults prefer `ValidationMode::Lenient`; ExternalStrict/Preferred require providing an external validator in the context.
