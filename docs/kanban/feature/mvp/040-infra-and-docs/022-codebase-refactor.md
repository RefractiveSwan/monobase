# Kanban - feature/codebase-refactor (022)

**Theme:** Docs polish - public hosting, full-text search, theming  
**Branch:** `feature/meta/REFR-022-codebase-refactor`  
**Goal:** Turn the local mdBook into a searchable, themed, publicly hosted documentation site, integrated with `/docs` in the frontend.

> Status: **INPROGRESS**  
> Branch target version: `v0.1.0`  
> Introduced in: `v0.1.0`  
> Last updated in: `Unreleased`

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / polish  
* **DONE** – Completed  

---

## TODO

### REFR-01 – Docs (lib/app, lib/domain, lib/platform)

**Goal:** Every subdirectory under `lib/app`, `lib/domain`, and `lib/platform` is documented, and any docs-hosting/search logic is in the *right* crate with a clear boundary.

- [ ] For each of the following crates, work through organizing and refactoring the primary logic base:
  - [ ] `lib/app/`
    - [ ] `lib/app/cli` (`dfps_cli`)
      - [ ] Extract shared NDJSON streaming/logging/compliance helpers (used in map_bundles/map_codes/validate_fhir/load_datamart) into an internal module to cut duplication.
      - [ ] Avoid double-validation in `map_bundles` by plumbing ingestion validation data from `bundle_to_mapped_sr` instead of re-running `validate_bundle`.
      - [ ] Add a streaming mapping path in `map_codes` (no upfront Vec) that still propagates vector usage metrics and compliance failures.
      - [ ] Centralize CLI config/logging setup via `dfps_configuration` (namespace `app.cli`) so each bin stops initializing env_logger/env parsing manually.
    - [ ] `lib/app/web`
      - [ ] Add app-level README describing the split between frontend (Actix) and backend (Axum) and where shared DTOs/configs live.
      - [ ] Introduce a shared web DTO module so frontend client types, backend API responses, and analytics structs do not diverge.
      - [ ] `lib/app/web/frontend` (`dfps_web_frontend`)
        - [ ] Replace manual env parsing in `config.rs` with a typed config sourced from `dfps_configuration` (timeout validation, docs URL normalization).
        - [ ] Centralize backend client error handling/logging (currently inline in routes) and emit analytics request metrics via `dfps_observability`.
        - [ ] Add regression test for `/analytics` error handling (backend 5xx/timeout) to keep user-facing messages stable.
      - [ ] `lib/app/web/backend` (`dfps_web_backend`)
        - [ ] Document ownership/boundaries for API vs datamart crates and the expected env namespaces for each.
        - [ ] Hoist shared analytics DTOs into a reusable module so API handlers and frontend client structs stay in sync.
        - [ ] `lib/app/web/backend/api` (`dfps_api`)
          - [ ] Add a config module (using `dfps_configuration`) for host/port/warehouse/compliance instead of scattered `std::env::var` lookups in `server.rs`.
          - [ ] Extract analytics persistence/state management into a component with eviction/metrics to avoid unbounded HashMap growth.
          - [ ] Load compliance policy once at startup and thread it through handlers instead of calling `load_policy_from_env` per request.
        - [ ] `lib/app/web/backend/datamart` (`dfps_datamart`)
          - [ ] Refactor `WarehouseConfig::from_env` to use `dfps_configuration` validation (url/schema/pool) and cover it with unit tests.
          - [ ] Add idempotent migration/load tests for NO_MATCH handling, duplicate SR rows, and compliance export filtering.
          - [ ] Provide a streaming insert API so CLI/API callers don’t buffer full PipelineOutput lists before writing to SQLite.
  - [ ] `lib/domain`
    - [ ] Document domain layering (core/ingestion/mapping/eval/terminology) and keep domain crates free of direct environment reads.
    - [ ] Align error/reporting semantics across domain crates (ValidationError/Policy errors) to simplify pipeline/app boundaries.
    - [ ] `lib/domain/core` (`dfps_core`)
      - [ ] Add a `MappingResult` builder/constructor to standardize reason/state handling instead of duplicating logic in downstream crates.
      - [ ] Expose a helper for constructing stable `CodeElement` IDs so ingestion/mapping/datamart share the same format.
      - [ ] Add a crate README tying modules to system-design docs and clarifying staging vs mapping vs order invariants.
    - [ ] `lib/domain/evaluation/eval` (`dfps_eval`)
      - [ ] Replace raw `std::env` dataset root resolution with a config struct built via `dfps_configuration` (still honoring DFPS_EVAL_DATA_ROOT).
      - [ ] Split IO/parsing from scoring so eval functions accept injected readers/writers instead of reading files directly.
      - [ ] Add determinism/benchmark tests for fingerprint computation across chunk sizes and `top_k` settings.
    - [ ] `lib/domain/evaluation/eval::fake_data` (`dfps_eval::fake_data`)
      - [ ] Add module docs describing generator outputs/seed controls and link them from `data/eval/README.md`.
      - [ ] Centralize RNG seeding helpers to keep fixtures deterministic across modules and CLI bins.
      - [ ] Provide a thin config wrapper over `dfps_configuration` for the generators instead of ad-hoc env access.
    - [ ] `lib/domain/ingestion` (`dfps_ingestion`)
      - [x] Embed FHIR profiles under `dfps_ingestion::profiles` with tests for required snapshot elements and documented URLs.
      - [x] Add a crate README and crate-level docs describing the Bundle → staging/domain flow and profile hook.
      - [x] Extract a validator port (`ExternalValidator` + `ExternalValidationContext`) so app/platform layers own HTTP clients; keep transforms/profile loading pure.
      - [ ] Introduce a validated-bundle type that carries the `ValidationReport` to callers to avoid double validation in CLI/pipeline.
      - [ ] Return typed status/intent enums from parsing helpers (instead of strings) to reduce downstream re-parsing.
    - [ ] `lib/domain/mapping` (`dfps_mapping`)
      - [ ] Remove implicit env access (`load_policy_from_env`) from mapping functions; require Policy/config injection from the app layer.
      - [ ] Decouple vector-store wiring from the core engine (accept `VectorStoreConfig`/store from callers) and keep a pure deterministic constructor for tests.
      - [ ] Add deterministic tests that pin `MappingThresholds` + reason strings used in `build_result_with_score`.
    - [ ] Terminology `obo_graph` module (`lib/domain/ontologies/terminology`)
      - [ ] Document supported graph inputs and add a runtime loader for `.obo` paths (not just embedded minis).
      - [ ] Add an integration test exercising `CachedOntologyGraph` caching/eviction with larger sample graphs.
    - [ ] `lib/domain/pipeline` (`dfps_pipeline`)
      - [ ] Stop reading `VectorStoreConfig` from env inside `bundle_to_mapped_sr`; require injected config/store and surface errors instead of silent fallback.
      - [ ] Emit dfps_observability metrics/logging for ingestion + vector paths so downstream apps don’t re-count manually.
      - [ ] Add tests for vector-enabled vs offline paths to keep `vector_usage` semantics stable.
    - [ ] `lib/domain/ontologies/terminology` (`dfps_terminology`)
      - [ ] Expose code-system normalization helpers (`canonicalize_system`) for reuse by ingestion/mapping to avoid drift.
      - [ ] Add a crate README documenting license/source metadata semantics expected by analytics/mapping.
  - [ ] `lib/platform`
    - [ ] Add a short platform README clarifying when to add new platform crates vs domain/app modules and the env namespaces to use.
    - [ ] Deduplicate env-flag/number parsing helpers across platform crates (observability/vector_store/compliance/test_suite).
    - [ ] `lib/platform/compliance` (`dfps_compliance`)
      - [ ] Move env parsing into a dedicated config builder (DFPS_COMPLIANCE_*) with structured errors instead of panicking in `load_policy_from_env`.
      - [ ] Add tests for policy override file parsing and `assert_export_allowed` behavior across modes.
    - [ ] `lib/platform/configuration` (`dfps_configuration`)
      - [ ] Publish shared env parsing helpers (bool/int/port) to replace bespoke logic in web frontend/API/vector store configs.
      - [ ] Add tests for workspace root resolution and env search ordering (DFPS_ENV_DIR vs DFPS_WORKSPACE_ROOT).
    - [ ] `lib/platform/observability` (`dfps_observability`)
      - [ ] Make env loading fallible (no panic in `OBS_ENV`) and surface init errors to callers; add tests for missing env files.
      - [ ] Extend `PipelineMetrics` with structured analytics/cohort timing instead of ad-hoc counters.
    - [ ] `lib/platform/test_suite` (`dfps_test_suite`)
      - [ ] Remove the unsafe `set_var` in `TEST_SUITE_ENV`; inject DFPS_EVAL_DATA_ROOT via config/setup helpers instead.
      - [ ] Provide helpers for spinning up temporary SQLite datamart instances to share across API/CLI integration tests.
    - [ ] `lib/platform/vector_store` (`dfps_vector_store`)
      - [ ] Rework env parsing into a typed config builder using `dfps_configuration` (replace manual `env_flag`/parse) with per-backend unit tests.
      - [ ] Add backend health/index abstractions so unsupported backends (Milvus) fail fast and CLI/pipeline share indexing code paths.

### REFR-02 – Layer boundaries & dependency hygiene

- [ ] Enforce one-way dependencies (app → domain → platform) with an automated graph check (guppy/cargo metadata) and report exceptions.
- [ ] Document boundary rules per layer (app owns transport/adapters; domain owns business logic; platform owns infra/config) in `docs/system-design/base/directory-architecture.md` and link from crate READMEs.
- [ ] Add crate-level `//!` headers in each lib pointing to the exact system-design pages and kanban IDs governing its behavior.
- [ ] Introduce a “no env in domain” lint (deny `std::env` usage) for `lib/domain/**`; shift env lookup to app/platform configs.
- [ ] Verify no platform crate imports domain/app types (except shared primitives) and codify this as a CI check.
- [ ] Add a “dependency seams” doc mapping DTO ownership: FHIR/staging (dfps_core/dfps_ingestion), mapping (dfps_core/dfps_mapping), analytics (dfps_datamart/dfps_api), UI views (dfps_web_frontend).
  - [ ] Hex-port flow – lib/app (ports = HTTP/CLI; adapters = domain orchestration)
    - [ ] `lib/app/cli` — classify each bin: define hexagonal ports (commands) for ingestion/mapping/eval/vector-index and move IO/NDJSON parsing into adapters; replace direct domain calls with orchestrator traits in `dfps_pipeline`.
    - [ ] `lib/app/web/frontend` — treat reqwest client as outbound adapter; ensure routes/views depend only on frontend-facing ports (DTOs) and never on domain structs directly; document adapter boundary in `routes.rs`, `client.rs`.
    - [ ] `lib/app/web/backend/api` — expose inbound ports as axum handlers; push pipeline/datamart/compliance into injected application services; ensure `server.rs` only wires adapters (HTTP ↔ app services).
    - [ ] `lib/app/web/backend/datamart` — model DB as outbound adapter; keep fact/dim builders as domain mappers; surface a port trait (`DatamartSink`) consumed by API/CLI.
  - [ ] Hex-port flow – lib/domain (core hex core; ports = traits; adapters live in app/platform)
    - [ ] `lib/domain/core` — mark entities/value objects as core; add constructors/invariants; ensure zero IO/env.
    - [ ] `lib/domain/ingestion` — define trait ports for validation/profile lookup; keep transforms pure; move external validator adapter to app layer.
    - [ ] `lib/domain/mapping` — expose Mapper/Ranker/Terminology ports; remove env/policy loading; make vector store a port (trait) with adapters in platform.
    - [ ] `lib/domain/pipeline` — act as orchestrator port wiring ingestion/mapping; accept injected services/config; prohibit env/logger initialization.
    - [ ] `lib/domain/eval`/`fake_data`/`fhir_profiles`/`obo_graph`/`terminology` — classify as core data/providers; ensure any file IO/env is behind port traits (dataset provider, profile provider, ontology loader).
  - [ ] Hex-port flow – lib/platform (adapters & infra)
    - [ ] `lib/platform/configuration` — provide config-loading adapters; no domain coupling.
    - [ ] `lib/platform/observability` — treat logging/metrics as outbound adapter; expose trait(s) consumable by app/domain.
    - [ ] `lib/platform/vector_store` — pure adapter for vector backends implementing domain port; validate configs; document boundaries to mapping/pipeline/app.
    - [ ] `lib/platform/compliance` — policy loader as adapter; enforcement callable from app/domain ports without env.
    - [ ] `lib/platform/test_suite` — test-only adapters (datasets/db), no production env mutation.

### REFR-03 – Cross-surface contracts & DTO alignment

- [ ] Define canonical DTOs for analytics/eval/mapping in a shared module (mirrored between `dfps_api` responses, `dfps_web_frontend` client types, and `dfps_cli` outputs) and document their owners.
- [ ] Add regression tests that round-trip API responses (health/metrics/analytics/eval) through frontend client deserializers to prevent drift.
- [ ] Establish contract tests for mapping invariants: pipeline output → datamart loader → analytics summaries must preserve mapping state, license tier, and vector_usage metadata.
- [ ] Create a “compliance surface” checklist: every path emitting mapping results must show compliance mode, license-blocked counts, and export policy enforcement (CLI, API, datamart loader).
- [ ] Add versioned schema snapshots (JSON Schema or Rust type hashes) for public structs: `PipelineOutput`, `MappingResult`, `PipelineMetrics`, analytics summary/cohort rows.
- [ ] Document error taxonomies per surface (CLI exit codes, API status codes, frontend user messages) and map them back to domain errors (ingestion, mapping, compliance).

---



### REFR-09 – Domain pipeline orchestrator (`dfps_pipeline`)

**Goal:** Make `dfps_pipeline` the thin, explicit orchestrator connecting ingestion, mapping, and vector-store configuration, without owning transport, env, or platform concerns.

- [ ] Add a crate README and `//!` header describing:
  - [ ] The `bundle_to_mapped_sr` contract (input, output, error behavior).
  - [ ] How it composes `dfps_ingestion`, `dfps_mapping`, and vector-store traits.
- [ ] Review `try_vector_mapping`:
  - [ ] Identify all env/config reads (e.g., `VectorStoreConfig::from_env`) and plan to replace them with injected config/handles from app/platform layers.
  - [ ] Ensure Qdrant/Mock-specific details remain in platform/adapter crates where possible.
- [ ] Confirm that `PipelineOutput` only uses domain and platform DTOs that are stable across surfaces (CLI/API/web/datamart), and plan any schema adjustments needed to support docs and analytics.
- [ ] Add tests covering:
  - [ ] Pure lexical mapping path (no vector store).
  - [ ] Vector-enabled path using `VectorBackend::Mock`, asserting that `vector_usage` metadata is populated and stable.
  - [ ] Failure modes (invalid Bundle, ingestion error, disabled vector backend) with clear error mapping for upstream layers.

---

### REFR-10 – Platform configuration & env loading (`dfps_configuration`)

**Goal:** Make `dfps_configuration` the single entrypoint for environment/namespace/profile resolution across app, domain, and platform crates, with typed helpers instead of ad-hoc `std::env` and custom flag parsing.

- [ ] Add `lib/platform/configuration/README.md` summarizing:
  - [ ] How `load_env(namespace)` is intended to be used by `lib/app`, `lib/domain`, and `lib/platform`.
  - [ ] The search strategy (`DFPS_ENV_FILE`, `DFPS_ENV_DIR`, `DFPS_WORKSPACE_ROOT`, `data/environment`) and strict vs non-strict modes.
- [ ] Extract common helpers from `lib/platform/configuration/src/lib.rs` into reusable functions:
  - [ ] Typed flag parsing (bool/int/port) that can be called from vector store, compliance, observability, API/CLI configs.
  - [ ] A small “config root” helper that exposes the resolved workspace root and env search dirs to other crates.
- [ ] Inventory env usage in other crates (`dfps_vector_store`, `dfps_compliance`, `dfps_observability`, `dfps_test_suite`, apps) and:
  - [ ] Replace bespoke `env::var` + string parsing with typed helpers from `dfps_configuration`.
  - [ ] Ensure all env-namespace names reflect directory structure (`app.web.api`, `platform.vector_store`, etc.) and are documented.
- [ ] Add tests for:
  - [ ] `workspace_root()` discovery in nested directories and failure modes (`WorkspaceRootNotFound`).
  - [ ] Resolution order when `DFPS_ENV_FILE`, `DFPS_ENV_DIR`, and `DFPS_WORKSPACE_ROOT` are all present.
  - [ ] Strict vs non-strict behavior in CI and local dev (FileMissing vs silent no-op).

### REFR-11 – Platform compliance & export gating (`dfps_compliance`)

**Goal:** Provide a robust, testable compliance layer that expresses license/tier policies as pure data and config, with env loading and overrides clearly separated from domain mapping and datamart export.

- [ ] Add `lib/platform/compliance/README.md` that:
  - [ ] Explains `ComplianceMode`, `ComplianceAction`, `Policy`, and their relationship to mapping/export.
  - [ ] Documents env variables (`DFPS_COMPLIANCE_MODE`, `DFPS_COMPLIANCE_POLICY_PATH`, `DFPS_WORKSPACE_ROOT`) and example policies.
- [ ] Refactor `load_policy_from_env`:
  - [ ] Introduce a `ComplianceConfig` struct that encapsulates env-derived settings (mode, policy path, workspace root).
  - [ ] Use `dfps_configuration` helpers for env/paths instead of calling `env::var` directly.
  - [ ] Return structured errors without panicking and ensure message clarity for CI logs.
- [ ] Ensure `dfps_mapping`, `dfps_pipeline`, and `dfps_datamart` call `Policy::default_for_mode` or injected `Policy` instead of repeatedly calling `load_policy_from_env`:
  - [ ] Plan entrypoints where app/web/CLI wires a `Policy` into mapping/pipeline/warehouse surfaces.
  - [ ] Keep mapping domain code free of direct env/config parsing.
- [ ] Add tests covering:
  - [ ] Policy overrides from JSON/YAML across all modes and actions.
  - [ ] Export gating semantics for each license tier in `Internal`, `Partner`, and `OpenSource` modes.
  - [ ] Error behavior when policy files are malformed or missing, including `DFPS_WORKSPACE_ROOT` fallbacks.

### REFR-12 – Platform observability & metrics (`dfps_observability`)

**Goal:** Turn `dfps_observability` into a stable, opt-in metrics/logging adapter that can be wired from app/CLI/API, without panicking on env load and with clear contracts for pipeline/mapping/vector metrics.

- [ ] Replace the panicking `OBS_ENV` initializer:
  - [ ] Make `init_environment()` return a `Result<(), EnvLoadError>` that callers can handle, instead of `panic!`.
  - [ ] Use `dfps_configuration::load_env("platform.observability")` with clear error messages and tests.
- [ ] Clarify the ownership of `PipelineMetrics`:
  - [ ] Document which surfaces update which fields (pipeline, vector store, analytics, cohorts).
  - [ ] Ensure fields like `analytics_requests`, `cohort_queries`, `cohort_results_total` are updated only in app/API layers, not domain crates.
- [ ] Add helper functions for:
  - [ ] Emitting metrics snapshots as JSON structs that can be consumed by `/metrics/summary` endpoints and CLI exporters.
  - [ ] Mapping `VectorUsageSnapshot` and vector capacity proxies (`geom_rm_sqrt_dm`, `cap_alpha_sim`) into pipeline metrics consistently.
- [ ] Add tests that:
  - [ ] Confirm `log_pipeline_output` and `log_no_match` do not panic when env files are missing (non-strict mode).
  - [ ] Verify metrics counters (bundle_count, mapping_count, license_blocked, vector_* fields) match expectations for sample pipeline runs.
  - [ ] Cover behavior when vector capacity is missing vs present (capacity fields remain `None` vs set).

### REFR-13 – Platform vector backends (`dfps_vector_store`)

**Goal:** Provide a cohesive, well-tested vector backend abstraction where env/config parsing is centralized, backends are pluggable, and usage metrics/capacity proxies integrate cleanly with domain mapping and observability.

- [ ] Add `lib/platform/vector_store/README.md` describing:
  - [ ] Supported backends (Qdrant, PgVector, Milvus, Mock), their env variables, and deployment expectations.
  - [ ] The `VectorStore` trait, `VectorStoreConfig`, and how they are consumed by mapping and pipeline.
- [ ] Refactor `VectorStoreConfig::from_env`:
  - [ ] Use shared env helpers from `dfps_configuration` (bool/int/URL parsing) instead of local `env_flag` and `env::var` parsing.
  - [ ] Clearly separate config validation (`validate()`) from env parsing; document expected error messages in CI.
- [ ] Review backend implementations:
  - [ ] Ensure `QdrantVectorStore` and `PgVectorStore` handle collection/table creation idempotently and surface clear `IndexFailed`/`SearchFailed` errors.
  - [ ] Add timeouts and retry behavior consistent with `health_timeout_ms`.
  - [ ] Confirm `MockVectorStore` remains deterministic for tests and does not accidentally fetch real env config.
- [ ] Align capacity and usage reporting:
  - [ ] Guarantee that all backends can propagate `CapacityProxies` to callers when available and safely omit them when not.
  - [ ] Add tests for dimension mismatch, namespace validation, pool_max/timeout edge cases, and health checks across all supported backends.

### REFR-14 – Platform test harness & regression suite (`dfps_test_suite`)

**Goal:** Consolidate cross-crate testing concerns (fixtures, assertions, env bootstrapping) inside `dfps_test_suite` so that app/domain/platform tests use a single, well-defined harness without unsafe env mutations.

- [ ] Add `lib/platform/test_suite/README.md` that:
  - [ ] Explains the structure of `src/` (assertions, fixtures, regression) and `tests/` (e2e, integration, unit).
  - [ ] Describes how other crates should depend on this crate (for fixtures, not for production code).
- [ ] Refactor env handling in `TEST_SUITE_ENV`:
  - [ ] Avoid `unsafe` `set_var` by providing explicit setup helpers (e.g., `init_eval_data_root(workspace_root)`).
  - [ ] Use `dfps_configuration` to discover workspace root and env files for test namespaces instead of hard-coded ancestor traversal.
- [ ] Clarify fixture ownership:
  - [ ] Ensure all regression/eval fixtures live under `lib/domain/evaluation/eval/data/**` and are accessed via `dfps_eval::fake_data::fixtures::Registry` helpers.
  - [ ] Document how new datasets/fixtures should be added (naming, manifests, baseline summaries) so tests remain stable.
- [ ] Harden test surfaces:
  - [ ] Ensure that e2e/integration/unit test modules do not depend on internal APIs that are likely to change; prefer public ports (CLI/app services, pipeline, datamart, API endpoints).
  - [ ] Add a small “smoke test index” that verifies all major flows (ingestion, mapping, eval, vector, warehouse, web API) still run after refactors.
- [ ] Add CI guidance:
  - [ ] Document which features (`backend-pgvector`, external validation mocks) must be enabled for the full suite.
  - [ ] Provide recommended command lines (`cargo test -p dfps_test_suite --features backend-pgvector`) to reproduce CI locally.

---

### REFR-15 – CLI surfaces & orchestration (`dfps_cli`)

**Goal:** Treat `dfps_cli` as a thin orchestration layer over domain + platform crates, with shared IO/config/compliance handling and consistent UX across all binaries.

- [ ] Add/expand `lib/app/cli/README.md` to:
  - [ ] Map each bin (`map_bundles`, `map_codes`, `eval_mapping`, `validate_fhir`, `load_datamart`, `build_vector_index`) to its underlying domain flows (ingestion, mapping, eval, datamart load, vector index).
  - [ ] Document common flags (env namespace, log level, compliance behavior) and how they relate to API/web behavior.
- [ ] Introduce a small internal “CLI core” module (e.g., `src/cli_core.rs`) that:
  - [ ] Provides shared helpers for env loading (`load_env("app.cli")`), log initialization, and structured error reporting.
  - [ ] Wraps repeated NDJSON reading/writing logic (streaming readers for Bundles, StgSrCodeExploded, PipelineOutput, EvalCase).
  - [ ] Centralizes exit code conventions (e.g., non-zero on compliance block, threshold failure, invalid input).
- [ ] `map_bundles`:
  - [ ] Replace inline env/logging setup with shared CLI core helpers and ensure validation + pipeline + metrics calls are consistently structured.
  - [ ] Add a streaming path (reading Bundles incrementally) that still accumulates a single `PipelineMetrics` summary.
  - [ ] Align JSON output schema (`kind` field) with API responses for easier downstream parsing.
- [ ] `map_codes`:
  - [ ] Share vector config handling with `build_vector_index` (via a small helper that wraps `VectorStoreConfig::from_env` and backend selection).
  - [ ] Move compliance reporting and `fail_on_license_block` behavior behind a single helper used by both CLI and API surfaces.
  - [ ] Ensure explanation output (`--explain`, `--explain-top`) is documented and stable for downstream tooling.
- [ ] `eval_mapping`:
  - [ ] Factor dataset loading/reporting into reusable helpers that mirror API eval endpoints (dataset list, summary, run).
  - [ ] Clarify top-k semantics (“placeholder until multi-candidate support”) and future-proof the flag by plumbing top-k into `dfps_mapping` when available.
  - [ ] Ensure threshold/compare/deterministic checks share code with API eval (minimize duplicated logic).
- [ ] `validate_fhir`:
  - [ ] Align mode flags (`lenient`, `strict`, `external_preferred`, `external_strict`) with ingestion docs and API options.
  - [ ] Provide a stable NDJSON output schema for issues and summaries that can be consumed by CI dashboards and dfps_web_frontend.
- [ ] `load_datamart`:
  - [ ] Remove duplicated export-policy logic by delegating to a shared helper that is also used in `dfps_api` and `dfps_datamart`.
  - [ ] Ensure `WarehouseConfig::from_env` uses `dfps_configuration` for env parsing and that CLI errors surface actionable messages for missing URL/schema/permissions.
- [ ] `build_vector_index`:
  - [ ] Refactor panicking paths (dimension overflows, unsupported backends) into structured CLI errors.
  - [ ] Share embedding/version metadata semantics with mapping/vector-store docs (documented in mdBook and CLI help).

### REFR-16 – HTTP backend & warehouse surfaces (`dfps_api`, `dfps_datamart`)

**Goal:** Treat the Axum API and datamart service as the primary HTTP-facing ports into the domain/pipeline/datamart, with clean config injection, compliance boundaries, and DTOs aligned across app/frontend/docs.

- [ ] `dfps_api`:
  - [ ] Add a crate-level README that describes:
    - [ ] The main routes (`/api/map-bundles`, `/metrics/summary`, `/health`, `/analytics/*`, `/api/eval/*`).
    - [ ] How it composes `dfps_pipeline`, `dfps_datamart`, `dfps_observability`, `dfps_eval`, `dfps_compliance`, `dfps_terminology`.
  - [ ] Replace raw `env::var` usage in `ApiServerConfig::default` with a typed config struct built via `dfps_configuration` (host, port, warehouse URL, etc.).
  - [ ] Factor compliance enforcement into one helper (shared with CLI/datamart) so license-tier export rules are defined once.
  - [ ] Ensure `ApiState` is constructed with injected `Policy`, `WarehouseConfig`, and `PipelineMetrics`, rather than loading env/policy deep inside handlers.
  - [ ] Review error taxonomy:
    - [ ] Keep `ApiError` variants stable and documented (`invalid_json`, `invalid_fhir`, `invalid_dataset`, `compliance_blocked`, `internal_error`).
    - [ ] Align HTTP status codes and error payloads with frontend expectations and CLI error messages.
- [ ] `dfps_datamart`:
  - [ ] Document how `Dim*` and `FactServiceRequest` types map onto the SQL schema and how they’re consumed by analytics/cohort endpoints.
  - [ ] Move `WarehouseConfig::from_env` onto `app.web.backend.datamart` namespace and use `dfps_configuration` helpers for parsing numeric params.
  - [ ] Ensure `load_from_pipeline_output` uses the same compliance helper as CLI/API (no duplicated `parse_license_tier` logic).
  - [ ] Add tests for:
    - [ ] Idempotent upserts (dim tables) and stable keys across re-runs.
    - [ ] NO_MATCH sentinel behavior and referential integrity with fact rows.

### REFR-17 – Web frontend & UX (`dfps_web_frontend`)

**Goal:** Make `dfps_web_frontend` a thin, well-typed UI shell over `dfps_api`, with configuration driven by `dfps_configuration`, clean DTO mapping, and views that reflect the system-design/analytics docs.

- [ ] Add a top-level README section that:
  - [ ] Maps each page/route (`/`, `/analytics`, `/eval`, `/docs`) to the backend endpoints it calls.
  - [ ] Explains env variables (`DFPS_FRONTEND_LISTEN_ADDR`, `DFPS_API_BASE_URL`, `DFPS_API_CLIENT_TIMEOUT_SECS`, `DFPS_DOCS_URL`) and how they relate to configuration docs.
- [ ] `config.rs`:
  - [ ] Replace direct `env::var` parsing with helpers from `dfps_configuration` (bool/int/string) so error handling and defaults are consistent.
  - [ ] Ensure all config errors are surfaced as structured IO errors in `run()`, not panics.
- [ ] `client.rs`:
  - [ ] Confirm DTOs (`MapBundlesResponse`, `EvalRunResponse`, `AnalyticsSummaryResponse`, `CohortResponse`) mirror backend DTOs; add tests that deserialize API responses from `dfps_api` test fixtures.
  - [ ] Normalize error reporting (`ClientError`) for use in views (short, user-friendly messages).
  - [ ] Ensure timeouts and base URLs are fully driven by `AppConfig`.
- [ ] `routes.rs` + `views.rs` + `view_model.rs`:
  - [ ] Document how HTMX fragments map to API calls and eval/analytics flows.
  - [ ] Audit Bundle upload/paste flows for size limits, error messaging, and alignment with CLI behavior (e.g., “no bundles found” vs “invalid JSON”).
  - [ ] Ensure eval/analytics panels clearly surface compliance mode, mapping states, and NoMatch reasons consistent with docs.
  - [ ] Clarify how default eval dataset (`DEFAULT_EVAL_DATASET`) is chosen and keep it in sync with doc examples.
- [ ] Cross-surface DTO alignment:
  - [ ] Verify that `MappingResultsView`, `AnalyticsSummaryView`, `CohortView`, and eval panels stay in sync with `dfps_api` DTOs and CLI output (no divergent field names).
  - [ ] Add snapshot tests (string-based) for key HTML fragments (mapping results, NoMatch explorer, analytics panels, eval panel) to guard against breaking UI contracts used in screenshots/docs.

---

## INPROGRESS
- _Empty_

---

## REVIEW

### REFR-04 – Domain core & staging (`dfps_core`)

**Goal:** Make `dfps_core` the canonical source of truth for domain entities, IDs, staging rows, and mapping result types, with no IO/env/platform coupling.

- [x] Add `lib/domain/core/README.md` that:
  - [x] Explains the split between `encounter`, `order`, `patient`, `staging`, `mapping`, `value`, and `fhir` modules.
  - [x] Links each module back to the relevant system-design diagrams (FHIR class/model, NCIt architecture, staging tables).
- [x] Restructure `dfps_core` into super-domains (`primitives`, `clinical`, `interop`, `semantics`) with bridges placed under the consumer (`bridge/`) modules.
- [x] Preserve legacy public API paths via `lib.rs` re-exports and `prelude` wiring so `dfps_core::{patient, order, staging, mapping, value, fhir}` continue to work.
- [ ] Review `dfps_core::mapping` types (`CodeElement`, `MappingResult`, `MappingThresholds`, `MappingSourceVersion`, `NCItConcept`, `DimNCITConcept`) and:
  - [x] Identify any duplicate mapping/result structs in other crates and plan to unify them on `dfps_core::mapping`.
  - [x] Add helper constructors/builders for common result patterns (AutoMapped / NeedsReview / NoMatch).
- [ ] Confirm staging structs (`StgServiceRequestFlat`, `StgSrCodeExploded`) match:
  - [x] Ingestion transforms (dfps_ingestion),
  - [x] Pipeline outputs (dfps_pipeline),
  - [x] Datamart schema (dim/fact tables), and document any intentional differences.
- [ ] Ensure `dfps_core` has:
  - [x] No direct `std::env` or file IO,
  - [x] Only serialization and optional `fake`/`Dummy` derives as dependencies.
- [x] Add doc-tests or small unit tests in core modules (value/order/encounter/patient/staging) that mirror the canonical ServiceRequest journey.

### REFR-05 – Domain ingestion & FHIR profiles (`dfps_ingestion`)

**Goal:** Keep ingestion/FHIR-profile logic as a pure domain service that transforms Bundles into staging/domain types and structured validation results, leaving HTTP/env concerns to app/platform layers.

- [x] `dfps_ingestion`
  - [x] Document the ingestion flow from `fhir::Bundle` → `StgServiceRequestFlat` / `StgSrCodeExploded` → `order::ServiceRequest` in a crate-level `//!` header and README.
  - [x] Review `IngestionError` and `ValidationIssue`/`ValidationReport` usage and:
    - [x] Ensure error types don’t encode HTTP/env assumptions.
    - [x] Clarify when validation failures vs decode errors vs external validator failures are returned.
  - [ ] Separate pure transforms (`sr_to_staging`, `sr_to_domain`) from validation orchestration (`bundle_to_*_with_validation`) so callers can compose them independently.
  - [x] Introduce a small, testable trait/port for external validation so app/platform layers can own HTTP clients.
  - [x] Embed FHIR profiles under `profiles/`, document include paths, and test cardinalities/known URLs.
  - [ ] Add a validated-bundle type to carry `ValidationReport` without double validation.
  - [ ] Return typed status/intent enums from parsing helpers instead of strings to reduce downstream re-parsing.

### REFR-06 – Domain mapping engine & NCIt integration (`dfps_mapping`)

**Goal:** Keep the mapping engine deterministic and domain-centric, while factoring out policy/vector/terminology wiring so platform/app layers can compose backends cleanly.

- [ ] Document the mapping pipeline in a crate-level `//!` header and README:
  - [x] Clarify roles of `Mapper`, `CandidateRanker`, `MappingEngine`, and policy/vector/terminology integration points.
  - [x] Link to NCIt/obo-graph system-design docs and evaluation epics (mapping/eval harness).
- [ ] Identify places where `dfps_mapping` directly reads env or config (e.g., `load_policy_from_env`):
  - [x] Introduce an explicit `MappingConfig` / policy parameter so mapping functions can be called without reading env.
  - [x] Plan to move env parsing for policies into `dfps_compliance` / `dfps_configuration` (`ComplianceConfig::from_env` now encapsulates `DFPS_COMPLIANCE_*` reads and feeds policies into mapping via injected config/policy).
  - [x] Wire CLI (`map_codes`, `map_bundles`, `load_datamart`), API (`dfps_api`), and datamart loaders to construct `ComplianceConfig` once at startup and pass the resulting policy through mapping/pipeline/datamart paths instead of calling `load_policy_from_env` repeatedly.
- [ ] Review vector-related wiring:
  - [x] Ensure `VectorRankerBackend` and `DeterministicEmbeddingProvider` are pure domain constructs that operate purely on traits (`VectorStore`, `EmbeddingProvider`).
  - [x] Avoid coupling mapping to specific backends (Qdrant/pgvector) beyond the trait layer.
- [ ] Align mapping result semantics:
  - [x] Confirm `build_result_with_score` uses a single source of truth for thresholds and `MappingState` transitions.
  - [x] Add refactor tasks for reusing `MappingThresholds`/`MappingSourceVersion` from `dfps_core` consistently.
- [x] Evaluate whether deprecated `eval::run_eval` can be removed or wrapped behind a clearer “mapping eval port” that just delegates to `dfps_eval` (`dfps_mapping::eval` shim deleted; callers use `dfps_eval::run_eval_with_mapper` directly).

### REFR-07 – Domain eval harness & fake data fixtures (`dfps_eval`, `dfps_eval::fake_data`)

**Goal:** Treat eval and fake-data crates as domain-aligned data/eval providers with deterministic behavior and clear boundaries to IO/config, ready to be driven by CLI/web/platform tooling.

- [x] `dfps_eval`
  - [x] Add a crate README explaining the role of EvalCase/EvalSummary, dataset manifests, and baseline snapshots.
  - [x] Separate dataset discovery/loading concerns from scoring/aggregation:
    - [x] Keep file/NDJSON IO behind small helpers that can later be replaced with alternative sources (e.g., HTTP, DB).
    - [x] Clarify how `DEFAULT_DATA_ROOT` and `DFPS_EVAL_DATA_ROOT` interact with future configuration layers.
  - [x] Identify any places where eval depends on CLI/web/platform behavior (env, logging) and plan to push those concerns outward.
- [x] `dfps_eval::fake_data`
  - [x] Document the structure of `data/` (eval/meta/regression) and how it maps to `fixtures::*` and generator modules.
  - [x] Centralize RNG seeding and ID generation helpers so CLI bins and tests share deterministic behavior.
  - [x] Ensure value generators (`fake_*_id`, `fake_order_description`, etc.) are thin over core types from `dfps_core` and do not embed app/platform assumptions (URLs, ports, etc.).
  - [x] Cross-check fixture schemas with `dfps_eval::EvalCase` and ingestion/mapping expectations; add small tests that load each tier (bronze/silver/gold) to catch drift.

### REFR-08 – Domain terminology & ontology graph (`dfps_terminology`)

**Goal:** Provide a clean, domain-level terminology layer (code systems, value sets, OBO graphs) that exposes stable APIs for mapping/compliance, with HTTP/env responsibilities clearly separated.

- [x] `dfps_terminology`
  - [x] Add a crate README that explains:
    - [x] The split between `codesystem`, `valueset`, `bridge`, `client`, `obo`, and `registry`.
    - [x] How license tiers and source kinds are intended to feed mapping/compliance decisions.
  - [x] Confirm `canonicalize_system` and `CodeKind` semantics are used consistently by both ingestion and mapping; plan to expose any missing helpers as public API.
  - [x] Inventory all HTTP/env touchpoints in `client::TerminologyClientConfig` and friends; mark them as adapter seams for future platform layering.
- [x] `dfps_terminology::obo_graph`
  - [x] Document the expectations around embedded `.obo` minis vs potential full graphs (NCIT/MONDO) and how graph versions are surfaced.
  - [x] Ensure `CachedOntologyGraph` exposes only pure graph operations (ancestors/descendants/synonyms/related_concepts), leaving file/network IO out of this crate.
  - [x] Add tests or examples tying synonym/related-concept behavior back to mapping/terminology use cases (e.g., PET/CT synonym sets).
  
---

## DONE
- _Empty_

---
