# FHIR profiles quickstart

Use this runbook when updating embedded FHIR `StructureDefinition` snapshots or exercising profile-aware validation.

## What ships today

- Embedded HL7 profiles for `Patient`, `Encounter`, and `ServiceRequest` live under `data/fhir/profiles/*.json`.
- `dfps_fhir_profiles` parses and exposes these snapshots via `load_profile(<url>)`.
- `dfps_ingestion` enables the `profile_validation` feature by default, layering profile cardinality checks (`ServiceRequest.id/status/intent/subject`) on top of hand-written validation and optional external `$validate` calls.

## Update flow

1) Edit `data/fhir/profiles/<Resource>.json` with the desired `snapshot.element` entries. Keep only the DFPS-relevant paths to avoid unnecessary churn.
2) Run the profile loader tests:
   ```bash
   cargo test -p dfps_fhir_profiles
   ```
3) Verify ingestion validation (profile + hand-written) still passes:
   ```bash
   cargo test -p dfps_ingestion
   cargo test -p dfps_test_suite --tests integration::validation
   ```
4) If using an external validator, set `DFPS_FHIR_VALIDATOR_BASE_URL`/`DFPS_FHIR_VALIDATOR_PROFILE` and call:
   ```bash
   cargo run -p dfps_cli --bin validate_fhir -- --profile <profile-url> ./bundle.ndjson
   ```
5) Update system-design docs and Kanban notes when profiles change (`docs/system-design/clinical/fhir/requirements/ingestion-requirements.md`, `docs/kanban/feature/mvp/018-fhir-profiles-and-structuredefinition.md`).
