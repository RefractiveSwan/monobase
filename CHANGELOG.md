# Changelog

All notable changes to this repository are documented here.  
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [SemVer](https://semver.org/).

## [Unreleased]

### Added
- FE-028 – “028-01 – UI architecture & style system” (documented `views/**` layout, centralized Tailwind theme tokens, wired config-driven navbar/footer links, and added the `/ui/components` preview route plus page shell/breadcrumb/callout support).
- FE-028 – “028-02 – Mapping workflow improvements” (inline validation summary fed by the API’s validation report, upload history cache + HTMX replay, sample Bundle templates sourced from `lib/domain/meta/evaluation/data/regression`, an expanded NoMatch remediation pane, an NDJSON download endpoint for the latest mapping run, a cohort CSV export route, and compliance/licensing banners powered by pipeline metrics).
- FE-028 – “028-03 – Analytics & Observability surfacing” (dedicated `/observability` page with vector/compliance/dataset health cards + live log panel, analytics summary filters with HTMX chart fragments, and pollable `/logs/latest` + `/analytics/summary/fragment` endpoints).
- FE-028 – “028-04 – Evaluation control center” (tier-aware dataset catalog sourced from manifest `tier` metadata, HTMX job queue + history, calibration score-bucket/system charts, and a compare-runs panel that diffs precision/recall/coverage deltas).
- FE-028 – “028-05 – Accessibility & docs” (frontend accessibility checklist, `docs/runbook/frontend-workbench.md`, screenshot-based mdBook chapter, Playwright smoke tests under `tools/ui-smoke-tests`, and env var documentation baked into the runbook).
- FE-028 – “028-06 – Domain & platform surface area” (Observability now shows terminology insights, compliance policy breakdowns, ingestion validation stats, and vector backend toggles that propagate to `/api/map-bundles`).  
- FE-028 – “028-07 – Dataset & storage management” (new `/admin/datasets` page with dataset catalog table, manifest/NDJSON uploader, fixture downloads, regression smoke job queue calling `refractive_swan_test_suite`, datamart reset controls, and a maintenance console for cache clearing).
- FE-028 – “028-08 – Mesh & future node readiness” (reserved `/mesh` page visualizing `MeshNodeId`/`NodeCapabilities`, stubbed EvalDataset/Analytics job rows, governance/fallback notes, and vector-control alignment for per-node contexts).
- FE-028 – “028-09 – API & configuration surfacing” (added `/environment` config surface with feature flags and diagnostics JSON download, vector/mesh flag widget, and affordances to pull logs/metrics for debugging).
- REFR-027 – `refractive_swan_datamart` and `refractive_swan_vector_store` now live under `lib/platform/data/{data-plane,mart,data-stores/vector_store}`, and the new `refractive_swan_mesh_node` crate publishes the shared `NodeDataPlane` so mesh/runtime work can reuse it outside `refractive_swan_api`.
- REFR-027 – Mesh DTO veneer (`refractive_swan_mesh_dto`) re-exports mesh node/hub contracts and updates the architecture/kanban docs so DTO surfaces (web, CLI, mesh) have dedicated crates under `lib/dto`.
- FHIR-CONF-015 – External validator model and client (`refractive_swan_ingestion::validation::external` adds OperationOutcome/ExternalValidationReport, `validate_bundle_external`, and env template for external FHIR validation).
- FHIR-CONF-015 – Internal+external validation wiring and CLI (`ValidationMode` gains ExternalPreferred/ExternalStrict, `validate_bundle_with_external_profile` merges OperationOutcome issues, and new `refractive_swan_cli validate_fhir` emits NDJSON issues/summary with optional external profile support). 
- VEC-013 – “VectorStore abstraction & Qdrant/mock wiring” (checkboxes: `refractive_swan_vector_store` crate + config, `VectorRankerBackend`, mock store tests, `build-vector-index` CLI).
- FHIR-018 – “FHIR profiles crate + profile validation” (checkboxes: `refractive_swan_fhir_profiles` embedded StructureDefinitions, `profile_requirement_links`, `validate_sr_profile` + profile-driven bundle validation paths, docs/runbook updates).
- TERM-014 – Terminology client abstraction + optional HTTP/mock wiring (`refractive_swan_terminology` client module, composite + mock client, mapping external lookup counters, env example, terminology APIs quickstart).
- TERM-019 – “TERM-01 – OBO graph crate”, “TERM-02 – Reasoning utilities”, “TERM-03 – Integration with terminology & mapping”, “TERM-04 – Tests & fixtures”, “TERM-05 – Docs” (checkboxes completed; obo-graph feature flag, cached graph queries, fixtures/versions surfaced in terminology layer and mapping hooks).
- LIC-020 – Compliance policy crate + mapping/CLI enforcement (mode-aware Policy loader, `license_blocked` NoMatch reason, CLI summaries, export guard helper).
- LIC-020 – Compliance metrics + API/warehouse guardrails (pipeline metrics track `license_blocked` + compliance mode, map_bundles fail-fast, API export compliance blocks writes, runbook documents modes and env flags).
- REFR-024 – `refractive_swan_ingestion::ValidatedBundle` + `bundle_to_mapped_sr_from_validated_bundle` so CLIs/pipeline callers can validate once and reuse the report alongside staging/mapping output.
- REFR-024 – Terminology `obo_graph` runtime loader (`load_ontology_graph_from_path`) and cache instrumentation/tests for synthetic large graphs plus documentation of supported inputs.
- REFR-024 – Platform README now documents env namespaces + crate responsibilities, and `refractive_swan_test_suite` ships a `TempSqliteWarehouse` helper for temporary datamart instances.

### Changed
- MESH-030 – “P1.1/P1.3 mesh node wiring” (checkboxes: deterministic `NodePlaneConfig::from_env_with_node_id`, `/metrics/summary` now emits `MetricsSnapshot`, new `/mesh/health` `/mesh/capabilities` `/mesh/governance` + `/mesh/job` endpoints, `MeshJobType` variants finalized, NodeDataPlane `run_mesh_job` covers eval/analytics/health/export/introspection with regression-based MappingHealthCheck output, and stub hub endpoints for `/hub/nodes`, `/hub/jobs/analytics/ncit-summary`, `/hub/jobs/eval`).
- MESH-030 – “P3.1 governance on node jobs” (checkbox completed: `QueryDescriptor` + `NodePolicy` budget/DP handling, governance-wrapped mesh jobs with requester-tagged metrics and pseudonymous audit traces).
- MESH-030 – “P3.2 hub governance adapter” (checkbox completed: hub-side allow/deny rules by compliance mode/tags/export policy, hashed requester propagation, pre-built NodePolicy injection with tests, and JobQueue dispatch short-circuit for denied nodes).
- MESH-030 – Cache/DP budget hardening (Redis backend for `refractive_swan_cache_store` with env toggle + optional Redis integration test, DP budgets recorded to `mesh_dp_budget` ledger, `/mesh/health` cache fields surfaced to frontend diagnostics, and ledger ready to switch to Postgres/DuckDB once `refractive_swan_relational_store` backends land).
- MESH-030 – Mesh frontend DTOs for dashboards (`NodeView`, `FederatedEvalView`, `MeshTogglesView` added to `refractive_swan_web_dto` for mesh/hub/health responses).
- MESH-030 – Mesh API alignment for dashboards (`/admin/toggles` returns mesh-aware view, `/hub/jobs/eval` now emits `FederatedEvalView` keyed by node_id, and hub routes gain coverage for `/hub/jobs/*` payloads).
- BE-029 – “Ensure each mesh entity reports metrics tagged with mesh_node_id so observability dashboards can pivot per node” (checkbox completed).
- REFR-022 – “Restructure refractive_swan_core into super-domains (primitives/clinical/interop/semantics) with bridges under consumer modules” (checkbox completed).
- REFR-022 – “Preserve legacy public API paths via lib.rs re-exports and prelude wiring” (checkbox completed).
- REFR-05  – FHIR profiles folded into `refractive_swan_ingestion::profiles` with crate README/docs, external validator port/context, and CLI/test adapters owning HTTP/env; workspace crate paths aligned to `evaluation/` + `ontologies/` layout.
- REFR-06  – Mapping engine docs/config cleanup (`MappingConfig` injects policy/thresholds/versions, env reads removed; README + crate docs added; vector ranker remains trait-based).
- REFR-06  – Compliance policy env parsing now centralized in `refractive_swan_compliance::ComplianceConfig` (using `refractive_swan_configuration` namespace loading); CLI (`map_codes`, `map_bundles`, `load_datamart`), API (`refractive_swan_api`), and datamart loaders now build the policy once per process instead of calling `load_policy_from_env` per request; and the deprecated `refractive_swan_mapping::eval::run_eval` shim has been removed in favor of `refractive_swan_eval::run_eval_with_mapper`.
- REFR-12  – `refractive_swan_observability` no longer panics on env load (`init_environment` returns `Result` and logging helpers warn instead of aborting), adds JSON-ready `metrics_snapshot` helpers plus vector usage applicators, documents PipelineMetrics ownership in a crate README, and adds tests covering env failures, snapshot ratios, and vector capacity handling.
- REFR-14  – `refractive_swan_test_suite` now documents its fixtures/assertions/tests in a crate README, exposes env helpers (`init_environment`, `ensure_eval_data_root`, `ScopedEnvVar`) to replace unsafe `set_var`, keeps regression data rooted in `lib/domain/meta/evaluation/data/**`, and adds a smoke-index e2e test that exercises ingestion → mapping → datamart → eval → vector flows.
- REFR-11  – `refractive_swan_compliance` README now documents env namespaces/entrypoints, `ComplianceConfig` stores the resolved workspace root and policy path via `refractive_swan_configuration` string helpers, JSON/YAML override handling/tests cover all actions and modes, export gating semantics are asserted per tier/mode, and missing/relative policy files emit clear structured errors.
- REFR-022 – Domain core helpers and docs alignment (`refractive_swan_core` README, `CodeElement::id_for`, MappingResult builders, staging/datagate alignment notes, staging serde tests, unit coverage).
- REFR-08  – Terminology + OBO graph docs and helpers (`refractive_swan_terminology`/`refractive_swan_terminology::obo_graph` READMEs, `canonicalize_system_url` for ingestion/mapping parity, terminology client env seams documented, and PET/CT synonym tests tied back to mapping use cases).
- REFR-07  – `refractive_swan_eval` now exposes a `FileDatasetStore` seam + README, CLI/API/frontend inject `refractive_swan_EVAL_DATA_ROOT`, fake-data RNGs/ID helpers were centralized with new docs/tests, and eval fixtures/regression docs now live under `lib/domain/meta/evaluation/data`.
- REFR-024 – `StgServiceRequestFlat` now carries typed `ServiceRequestStatus`/`ServiceRequestIntent` fields with serde fallbacks, and mapping helper tests pin `MappingThresholds` + default NoMatch reasons for deterministic behavior.
- REFR-024 – `CachedOntologyGraph` caches are now bounded LRU maps (per query type) so long-running services do not leak memory; docs and tests cover eviction plus runtime `.obo` ingestion paths.
- REFR-024 – `refractive_swan_pipeline::PipelineExecution` surfaces per-run `PipelineMetrics`, and CLI/API surfaces now use `log_pipeline_output_with_summary` so vector usage + AutoMapped/NeedsReview counts are merged without re-counting raw rows.
- REFR-024 – `refractive_swan_terminology` exports `canonicalize_system` (aliasing `canonicalize_system_url`) and the crate README documents how license/source metadata flows into analytics/mapping.
- REFR-024 – Platform crates now use `refractive_swan_configuration` env helpers (vector store config, compliance, observability), `refractive_swan_configuration` adds workspace-root/search-order tests, and `refractive_swan_test_suite::ensure_eval_data_root()` resolves paths without mutating process env.
- REFR-17  – `refractive_swan_web_frontend` now documents the route <-> backend map + envs, configuration uses `refractive_swan_configuration`, the HTTP client consumes `refractive_swan_contracts` DTOs with structured error handling/tests, HTMX routes/views/view-models call out their backend flows (paste/upload limits, default eval datasets, compliance/mapping state notes), new handler modules (home/mapping/analytics/eval/docs) keep routing/local behavior isolated, and Insta snapshots guard mapping/no-match/analytics/eval fragments.
- REFR-15  – `refractive_swan_cli` gains a shared `cli_core` (env/logging/error/NDJSON/compliance/vector helpers), a README that maps each binary to its domain flow + common flags, and each bin now streams input, emits API-aligned `kind` records, shares compliance/vector config, and surfaces structured exit codes (map_bundles/map_codes/eval_mapping/validate_fhir/load_datamart/build_vector_index refactored accordingly).
- REFR-02  – Added `tools/layer_lint` + `cargo make layers-check` to enforce app→domain→platform dependencies and forbid `std::env` usage in `lib/domain/**`, documented the boundary rules in `docs/system-design/base/directory-architecture.md`, removed the legacy `TerminologyClientConfig::from_env` helper, and moved external FHIR validator env handling out of `refractive_swan_ingestion`.

### Planned
- CLI application (`feature/app/cli-mvp` – 004): `refractive_swan_cli` scaffold + `map-bundles` / `generate-fhir-bundles` subcommands, flags, tests, CI smoke.
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
- EVAL-012 – “EVAL-02 – Evaluation core API” (`refractive_swan_mapping::eval::{EvalCase, EvalResult, EvalSummary, run_eval}` powering precision/recall + MappingState confusion metrics).
- EVAL-012 – “EVAL-03 – Test harness integration” (refractive_swan_test_suite fixture loader + `mapping_eval` integration tests asserting precision + NoMatch coverage).
- EVAL-012 – “EVAL-04 – CLI wrapper” (`refractive_swan_cli eval_mapping` reads gold NDJSON and prints summary + optional EvalResult lines).
- EVAL-012 – “EVAL-05 – Docs & requirements link” (`mapping-eval-quickstart` runbook + MAP_ACCURACY verification update).
- EVAL-022 – “EVAL-PLAT-01 – Eval crate & datasets” (`refractive_swan_eval` crate, `refractive_swan_EVAL_DATA_ROOT`, datasets now in `lib/domain/meta/evaluation/data/eval/`, CLI/test suite wired to named datasets).
- EVAL-022 – “EVAL-PLAT-02/03 – Advanced metrics & CI guard” (EvalSummary adds F1 + stratified metrics; `refractive_swan_cli eval_mapping` supports `--dataset` + `--thresholds` for regression gates).
- EVAL-022 – “EVAL-PLAT-04/05 – Artifacts + tiered datasets” (`refractive_swan_cli eval_mapping --out-dir/--report` writes summary/results/report artifacts; nine bronze/silver/gold datasets with updated runbooks/docs/tests; `/api/eval/summary` exposed in `refractive_swan_api`).
- EVAL-022 – “EVAL-PLAT-02 – Calibration buckets” (`EvalSummary.score_buckets` now capture deterministic 0.1 score bands with accuracy per bucket; Markdown report/runbook document the calibration view).
- EVAL-022 – “EVAL-PLAT-03 – CI regression gate” (`EvalSummary` tracks overall accuracy and AutoMapped precision; CLI thresholds/CI workflow enforce regression guards on the `gold_pet_ct_small` dataset).
- EVAL-022 – “EVAL-PLAT-04 – Dashboards & reporting” (`refractive_swan_eval::report` loads baseline snapshots and renders Markdown + HTML fragments; `refractive_swan_web_frontend` exposes an HTMX dataset picker backed by `/eval/report`; baseline fixtures documented in `lib/domain/meta/evaluation/data/eval/README.md`).
- EVAL-022 – “EVAL-PLAT-06 – Eval harness migration” (`refractive_swan_eval::run_eval_with_mapper` owns the harness + streaming NDJSON reader; `refractive_swan_mapping::eval` now exposes a deprecated shim, and CLI/API/test suites call the new surface).
- EVAL-022 – “EVAL-PLAT-07 – Dataset manifests & licensing” (all corpora ship with `<dataset>.manifest.json`; `refractive_swan_eval::load_dataset_with_manifest` validates SHA-256 + row counts, `pet_ct_extended.ndjson` joins the catalog, and CLI runs warn when manifest checksums drift).
- EVAL-022 – “EVAL-PLAT-08/09 – Determinism & top-k coverage” (`EvalSummary` adds coverage/top-k and per-system confusion; CLI gains `--deterministic` guard and `--top-k` flag; runbook updated for fingerprint-based stability checks).
- EVAL-022 – “EVAL-PLAT-08 – Stability test” (refractive_swan_test_suite adds a deterministic eval property asserting identical bytes across runs; optional `eval-advanced` feature includes bootstrap CIs).
- EVAL-022 – “EVAL-PLAT-10/11/12 – CLI thresholds, eval API, and frontend” (`eval_mapping` enforces min_top1 + allowed NoMatch reasons and uploads artifacts in CI; `refractive_swan_api` exposes dataset/run/latest endpoints with cached summaries; frontend `/eval` page adds dataset picker + HTMX fragment with metrics and NoMatch reasons).
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
  - **Backend** (`refractive_swan_api`): `POST /api/map-bundles`, `GET /health`, `GET /metrics/summary`; structured logs with request IDs; integration tests + CI smoke; directory‑architecture docs.

- **Environment & observability (007)**
  - `refractive_swan_configuration` crate loading `.env.<namespace>.<profile>`; integrated across API/frontend/CLI/fake_data/observability/test_suite.
  - Runbook & directory‑architecture updates; CI strictness for env; observability docs.

- **Docs & Makefiles (008)**
  - mdBook scaffold + runbook/kanban sync; build/serve targets.
  - `/docs` redirect in web frontend via `refractive_swan_DOCS_URL`.
  - Workspace `Makefile` with standard dev/CI/doc targets and CLI wrappers.
  - Makefile quickstart runbook.

- **NCIt analytics mart (009)**
  - `refractive_swan_datamart` crate: dims/facts mirroring ERD; stable surrogate keys; pipeline→mart mappers with deterministic dedupe.
  - Sentinel **NO_MATCH** NCIt dimension for `NoMatch` facts.
  - Unit/integration tests; docs updated to reflect implementation.

- **FHIR validation profiles (010)**
  - Validation model (`ValidationIssue`, severities) for SR+Bundle; requirement‑linked checks; `ValidationMode` and `Validated<T>` sidecar.
  - Fixtures/tests; verification docs & validation quickstart.

- **Terminology layer (011)**
  - `refractive_swan_terminology` crate (codesystem metadata with license tiers/source kinds; OBO registry).
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
