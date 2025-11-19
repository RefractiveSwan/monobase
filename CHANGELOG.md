# Changelog

All notable changes to this repository are documented here.  
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [SemVer](https://semver.org/).

## [Unreleased]

### Added
- FHIR-CONF-015 – External validator model and client (`dfps_ingestion::validation::external` adds OperationOutcome/ExternalValidationReport, `validate_bundle_external`, and env template for external FHIR validation).
- FHIR-CONF-015 – Internal+external validation wiring and CLI (`ValidationMode` gains ExternalPreferred/ExternalStrict, `validate_bundle_with_external_profile` merges OperationOutcome issues, and new `dfps_cli validate_fhir` emits NDJSON issues/summary with optional external profile support). 
- VEC-013 – “VectorStore abstraction & Qdrant/mock wiring” (checkboxes: `dfps_vector_store` crate + config, `VectorRankerBackend`, mock store tests, `build-vector-index` CLI).
- FHIR-018 – “FHIR profiles crate + profile validation” (checkboxes: `dfps_fhir_profiles` embedded StructureDefinitions, `profile_requirement_links`, `validate_sr_profile` + profile-driven bundle validation paths, docs/runbook updates).
- TERM-014 – Terminology client abstraction + optional HTTP/mock wiring (`dfps_terminology` client module, composite + mock client, mapping external lookup counters, env example, terminology APIs quickstart).
- TERM-019 – “TERM-01 – OBO graph crate”, “TERM-02 – Reasoning utilities”, “TERM-03 – Integration with terminology & mapping”, “TERM-04 – Tests & fixtures”, “TERM-05 – Docs” (checkboxes completed; obo-graph feature flag, cached graph queries, fixtures/versions surfaced in terminology layer and mapping hooks).
- LIC-020 – Compliance policy crate + mapping/CLI enforcement (mode-aware Policy loader, `license_blocked` NoMatch reason, CLI summaries, export guard helper).
- LIC-020 – Compliance metrics + API/warehouse guardrails (pipeline metrics track `license_blocked` + compliance mode, map_bundles fail-fast, API export compliance blocks writes, runbook documents modes and env flags).

### Changed
- REFR-022 – “Restructure dfps_core into super-domains (primitives/clinical/interop/semantics) with bridges under consumer modules” (checkbox completed).
- REFR-022 – “Preserve legacy public API paths via lib.rs re-exports and prelude wiring” (checkbox completed).
- REFR-05  – FHIR profiles folded into `dfps_ingestion::profiles` with crate README/docs, external validator port/context, and CLI/test adapters owning HTTP/env; workspace crate paths aligned to `evaluation/` + `ontologies/` layout.
- REFR-06  – Mapping engine docs/config cleanup (`MappingConfig` injects policy/thresholds/versions, env reads removed; README + crate docs added; vector ranker remains trait-based).
- REFR-06  – Compliance policy env parsing now centralized in `dfps_compliance::ComplianceConfig` (using `dfps_configuration` namespace loading); CLI (`map_codes`, `map_bundles`, `load_datamart`), API (`dfps_api`), and datamart loaders now build the policy once per process instead of calling `load_policy_from_env` per request; and the deprecated `dfps_mapping::eval::run_eval` shim has been removed in favor of `dfps_eval::run_eval_with_mapper`.
- REFR-022 – Domain core helpers and docs alignment (`dfps_core` README, `CodeElement::id_for`, MappingResult builders, staging/datagate alignment notes, staging serde tests, unit coverage).
- REFR-08  – Terminology + OBO graph docs and helpers (`dfps_terminology`/`dfps_obo_graph` READMEs, `canonicalize_system_url` for ingestion/mapping parity, terminology client env seams documented, and PET/CT synonym tests tied back to mapping use cases).
- REFR-07  – `dfps_eval` now exposes a `FileDatasetStore` seam + README, CLI/API/frontend inject `DFPS_EVAL_DATA_ROOT`, fake-data RNGs/ID helpers were centralized with new docs/tests, and eval fixtures/regression docs now live under `lib/domain/evaluation/fake_data`.

### Planned
- CLI application (`feature/app/cli-mvp` – 004): `dfps_cli` scaffold + `map-bundles` / `generate-fhir-bundles` subcommands, flags, tests, CI smoke.
- Desktop application (`feature/app/desktop-mvp` – 006): shell scaffold, pipeline wiring, minimal UI, export, logging, docs.
- Frontend CI hook (`WEB-FE-05` optional) to build and run critical tests.
- FHIR serde field name audit & seed determinism check (002 – REVIEW).
- Mapping REVIEW checks (003): mock-table/code coverage, mapping-state alignment.
- Docs & Makefiles REVIEW checks (008): `/docs` redirect confirmation; make targets green on clean checkout.
- Datamart REVIEW checks (009): ERD alignment, regression safety.
- Validation REVIEW checks (010): requirement ID consistency; error separation clarity.
- Terminology REVIEW checks (011): license/source modeling; stability; doc alignment.

### Added
- EVAL-012 – “EVAL-01 – Gold standard format” (documented NDJSON schema + PET/CT gold sample).
- EVAL-012 – “EVAL-02 – Evaluation core API” (`dfps_mapping::eval::{EvalCase, EvalResult, EvalSummary, run_eval}` powering precision/recall + MappingState confusion metrics).
- EVAL-012 – “EVAL-03 – Test harness integration” (dfps_test_suite fixture loader + `mapping_eval` integration tests asserting precision + NoMatch coverage).
- EVAL-012 – “EVAL-04 – CLI wrapper” (`dfps_cli eval_mapping` reads gold NDJSON and prints summary + optional EvalResult lines).
- EVAL-012 – “EVAL-05 – Docs & requirements link” (`mapping-eval-quickstart` runbook + MAP_ACCURACY verification update).
- EVAL-022 – “EVAL-PLAT-01 – Eval crate & datasets” (`dfps_eval` crate, `DFPS_EVAL_DATA_ROOT`, datasets now in `lib/domain/evaluation/fake_data/data/eval/`, CLI/test suite wired to named datasets).
- EVAL-022 – “EVAL-PLAT-02/03 – Advanced metrics & CI guard” (EvalSummary adds F1 + stratified metrics; `dfps_cli eval_mapping` supports `--dataset` + `--thresholds` for regression gates).
- EVAL-022 – “EVAL-PLAT-04/05 – Artifacts + tiered datasets” (`dfps_cli eval_mapping --out-dir/--report` writes summary/results/report artifacts; nine bronze/silver/gold datasets with updated runbooks/docs/tests; `/api/eval/summary` exposed in `dfps_api`).
- EVAL-022 – “EVAL-PLAT-02 – Calibration buckets” (`EvalSummary.score_buckets` now capture deterministic 0.1 score bands with accuracy per bucket; Markdown report/runbook document the calibration view).
- EVAL-022 – “EVAL-PLAT-03 – CI regression gate” (`EvalSummary` tracks overall accuracy and AutoMapped precision; CLI thresholds/CI workflow enforce regression guards on the `gold_pet_ct_small` dataset).
- EVAL-022 – “EVAL-PLAT-04 – Dashboards & reporting” (`dfps_eval::report` loads baseline snapshots and renders Markdown + HTML fragments; `dfps_web_frontend` exposes an HTMX dataset picker backed by `/eval/report`; baseline fixtures documented in `lib/domain/evaluation/fake_data/data/eval/README.md`).
- EVAL-022 – “EVAL-PLAT-06 – Eval harness migration” (`dfps_eval::run_eval_with_mapper` owns the harness + streaming NDJSON reader; `dfps_mapping::eval` now exposes a deprecated shim, and CLI/API/test suites call the new surface).
- EVAL-022 – “EVAL-PLAT-07 – Dataset manifests & licensing” (all corpora ship with `<dataset>.manifest.json`; `dfps_eval::load_dataset_with_manifest` validates SHA-256 + row counts, `pet_ct_extended.ndjson` joins the catalog, and CLI runs warn when manifest checksums drift).
- EVAL-022 – “EVAL-PLAT-08/09 – Determinism & top-k coverage” (`EvalSummary` adds coverage/top-k and per-system confusion; CLI gains `--deterministic` guard and `--top-k` flag; runbook updated for fingerprint-based stability checks).
- EVAL-022 – “EVAL-PLAT-08 – Stability test” (dfps_test_suite adds a deterministic eval property asserting identical bytes across runs; optional `eval-advanced` feature includes bootstrap CIs).
- EVAL-022 – “EVAL-PLAT-10/11/12 – CLI thresholds, eval API, and frontend” (`eval_mapping` enforces min_top1 + allowed NoMatch reasons and uploads artifacts in CI; `dfps_api` exposes dataset/run/latest endpoints with cached summaries; frontend `/eval` page adds dataset picker + HTMX fragment with metrics and NoMatch reasons).
- EVAL-022 – “EVAL-PLAT-13/14 – Performance benches & reporting artifacts” (streaming eval reader + Criterion benches, CLI chunking, baseline compare flag, and `pet_ct_small.baseline.json` for delta reporting).

---

## [0.1.0] – 2025‑11‑15

### Added
- **Base skeleton (001)**
  - Domain model (`lib/core`): bounded contexts; value objects/newtypes; serde‑ready entities; JSON round‑trip tests.
  - Fake data (`lib/fake_data`): `fake`/`Dummy` wiring; coherent aggregate generators; deterministic seeding; NDJSON sample binary.
  - Test suite (`lib/test_suite`): shared fixtures/assertions; unit/integration/e2e layout; property tests; regression fixtures; CI for fmt/clippy/tests.
  - Observability & ergonomics: structured logging hooks; metrics snapshot e2e test; CLI ergonomics/docs.
  - Workspace wiring: workspace layout & manifests; compiles cleanly.

- **FHIR pipeline MVP (002)**
  - Typed minimal FHIR R4 + staging models; transforms (`sr_to_staging`, `bundle_to_staging/domain`).
  - Raw FHIR generators & NDJSON bundle CLI.
  - E2E/property/regression tests; docs alignment & quickstart.
  - Public facade `bundle_to_mapped_sr`; pipeline CLI for bundle mapping.
  - Validation & error handling coverage; messy FHIR regression fixtures.

- **Mapping NCIt skeleton (003)**
  - Mapping/core types; mapping engine with lexical + deterministic vector‑mock rankers + rule re‑ranker.
  - Embedded NCIt/UMLS mock data; pipeline function (`map_staging_codes`) + optional CLI.
  - Golden/property/regression tests; provenance/thresholds; mapping state machine; explainability helpers.
  - Docs/diagrams; workspace & CI.

- **App / Web MVP (005)**
  - **Frontend**: project scaffold; backend discovery; API client; upload/paste bundle; mapping table; metrics dashboard; NoMatch explorer; unit coverage; UX/copy help text & empty/error states; docs & quickstart.
  - **Backend** (`dfps_api`): `POST /api/map-bundles`, `GET /health`, `GET /metrics/summary`; structured logs with request IDs; integration tests + CI smoke; directory‑architecture docs.

- **Environment & observability (007)**
  - `dfps_configuration` crate loading `.env.<namespace>.<profile>`; integrated across API/frontend/CLI/fake_data/observability/test_suite.
  - Runbook & directory‑architecture updates; CI strictness for env; observability docs.

- **Docs & Makefiles (008)**
  - mdBook scaffold + runbook/kanban sync; build/serve targets.
  - `/docs` redirect in web frontend via `DFPS_DOCS_URL`.
  - Workspace `Makefile` with standard dev/CI/doc targets and CLI wrappers.
  - Makefile quickstart runbook.

- **NCIt analytics mart (009)**
  - `dfps_datamart` crate: dims/facts mirroring ERD; stable surrogate keys; pipeline→mart mappers with deterministic dedupe.
  - Sentinel **NO_MATCH** NCIt dimension for `NoMatch` facts.
  - Unit/integration tests; docs updated to reflect implementation.

- **FHIR validation profiles (010)**
  - Validation model (`ValidationIssue`, severities) for SR+Bundle; requirement‑linked checks; `ValidationMode` and `Validated<T>` sidecar.
  - Fixtures/tests; verification docs & validation quickstart.

- **Terminology layer (011)**
  - `dfps_terminology` crate (codesystem metadata with license tiers/source kinds; OBO registry).
  - Staging↔terminology enrichment; `CodeKind` classification; mapping integration with reason codes (unknown/missing system).
  - Unit/integration tests; comprehensive docs across clinical FHIR/NCIt sections.

### Changed
- Unified docs and directory architecture references to clinical paths.
- Pipeline/Mapping surfaces instrumented with structured logs & metrics.

### Testing / CI
- Workspace CI: fmt, clippy (`-D warnings`), tests across crates.
- Golden/property/regression coverage for mapping & pipeline invariants.
- Web backend integration tests + CI smoke; datamart invariants tests.

### Notes
- CLI (004) and Desktop (006) are intentionally out of scope for this release; tracked in roadmap.
