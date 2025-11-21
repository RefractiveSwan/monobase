# Crate: lib/platform/test_suite — `refractive_swan_test_suite`

**Purpose**  
Reusable fixtures/assertions and a full test harness (unit, integration, E2E) spanning ingestion → mapping → datamart → web API.

**Env**
- Eagerly loads `platform.test_suite` via `refractive_swan_configuration`.
- `ping()` returns `"test-suite-ready"` post‑init.

**Exports**
- `assertions`:
  - `assert_json_roundtrip<T>(&T)`
  - `assert_service_request_integrity(&ServiceRequest)`
  - `assert_scenario_consistency(&ServiceRequestScenario)`
- `fixtures`:
  - Scenario builders (seeded helpers)
  - Mapping fixtures for CPT/SNOMED/NCIt/unknown (`mapping_*` fns)
- `regression`:
  - Accessors for embedded JSON fixtures:
    - `baseline_service_request()`
    - `baseline_fhir_bundle()`
    - `fhir_bundle_missing_subject()`
    - `fhir_bundle_invalid_status()`
    - `fhir_bundle_extra_codings()`
    - `fhir_bundle_uppercase_status()`
    - `fhir_bundle_unknown_code()`
    - `fhir_bundle_missing_encounter()`

**Test suites**
- **E2E** (`tests/e2e/`):
  - `fhir_ingest_flow.rs` — flats vs coding counts; ID normalization checks
  - `mapping_pipeline.rs` — end‑to‑end NCIt mapping (expects `NCIT:C19951`)
  - `observability_metrics.rs` — metrics snapshot after pipeline run
  - `service_request_flow.rs` — scenario invariants + serde round‑trip
- **Integration** (`tests/integration/`):
  - `fhir_ingest.rs` — strict validation errors/warnings (issue IDs)
  - `mapping.rs` — state + metadata (license_tier, source_kind)
  - `datamart.rs` — dims/facts wiring + `NO_MATCH` sentinel
  - `validation.rs` — missing subject/encounter/status cases
  - `web_api.rs` — `/api/map-bundles`, `/metrics/summary`, `/health` via Axum
- **Unit** (`tests/unit/`):
  - `mapping_properties.rs` — property‑based ranking invariants
  - `property_roundtrip.rs` — seeded scenario invariants

**Dev deps**
- `axum`, `tokio`, `reqwest`, `http-body-util`, `tower`
- `proptest` (property tests)

**Run**
```bash
cd code
cargo test -p refractive_swan_test_suite
```
