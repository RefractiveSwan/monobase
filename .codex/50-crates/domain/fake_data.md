# Crate: lib/domain/fake_data - `refractive_swan_fake_data`

**Path:** `code/lib/domain/fake_data`  
**Depends on:** `refractive_swan_core` (with `dummy`), `rand`, `fake`, `serde(_json)`, `refractive_swan_configuration`.

## Responsibilities
- Deterministic, **seeded** generators for domain + minimal FHIR.
- CLI tools for generating **scenarios** and **raw FHIR Bundles** as JSON/NDJSON.

## Modules & bins
- `value.rs` - ID/status/intent/description fakers; seeded helpers.
- `patient.rs`, `encounter.rs`, `order.rs` - domain entity generators.
- `scenarios.rs` - cohesive `ServiceRequestScenario { patient, encounter, service_request }`.
- `raw_fhir.rs` - fake **FHIR** `Patient`, `Encounter`, `ServiceRequest`, and `Bundle` with plausible codings (SNOMED/CPT/LOINC); includes normalization to keep intent/status coherent.
- `bin/generate_sample.rs` - emits **domain** scenarios (reads env via `refractive_swan_configuration`).
- `bin/generate_fhir_bundle.rs` - emits **FHIR Bundle** NDJSON; supports `--seed`, `--count`.

## Conventions
- Always provide `*_with_seed` and `*_with_rng` for determinism.
- Prefer minimal surface area for FHIR mock data; keep display/system/code realistic.

## Tests
- Round‑trip serde tests where helpful.
- Keep RNG usage explicit in tests (`StdRng::seed_from_u64`).
