# Kanban - feature/codebase-refactor (024)

**Theme:** Docs polish - public hosting, full-text search, theming  
**Branch:** `feature/meta/REFR-024-codebase-refactor`  
**Goal:** Turn the local mdBook into a searchable, themed, publicly hosted documentation site, integrated with `/docs` in the frontend.

> Status: **INPROGRESS**  
> Branch target version: `v0.1.0`  
> Introduced in: `v0.1.0`  
> Last updated in: `v0.1.0`

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
    - [ ] `lib/app/frontend/cli` (`refractive_swan_cli`)
      - [x] Extract shared NDJSON streaming/logging/compliance helpers (used in map_bundles/map_codes/validate_fhir/load_datamart) into an internal module to cut duplication.
      - [x] Avoid double-validation in `map_bundles` by plumbing ingestion validation data from `bundle_to_mapped_sr` instead of re-running `validate_bundle`.
      - [x] Add a streaming mapping path in `map_codes` (no upfront Vec) that still propagates vector usage metrics and compliance failures.
      - [x] Centralize CLI config/logging setup via `refractive_swan_configuration` (namespace `app.cli`) so each bin stops initializing env_logger/env parsing manually.
    - [ ] `lib/app/web`
      - [x] Add app-level README describing the split between frontend (Actix) and backend (Axum) and where shared DTOs/configs live. (`lib/app/web/README.md` now documents the split + env namespaces.)
      - [x] Introduce a shared web DTO module so frontend client types, backend API responses, and analytics structs do not diverge. (`refractive_swan_web_dto` crate re-exports the canonical contracts.)
      - [ ] `lib/app/frontend/web` (`refractive_swan_web_frontend`)
        - [x] Replace manual env parsing in `config.rs` with a typed config sourced from `refractive_swan_configuration` (timeout validation, docs URL normalization). (Already wired via `AppConfig::from_env()`.)
        - [x] Centralize backend client error handling/logging (currently inline in routes) and emit analytics request metrics via `refractive_swan_observability`. (Backend client now logs via `log`, and `AppState` tracks analytics metrics with `PipelineMetrics`.)
        - [x] Add regression test for `/analytics` error handling (backend 5xx/timeout) to keep user-facing messages stable. (New Wiremock test covers the error path.)
      - [ ] `lib/app/servers` (`refractive_swan_web_backend`)
        - [x] Document ownership/boundaries for API vs datamart crates and the expected env namespaces for each.
        - [x] Hoist shared analytics DTOs into a reusable module so API handlers and frontend client structs stay in sync. (`refractive_swan_web_dto` is now consumed by both API and frontend client.)
        - [ ] `lib/app/servers/api` (`refractive_swan_dataplane`{formerly `refractive_swan_api`})
          - [x] Add a config module (using `refractive_swan_configuration`) for host/port/warehouse/compliance instead of scattered `std::env::var` lookups in `server.rs`.
          - [x] Extract analytics persistence/state management into a component with eviction/metrics to avoid unbounded HashMap growth.
          - [x] Load compliance policy once at startup and thread it through handlers instead of calling `load_policy_from_env` per request.
        - [ ] `lib/app/servers/datamart` (`refractive_swan_datamart`)
          - [x] Refactor `WarehouseConfig::from_env` to use `refractive_swan_configuration` validation (url/schema/pool) and cover it with unit tests.
          - [x] Add idempotent migration/load tests for NO_MATCH handling, duplicate SR rows, and compliance export filtering.
          - [x] Provide a streaming insert API so CLI/API callers don’t buffer full PipelineOutput lists before writing to SQLite.
  - [ ] `lib/domain`
    - [x] Document domain layering (core/ingestion/mapping/eval/terminology) and keep domain crates free of direct environment reads. (Directory architecture doc now calls out the layering + env-free constraints.)
    - [x] Align error/reporting semantics across domain crates (ValidationError/Policy errors) to simplify pipeline/app boundaries. (`EvalMetrics` splits results vs metrics; dataset/config errors now use the shared `thiserror` pattern.)
    - [ ] `lib/domain/core` (`refractive_swan_core`)
      - [x] Add a `MappingResult` builder/constructor to standardize reason/state handling instead of duplicating logic in downstream crates. (Documented in `README.md`; builders already in `result.rs`.)
      - [x] Expose a helper for constructing stable `CodeElement` IDs so ingestion/mapping/datamart share the same format. (`CodeElement::id_for` documented + referenced by README.)
      - [x] Add a crate README tying modules to system-design docs and clarifying staging vs mapping vs order invariants.
    - [ ] `lib/domain/meta/evaluation` (`refractive_swan_eval`)
      - [x] Replace raw `std::env` dataset root resolution with a config struct built via `refractive_swan_configuration` (still honoring refractive_swan_EVAL_DATA_ROOT).
      - [x] Split IO/parsing from scoring so eval functions accept injected readers/writers instead of reading files directly. (`run_eval_streaming[_metrics]_with_mapper` accept `BufRead`; summary/metrics aggregation separated.)
      - [x] Add determinism/benchmark tests for fingerprint computation across chunk sizes and `top_k` settings. (New fingerprint bench + streaming tests verifying chunk-size parity.)
    - [ ] `lib/domain/meta/evaluation::fake_data` (`refractive_swan_eval::fake_data`)
      - [x] Add module docs describing generator outputs/seed controls and link them from `data/eval/README.md`.
      - [x] Centralize RNG seeding helpers to keep fixtures deterministic across modules and CLI bins.
      - [x] Provide a thin config wrapper over `refractive_swan_configuration` for the generators instead of ad-hoc env access. (`FakeDataConfig` now wraps `refractive_swan_FAKE_DATA_*`.)
    - [ ] `lib/domain/ingestion` (`refractive_swan_ingestion`)
      - [x] Embed FHIR profiles under `refractive_swan_ingestion::profiles` with tests for required snapshot elements and documented URLs.
      - [x] Add a crate README and crate-level docs describing the Bundle → staging/domain flow and profile hook.
      - [x] Extract a validator port (`ExternalValidator` + `ExternalValidationContext`) so app/platform layers own HTTP clients; keep transforms/profile loading pure.
      - [x] Introduce a validated-bundle type that carries the `ValidationReport` to callers to avoid double validation in CLI/pipeline. (`ValidatedBundle::try_new` + pipeline entrypoints reuse the cached report.)
      - [x] Return typed status/intent enums from parsing helpers (instead of strings) to reduce downstream re-parsing. (`StgServiceRequestFlat` now carries `ServiceRequestStatus/Intent`; serde derives fall back for legacy payloads.)
    - [ ] `lib/domain/mapping` (`refractive_swan_mapping`)
      - [x] Remove implicit env access (`load_policy_from_env`) from mapping functions; require Policy/config injection from the app layer. (Confirmed MappingConfig handles policy injection; crate stays env-free.)
      - [x] Decouple vector-store wiring from the core engine (accept `VectorStoreConfig`/store from callers) and keep a pure deterministic constructor for tests. (Vector pipelines/engines accept injected stores/configs; `DefaultPipeline` now exposes validated-bundle path.)
      - [x] Add deterministic tests that pin `MappingThresholds` + reason strings used in `build_result_with_score`. (New helper tests cover threshold classification + default reasons.)
    - [ ] Terminology `obo_graph` module (`lib/domain/ontologies/terminology`)
      - [x] Document supported graph inputs and add a runtime loader for `.obo` paths (not just embedded minis). (`load_ontology_graph_from_path` + docs spell out runtime sources.)
      - [x] Add an integration test exercising `CachedOntologyGraph` caching/eviction with larger sample graphs. (Synthetic graph test drives the bounded caches; runtime loader fixture covered.)
    - [ ] `lib/domain/meta/pipeline` (`refractive_swan_pipeline`)
      - [x] Stop reading `VectorStoreConfig` from env inside `bundle_to_mapped_sr`; require injected config/store and surface errors instead of silent fallback. (Vector context is injected; `PipelineExecution` now drives reuse + validated bundles.)
      - [x] Emit refractive_swan_observability metrics/logging for ingestion + vector paths so downstream apps don’t re-count manually. (`PipelineExecution.metrics` + `log_pipeline_output_with_summary` expose per-run metrics without re-counting; CLI/API switched to the new flow.)
      - [x] Add tests for vector-enabled vs offline paths to keep `vector_usage` semantics stable. (Pipeline tests cover lexical/vector/no-context flows + validated bundle parity.)
    - [ ] `lib/domain/ontologies/terminology` (`refractive_swan_terminology`)
      - [x] Expose code-system normalization helpers (`canonicalize_system`) for reuse by ingestion/mapping to avoid drift. (`bridge::canonicalize_system` now exported + tests; README documents usage.)
      - [x] Add a crate README documenting license/source metadata semantics expected by analytics/mapping. (README expanded with canonicalization + license/source guidance.)
  - [ ] `lib/platform`
    - [x] Add a short platform README clarifying when to add new platform crates vs domain/app modules and the env namespaces to use. (`lib/platform/README.md` enumerates namespaces + ownership.)
    - [x] Deduplicate env-flag/number parsing helpers across platform crates (observability/vector_store/compliance/test_suite). (`refractive_swan_vector_store::config_from_env` now uses `refractive_swan_configuration` helpers for bools/ints/strings.)
    - [x] `lib/platform/compliance` (`refractive_swan_compliance`)
      - [x] Move env parsing into a dedicated config builder (refractive_swan_COMPLIANCE_*) with structured errors instead of panicking in `load_policy_from_env`. (`ComplianceConfig::from_env` + policy overrides already handle this; noted here for closure.)
      - [x] Add tests for policy override file parsing and `assert_export_allowed` behavior across modes. (Existing tests under `lib.rs` cover overrides + export gating; documented here.)
    - [x] `lib/platform/configuration` (`refractive_swan_configuration`)
      - [x] Publish shared env parsing helpers (bool/int/port) to replace bespoke logic in web frontend/API/vector store configs. (`values.rs` exports the helpers; README + docs reference them.)
      - [x] Add tests for workspace root resolution and env search ordering (refractive_swan_ENV_DIR vs refractive_swan_WORKSPACE_ROOT). (New tests in `src/tests.rs` verify overrides + search order.)
    - [x] `lib/platform/observability` (`refractive_swan_observability`)
      - [x] Make env loading fallible (no panic in `OBS_ENV`) and surface init errors to callers; add tests for missing env files. (Existing `env.rs` returns `Result` + tests; tracked here.)
      - [x] Extend `PipelineMetrics` with structured analytics/cohort timing instead of ad-hoc counters. (`metrics.rs` already exposes the structured fields.)
    - [x] `lib/platform/test_suite` (`refractive_swan_test_suite`)
      - [x] Remove the unsafe `set_var` in `TEST_SUITE_ENV`; inject refractive_swan_EVAL_DATA_ROOT via config/setup helpers instead. (`ensure_eval_data_root` now returns the path without mutating env; callers pass it via `Command::env`/`ScopedEnvVar`.)
      - [x] Provide helpers for spinning up temporary SQLite datamart instances to share across API/CLI integration tests. (`TempSqliteWarehouse` lives in `src/datamart.rs`; CLI tests now use it.)
    - [x] `lib/app/servers/vector_store` (`refractive_swan_vector_store`)
      - [x] Rework env parsing into a typed config builder using `refractive_swan_configuration` (replace manual `env_flag`/parse) with per-backend unit tests. (`config_from_env` now uses the shared helpers and existing tests cover the cases.)
      - [x] Add backend health/index abstractions so unsupported backends (Milvus) fail fast and CLI/pipeline share indexing code paths. (VectorStore trait + Qdrant/pg backends already enforce this; documenting here.)

---

## INPROGRESS
- _Empty_

---

## REVIEW

### REFR-02 – Layer boundaries & dependency hygiene

- [x] Enforce one-way dependencies (app → domain → platform) with an automated graph check (guppy/cargo metadata) and report exceptions (`cargo make layers-check` via `tools/layer_lint`; currently allowlists `refractive_swan_compliance`/`refractive_swan_observability` edges until their adapters move).
- [x] Document boundary rules per layer (app owns transport/adapters; domain owns business logic; platform owns infra/config) in `docs/system-design/base/directory-architecture.md` and link from crate READMEs.
- [ ] Add crate-level `//!` headers in each lib pointing to the exact system-design pages and kanban IDs governing its behavior.
- [x] Introduce a “no env in domain” lint (deny `std::env` usage) for `lib/domain/**`; shift env lookup to app/platform configs (`TerminologyClientConfig` + external validator env seams now live in app/platform adapters; enforced via `tools/layer_lint`).
- [x] Verify no platform crate imports domain/app types (except shared primitives) and codify this as a CI check (same `layers-check` task).
- [x] Add a “dependency seams” doc mapping DTO ownership: FHIR/staging (refractive_swan_core/refractive_swan_ingestion), mapping (refractive_swan_core/refractive_swan_mapping), analytics (refractive_swan_datamart/refractive_swan_api), UI views (refractive_swan_web_frontend) (`docs/system-design/base/dependency-seams.md`).
  - [ ] Hex-port flow – lib/app (ports = HTTP/CLI; adapters = domain orchestration)
    - [x] `lib/app/frontend/cli` — classify each bin: define hexagonal ports (commands) for ingestion/mapping/eval/vector-index and move IO/NDJSON parsing into adapters; replace direct domain calls with orchestrator traits in `refractive_swan_pipeline`. (PipelinePort added; CLI `map_bundles` now drives the port and shared README updated.)
    - [x] `lib/app/frontend/web` — treat reqwest client as outbound adapter; ensure routes/views depend only on frontend-facing ports (DTOs) and never on domain structs directly; document adapter boundary in `routes.rs`, `client.rs`. (Client + handlers now import contracts from `refractive_swan_contracts`, module docs describe the adapter boundary, and tests enforce NDJSON/HTML fragments deserialize from shared DTOs.)
    - [x] `lib/app/servers/api` — expose inbound ports as axum handlers; push pipeline/datamart/compliance into injected application services; ensure `server.rs` only wires adapters (HTTP ↔ app services). (NodeDataPlane now injects `PipelinePort` + `SqliteDatamart` (`DatamartSink`), handlers translate HTTP ↔ refractive_swan_contracts DTOs, and README documents the adapter boundary.)
    - [x] `lib/app/servers/datamart` — model DB as outbound adapter; keep fact/dim builders as domain mappers; surface a port trait (`DatamartSink`) consumed by API/CLI. (`DatamartSink` trait + `SqliteDatamart` implementation added; README documents env + adapter semantics.)
  - [ ] Hex-port flow – lib/domain (core hex core; ports = traits; adapters live in app/platform)
    - [x] `lib/domain/core` — mark entities/value objects as core; add constructors/invariants; ensure zero IO/env. (ID newtypes now validate non-empty strings + expose helpers; ServiceRequest construction uses the typed IDs everywhere.)
    - [x] `lib/domain/ingestion` — define trait ports for validation/profile lookup; keep transforms pure; move external validator adapter to app layer. (`ExternalValidator` is an explicit `Send + Sync` port and transforms rely solely on injected IDs/traits, no env access.)
    - [x] `lib/domain/mapping` — expose Mapper/Ranker/Terminology ports; remove env/policy loading; make vector store a port (trait) with adapters in platform. (`refractive_swan_vector_port` now hosts the vector traits; mapping/pipeline crates depend only on that port.)
    - [x] `lib/domain/meta/pipeline` — act as orchestrator port wiring ingestion/mapping; accept injected services/config; prohibit env/logger initialization. (Already using `PipelinePort` + injected vector contexts, no env/log init.)
    - [x] `lib/domain/meta/evaluation`/`fake_data`/`fhir_profiles`/`obo_graph`/`terminology` — classify as core data/providers; ensure any file IO/env is behind port traits (dataset provider, profile provider, ontology loader). (`refractive_swan_eval::DatasetStore` trait introduced; API/web/CLI now accept `Arc<dyn DatasetStore>` instead of concrete file IO.)
  - [ ] Hex-port flow – lib/platform (adapters & infra)
    - [ ] `lib/platform/configuration` — provide config-loading adapters; no domain coupling.
    - [x] `lib/platform/observability` — treat logging/metrics as outbound adapter; expose trait(s) consumable by app/domain. (Metrics/logging now consume `refractive_swan_vector_port` snapshots instead of defining their own schema.)
    - [x] `lib/app/servers/vector_store` — pure adapter for vector backends implementing domain port; validate configs; document boundaries to mapping/pipeline/app. (`config_from_env` lives here; crate re-exports the `refractive_swan_vector_port` traits and only implements adapters.)
    - [ ] `lib/platform/compliance` — policy loader as adapter; enforcement callable from app/domain ports without env.
    - [ ] `lib/platform/test_suite` — test-only adapters (datasets/db), no production env mutation.

### REFR-03 – Cross-surface contracts & DTO alignment

**Epic ID**: `REFR-03`
**Theme**: canonical DTOs & schemas across CLI / HTTP / datamart / eval / analytics
**Touches**:

* Domain:

  * `lib/domain/core` (`refractive_swan_core`) – `MappingResult`, `DimNCITConcept`, `Stg*` types.
  * `lib/domain/meta/pipeline` (`refractive_swan_pipeline`) – `PipelineOutput`.
  * `lib/domain/meta/evaluation` (`refractive_swan_eval`) – `EvalCase`, `EvalSummary`, `DatasetManifest`, `FileDatasetStore`.
* App:

  * `lib/app/servers/api/src/dto.rs` – `AnalyticsSummaryResponse`, `CohortResponse`, `EvalRunResponse`.
  * `lib/app/servers/api/src/server.rs` – `MapBundlesResponse`, `ErrorResponse`, `ApiError`.
  * `lib/app/servers/datamart` – dim/fact structs + SQL rows.
  * (later) `lib/app/frontend/cli`, `lib/app/frontend/web`.
* Platform (indirect):

  * `refractive_swan_configuration`, `refractive_swan_compliance`, `refractive_swan_observability` – referenced in error/metrics/compliance DTOs.

#### Goal

Define **one canonical set of contracts** (“Pipeline/Analytics/Eval contracts”) that:

1. Travel unchanged across:

   * `refractive_swan_api` HTTP responses,
   * CLI NDJSON outputs,
   * datamart/warehouse schemas,
   * eval fixtures,
   * web UI clients.
2. Are **schema-versioned** and **CI-guarded** against accidental breaking changes.
3. Encode a **unified error taxonomy** that ties `ApiError` + CLI exit codes back to domain/platform error categories.

#### Current problems in code

* DTO duplication / drift risk:

  * HTTP analytics DTOs live in `lib/app/servers/api/src/dto.rs`.
  * Datamart structs (`DimPatient`, `DimNCIT`, `FactServiceRequest`) live in `refractive_swan_datamart`.
  * Domain structs (`MappingResult`, `DimNCITConcept`, staging rows, `PipelineOutput`) live in `refractive_swan_core`/`refractive_swan_pipeline`.
  * Eval manifests and summaries live only in `refractive_swan_eval`.
* No explicit **contract crate**; nothing stops Web/CLI from inventing slightly divergent shapes.
* Error taxonomy is implicit:

  * `ApiError` categories exist but not mapped to a documented taxonomy.
  * Datamart’s `LoadError` and vector errors (`VectorStoreError`) are not unified.

#### Design

##### 03.1 Canonical contracts crate

Create a new crate (name exact paths up to you, but conceptually):

* `lib/domain/contracts` (package: `refractive_swan_contracts`).

**Ownership rules:**

* **Domain-level contracts** (no HTTP-specific wrapping, no CLI-only fields) live here:

  * `PipelineOutput` *re-export* from `refractive_swan_pipeline` (or a newtype alias if needed).
  * `MappingResult`, `DimNCITConcept` *re-export* from `refractive_swan_core`.
  * Analytics contracts:

    * `AnalyticsSummaryRow` (structurally equivalent to `AnalyticsNcitSummaryRow`).
    * `AnalyticsSummaryResponse`.
    * `CohortRow`, `CohortResponse`.
  * Eval contracts:

    * `DatasetManifest` (re-export from `refractive_swan_eval`).
    * `EvalSummary`, `EvalRunResponse`.
  * Metrics & vector usage snapshots:

    * `PipelineMetrics` (re-export from `refractive_swan_observability`) as a “public metrics contract”.
    * `VectorUsageSnapshot`, `VectorCapacitySnapshot`.
* **Transport wrappers** remain with the transport:

  * HTTP: `ErrorResponse { code, message, request_id }` stays in `refractive_swan_api` but uses a *stable error code enum* from `refractive_swan_contracts`.
  * CLI: exit codes, human-readable messages remain in CLI.

**Schema versioning:**

* Each public contract struct derives:

  * `Serialize`, `Deserialize`, and optionally `JsonSchema` (if you add `schemars`).
* Add a tiny `schema` module that:

  * Produces JSON Schema snapshots (or type hash manifests) under `target/contracts/`.
  * Associates each schema with a semantic version (e.g. `refractive_swan.contracts.PipelineOutput@v1`).

##### 03.2 Error taxonomy & mapping

Define a single `ErrorKind` enum in `refractive_swan_contracts`:

* Categories:

  * `DomainIngestion`, `DomainMapping`, `DomainCompliance`, `DomainVector`, `DomainWarehouse`.
  * `PlatformConfig`, `PlatformStore`, `PlatformTerminology`.
  * `AppHttpClient`, `AppHttpServer`, `AppCliUsage`, `AppCliRuntime`.

Add mapping tables:

* `refractive_swan_api::ApiError` → `(ErrorKind, http_status, code_string)`.
* `refractive_swan_datamart::LoadError` → `(ErrorKind, detail_string)` but no HTTP.
* `refractive_swan_vector_store::VectorStoreError` → `ErrorKind::DomainVector` / `PlatformStore`.
* `refractive_swan_compliance::ComplianceError` → `ErrorKind::DomainCompliance`.

This gives:

* Stable `code` strings for HTTP and CLI (e.g. `invalid_fhir`, `compliance_blocked`, `vector_backend_unavailable`, `warehouse_export_blocked`).
* A documentation anchor for all error surfaces in the docs.

#### Implementation tasks

##### 03.A – Create `refractive_swan_contracts`

* [x] Add `lib/domain/contracts` crate:

  * [x] Define modules:

    * `pipeline.rs` – `PipelineOutput` re-export + associated mapping summary contracts.
    * `analytics.rs` – `AnalyticsSummaryRow`, `AnalyticsSummaryResponse`, `CohortRow`, `CohortResponse`.
    * `eval.rs` – `DatasetManifest`, `EvalSummary`, `EvalRunResponse`.
    * `metrics.rs` – `PipelineMetrics`, `VectorUsageSnapshot`, `VectorCapacitySnapshot`.
    * `errors.rs` – `ErrorKind`, `ErrorCode` (string) and mapping helpers.
  * [x] Re-export domain types from `refractive_swan_core`, `refractive_swan_pipeline`, `refractive_swan_eval`, `refractive_swan_observability` instead of copying shapes.

##### 03.B – Wire contracts into app/servers

* [x] `refractive_swan_api`:

  * [x] Replace `AnalyticsSummaryResponse`, `AnalyticsNcitSummaryRow`, `CohortResponse`, `CohortRow`, `EvalRunResponse` in `src/dto.rs` with re-exports or thin wrappers around `refractive_swan_contracts`.
  * [x] `MapBundlesResponse`:

    * [x] Represent as “pipeline output envelope” referencing `PipelineOutput` fields (or drop it in favor of `PipelineOutput` when appropriate).
  * [x] `ErrorResponse.code`:

    * [x] Map `ApiError` variants to `ErrorKind + ErrorCode`.
    * [x] Ensure `code` is drawn from `refractive_swan_contracts::ErrorCode`.

* [ ] `refractive_swan_datamart`:

  * [x] Provide analytic query APIs returning `refractive_swan_contracts::AnalyticsSummaryRow` / `CohortRow`, not bespoke structs.
  * [x] Add `LoadSummary` as part of a “warehouse contract” for CLI/HTTP responses.

##### 03.C – CLI & eval alignment

* [x] Update CLI bins (`map_bundles`, `eval_mapping`, `load_datamart`) to:

  * [x] Produce NDJSON whose payload records conform exactly to `refractive_swan_contracts` types:

    * Mapping results: `MappingResult` from `refractive_swan_core`.
    * Analytics: same `AnalyticsSummaryRow` as HTTP.
    * Eval: `EvalSummary`, `EvalRunResponse`.
  * [x] Add tests that run CLI commands in-process (`refractive_swan_test_suite`) and deserialize results using `refractive_swan_contracts`.

##### 03.D – Schema snapshots & CI

* [x] Add a small `contracts-schema` test binary that:

  * [x] Generates JSON Schema (or stable hashes) for all public contracts.
  * [x] Writes them under `code/ci/contracts/*.json`.
* [ ] CI job:

  * [x] Compare generated schemas against committed ones.
  * [x] If any breaking changes occur (field removed/renamed), fail CI and require version bump + migration doc.

#### Acceptance criteria

* All HTTP responses and CLI JSON payloads for:

  * `map-bundles`, `analytics/ncit-summary`, `analytics/cohort`, `eval/*`
    deserialize into the same `refractive_swan_contracts` types.
* `refractive_swan_api`, `refractive_swan_datamart`, CLI, and (later) web frontend import **zero bespoke DTO definitions** for analytics/eval/pipeline; they only use `refractive_swan_contracts`.
* Error codes (`code` fields in HTTP, CLI exit mapping) are documented once in `refractive_swan_contracts::errors` and referenced by all surfaces.
* Schema diff in CI fails if a breaking change is introduced to any public contract struct without a version bump.

### REFR-04 – Domain core & staging (`refractive_swan_core`)

**Goal:** Make `refractive_swan_core` the canonical source of truth for domain entities, IDs, staging rows, and mapping result types, with no IO/env/platform coupling.

- [x] Add `lib/domain/core/README.md` that:
  - [x] Explains the split between `encounter`, `order`, `patient`, `staging`, `mapping`, `value`, and `fhir` modules.
  - [x] Links each module back to the relevant system-design diagrams (FHIR class/model, NCIt architecture, staging tables).
- [x] Restructure `refractive_swan_core` into super-domains (`primitives`, `clinical`, `interop`, `semantics`) with bridges placed under the consumer (`bridge/`) modules.
- [x] Preserve legacy public API paths via `lib.rs` re-exports and `prelude` wiring so `refractive_swan_core::{patient, order, staging, mapping, value, fhir}` continue to work.
- [ ] Review `refractive_swan_core::mapping` types (`CodeElement`, `MappingResult`, `MappingThresholds`, `MappingSourceVersion`, `NCItConcept`, `DimNCITConcept`) and:
  - [x] Identify any duplicate mapping/result structs in other crates and plan to unify them on `refractive_swan_core::mapping`.
  - [x] Add helper constructors/builders for common result patterns (AutoMapped / NeedsReview / NoMatch).
- [ ] Confirm staging structs (`StgServiceRequestFlat`, `StgSrCodeExploded`) match:
  - [x] Ingestion transforms (refractive_swan_ingestion),
  - [x] Pipeline outputs (refractive_swan_pipeline),
  - [x] Datamart schema (dim/fact tables), and document any intentional differences.
- [ ] Ensure `refractive_swan_core` has:
  - [x] No direct `std::env` or file IO,
  - [x] Only serialization and optional `fake`/`Dummy` derives as dependencies.
- [x] Add doc-tests or small unit tests in core modules (value/order/encounter/patient/staging) that mirror the canonical ServiceRequest journey.

### REFR-05 – Domain ingestion & FHIR profiles (`refractive_swan_ingestion`)

**Goal:** Keep ingestion/FHIR-profile logic as a pure domain service that transforms Bundles into staging/domain types and structured validation results, leaving HTTP/env concerns to app/platform layers.

- [x] `refractive_swan_ingestion`
  - [x] Document the ingestion flow from `fhir::Bundle` → `StgServiceRequestFlat` / `StgSrCodeExploded` → `order::ServiceRequest` in a crate-level `//!` header and README.
  - [x] Review `IngestionError` and `ValidationIssue`/`ValidationReport` usage and:
    - [x] Ensure error types don’t encode HTTP/env assumptions.
    - [x] Clarify when validation failures vs decode errors vs external validator failures are returned.
  - [ ] Separate pure transforms (`sr_to_staging`, `sr_to_domain`) from validation orchestration (`bundle_to_*_with_validation`) so callers can compose them independently.
  - [x] Introduce a small, testable trait/port for external validation so app/platform layers can own HTTP clients.
  - [x] Embed FHIR profiles under `profiles/`, document include paths, and test cardinalities/known URLs.
- [x] Add a validated-bundle type to carry `ValidationReport` without double validation. (`ValidatedBundle` + pipeline reuse path.)
  - [ ] Return typed status/intent enums from parsing helpers instead of strings to reduce downstream re-parsing.

### REFR-06 – Domain mapping engine & NCIt integration (`refractive_swan_mapping`)

**Goal:** Keep the mapping engine deterministic and domain-centric, while factoring out policy/vector/terminology wiring so platform/app layers can compose backends cleanly.

- [ ] Document the mapping pipeline in a crate-level `//!` header and README:
  - [x] Clarify roles of `Mapper`, `CandidateRanker`, `MappingEngine`, and policy/vector/terminology integration points.
  - [x] Link to NCIt/obo-graph system-design docs and evaluation epics (mapping/eval harness).
- [ ] Identify places where `refractive_swan_mapping` directly reads env or config (e.g., `load_policy_from_env`):
  - [x] Introduce an explicit `MappingConfig` / policy parameter so mapping functions can be called without reading env.
  - [x] Plan to move env parsing for policies into `refractive_swan_compliance` / `refractive_swan_configuration` (`ComplianceConfig::from_env` now encapsulates `refractive_swan_COMPLIANCE_*` reads and feeds policies into mapping via injected config/policy).
  - [x] Wire CLI (`map_codes`, `map_bundles`, `load_datamart`), API (`refractive_swan_dataplane`{formerly `refractive_swan_api`}), and datamart loaders to construct `ComplianceConfig` once at startup and pass the resulting policy through mapping/pipeline/datamart paths instead of calling `load_policy_from_env` repeatedly.
- [ ] Review vector-related wiring:
  - [x] Ensure `VectorRankerBackend` and `DeterministicEmbeddingProvider` are pure domain constructs that operate purely on traits (`VectorStore`, `EmbeddingProvider`).
  - [x] Avoid coupling mapping to specific backends (Qdrant/pgvector) beyond the trait layer.
- [ ] Align mapping result semantics:
  - [x] Confirm `build_result_with_score` uses a single source of truth for thresholds and `MappingState` transitions.
  - [x] Add refactor tasks for reusing `MappingThresholds`/`MappingSourceVersion` from `refractive_swan_core` consistently.
- [x] Evaluate whether deprecated `eval::run_eval` can be removed or wrapped behind a clearer “mapping eval port” that just delegates to `refractive_swan_eval` (`refractive_swan_mapping::eval` shim deleted; callers use `refractive_swan_eval::run_eval_with_mapper` directly).

### REFR-07 – Domain eval harness & fake data fixtures (`refractive_swan_eval`, `refractive_swan_eval::fake_data`)

**Goal:** Treat eval and fake-data crates as domain-aligned data/eval providers with deterministic behavior and clear boundaries to IO/config, ready to be driven by CLI/web/platform tooling.

- [x] `refractive_swan_eval`
  - [x] Add a crate README explaining the role of EvalCase/EvalSummary, dataset manifests, and baseline snapshots.
  - [x] Separate dataset discovery/loading concerns from scoring/aggregation:
    - [x] Keep file/NDJSON IO behind small helpers that can later be replaced with alternative sources (e.g., HTTP, DB).
    - [x] Clarify how `DEFAULT_DATA_ROOT` and `refractive_swan_EVAL_DATA_ROOT` interact with future configuration layers.
  - [x] Identify any places where eval depends on CLI/web/platform behavior (env, logging) and plan to push those concerns outward.
- [x] `refractive_swan_eval::fake_data`
  - [x] Document the structure of `data/` (eval/meta/regression) and how it maps to `fixtures::*` and generator modules.
  - [x] Centralize RNG seeding and ID generation helpers so CLI bins and tests share deterministic behavior.
  - [x] Ensure value generators (`fake_*_id`, `fake_order_description`, etc.) are thin over core types from `refractive_swan_core` and do not embed app/platform assumptions (URLs, ports, etc.).
  - [x] Cross-check fixture schemas with `refractive_swan_eval::EvalCase` and ingestion/mapping expectations; add small tests that load each tier (bronze/silver/gold) to catch drift.

### REFR-08 – Domain terminology & ontology graph (`refractive_swan_terminology`)

**Goal:** Provide a clean, domain-level terminology layer (code systems, value sets, OBO graphs) that exposes stable APIs for mapping/compliance, with HTTP/env responsibilities clearly separated.

- [x] `refractive_swan_terminology`
  - [x] Add a crate README that explains:
    - [x] The split between `codesystem`, `valueset`, `bridge`, `client`, `obo`, and `registry`.
    - [x] How license tiers and source kinds are intended to feed mapping/compliance decisions.
  - [x] Confirm `canonicalize_system` and `CodeKind` semantics are used consistently by both ingestion and mapping; plan to expose any missing helpers as public API.
  - [x] Inventory all HTTP/env touchpoints in `client::TerminologyClientConfig` and friends; mark them as adapter seams for future platform layering.
- [x] `refractive_swan_terminology::obo_graph`
  - [x] Document the expectations around embedded `.obo` minis vs potential full graphs (NCIT/MONDO) and how graph versions are surfaced.
  - [x] Ensure `CachedOntologyGraph` exposes only pure graph operations (ancestors/descendants/synonyms/related_concepts), leaving file/network IO out of this crate.
  - [x] Add tests or examples tying synonym/related-concept behavior back to mapping/terminology use cases (e.g., PET/CT synonym sets).
  
---

### REFR-09 – Domain pipeline orchestrator (`refractive_swan_pipeline`)

**Epic ID**: `REFR-09`
**Theme**: end-to-end Bundle → PipelineOutput orchestration, env-free and backend-agnostic
**Touches**:

* `lib/domain/meta/pipeline` (`refractive_swan_pipeline`) – `PipelineOutput`, `bundle_to_mapped_sr[_with_vector_context]`.
* `lib/domain/meta/ingestion` (`refractive_swan_ingestion`) – `bundle_to_staging`.
* `lib/domain/ontologies/mapping` (`refractive_swan_mapping`) – mapping pipelines (lexical & vector).
* `lib/app/servers/api` (`refractive_swan_api`) – uses pipeline for HTTP mapping.
* `lib/app/servers/datamart` (`refractive_swan_datamart`) – consumes `PipelineOutput`.
* `lib/app/servers/vector_store` (`refractive_swan_vector_store`) – trait & config for vector integration.
* `lib/platform/observability` (`refractive_swan_observability`) – `VectorUsageSnapshot`.
* `lib/platform/compliance` – indirectly via mapping & datamart.

#### Goal

Make `refractive_swan_pipeline` the **single, pure, deterministic API** for:

* Converting arbitrary FHIR Bundles into a **fully enriched** `PipelineOutput` (staging + mapping + NCIT dim concepts + optional vector usage).
* Without any:

  * env reads,
  * logging initialization,
  * transport awareness (HTTP/CLI),
  * backend-specific knowledge (Qdrant/PGVector).

All runtime-specific wiring (env, DataPlane, vector backends) happens in app/platform crates.

#### Current state

* `refractive_swan_pipeline` currently:

  * Depends on:

    * `refractive_swan_ingestion` (`bundle_to_staging`).
    * `refractive_swan_mapping` (lexical + vector pipelines).
    * `refractive_swan_observability::VectorUsageSnapshot`.
    * `refractive_swan_vector_store` (VectorStore trait, Embedding types, VectorStoreConfig).
  * Provides:

    * `PipelineOutput` with domain-friendly contents.
    * `bundle_to_mapped_sr` (lexical or vector, depending on context).
    * `bundle_to_mapped_sr_with_vector_context` that takes `VectorPipelineContext` (store + config + top_k).
  * Vector path:

    * `try_vector_mapping` uses:

      * `DeterministicEmbeddingProvider` from `refractive_swan_mapping`.
      * `ErasedVectorStore` wrapper (Arc<dyn VectorStore>) so `refractive_swan_mapping` can stay generic.
      * `map_staging_codes_with_vector` to fill mapping results, dim_concepts, usage.

**Good**: pipeline already does not read env, and vector context is injected.
**Remaining gaps** for the epic:

* The trait + config for vector store are physically hosted under `lib/app/servers/vector_store` (which is “app” in naming, but semantically a platform store).
* There is no explicit “pipeline config” struct for switching ingestion/mapping behavior (e.g., validation mode, strict vs lenient, external validator injection).
* `PipelineOutput` is not yet formally part of `refractive_swan_contracts` (REFR-03).

#### Design

##### 09.1 Pipeline as a pure orchestrator

Keep public surface minimal:

* `PipelineOutput` – canonical domain+analytics enabler.
* `PipelineError::Ingestion(refractive_swan_ingestion::IngestionError)` – only domain error it raises today.
* `bundle_to_mapped_sr(bundle: &Bundle) -> Result<PipelineOutput, PipelineError>`:

  * Strictly lexical / rule-based mapping (no vector).
* `bundle_to_mapped_sr_with_vector_context(bundle: &Bundle, ctx: Option<&VectorPipelineContext>)`:

  * Use vector path only if `Some(ctx)` and vector path succeeds; else fall back to lexical path.

Introduce optional future extensions:

* Additional overloads that accept a **pipeline configuration** struct:

  * `PipelineRunConfig { validation_mode, external_validator, mapping_config }`
    but keep them additive.

##### 09.2 Vector integration boundary

Inside `refractive_swan_pipeline`, vector is **entirely erased**:

* Only functions allowed to “see” vector-specific concepts:

  * `VectorPipelineContext` (defined in this crate).
  * `ErasedVectorStore` wrapper that implements `VectorStore` and forwards to the real backend.

No env or backend toggles:

* `VectorPipelineContext` is constructed in `refractive_swan_api` (or CLI) using `config_from_env` and a real `VectorStore` implementation (Qdrant, PGVector, Mock).
* `refractive_swan_pipeline` never calls `config_from_env` directly.

Add invariants:

* If vector mapping fails with `VectorRankerError` / `VectorStoreError`, pipeline logs (via `log::warn`) and **falls back** to lexical mapping, preserving the shape of `PipelineOutput` (vector_usage = `None` or partial) and never panicking.

##### 09.3 Test matrix

* Lexical only:

  * Regression tests using `refractive_swan_test_suite::regression::baseline_fhir_bundle` to ensure stable mapping counts.
  * Assert `vector_usage.is_none()`.
* Vector enabled:

  * Use `MockVectorStore` and `VectorStoreConfig { backend = Mock, enabled = true }`.
  * Assert:

    * `vector_usage.is_some()`.
    * `MappingResult` sets still consistent with lexical-only run (i.e., vector primarily influences ranking, not semantics).
* Failure modes:

  * Broken bundle (missing subject, invalid status) → `PipelineError::Ingestion(IngestionError)`; error is a pure domain mapping of ingestion issues.
  * Vector backend down (Mock unhealthy, Qdrant unreachable) → `warn` and lexical fallback (no panic).

#### Implementation tasks

##### 09.A – Harden vector context interface

* [x] Document `VectorPipelineContext`:

  * [x] Guarantee stable debug formatting (backend, namespace, top_k).
  * [x] Provide builder-like methods: `with_top_k`, later `with_embedding_override`.
* [x] Keep `VectorPipelineContext` independent of any env semantics; no `from_env` here.

##### 09.B – Formalize pipeline config seams (future-proof)

* [x] Introduce `PipelineRunConfig` (optional for now) with:

  * [x] `validation_mode: ValidationMode`.
  * [x] `external_validation_context: ExternalValidationContext<'a>`.
  * [x] `mapping_config: MappingRunConfig` (lexical-only toggle for now).
* [x] Add helper:

  * [x] `bundle_to_mapped_sr_with_validation(bundle, &PipelineRunConfig, ctx: Option<&VectorPipelineContext>)`.
* [x] Keep existing functions as sugar that call this with defaults.

##### 09.C – Remove any logging/env bleed-through

* [x] Ensure `refractive_swan_pipeline` does not:

  * [x] Call `env_logger::init` or `refractive_swan_observability::init_environment`.
  * [x] Read any env variables directly.
* [x] Use `log` macros only (and leave env setup to app layer).

##### 09.D – Contracts & cross-surface alignment (hooks into REFR-03)

* [x] Re-export `PipelineOutput` in `refractive_swan_contracts`.
* [x] Make `refractive_swan_api` and `refractive_swan_datamart` depend on that re-export, not their own copies.
* [x] Add doc references in `refractive_swan_pipeline::lib.rs` and `refractive_swan_pipeline::README.md` to the contracts doc.

#### Acceptance criteria

* There exists a single “happy path” call from FHIR Bundle to `PipelineOutput` that:

  * Does **not** read env or know about Qdrant/PGVector details.
  * Accepts vector context as an injected trait object only.
* Adding a new vector backend (e.g. Milvus) is purely a `refractive_swan_vector_store` change + app wiring; `refractive_swan_pipeline` remains untouched.
* Domain tests in `refractive_swan_pipeline` fully cover lexical vs vector behavior and ingestion errors.

---

### REFR-10 – Platform configuration & env loading (`refractive_swan_configuration`)

**Goal:** Make `refractive_swan_configuration` the single entrypoint for environment/namespace/profile resolution across app, domain, and platform crates, with typed helpers instead of ad-hoc `std::env` and custom flag parsing.

- [x] Add `lib/platform/configuration/README.md` summarizing:
  - [x] How `load_env(namespace)` is intended to be used by `lib/app`, `lib/domain`, and `lib/platform`.
  - [x] The search strategy (`refractive_swan_ENV_FILE`, `refractive_swan_ENV_DIR`, `refractive_swan_WORKSPACE_ROOT`, `data/environment`) and strict vs non-strict modes.
- [x] Extract common helpers from `lib/platform/configuration/src/lib.rs` into reusable functions:
  - [x] Typed flag parsing (bool/int/port) that can be called from vector store, compliance, observability, API/CLI configs.
  - [x] A small “config root” helper that exposes the resolved workspace root and env search dirs to other crates.
- [ ] Inventory env usage in other crates (`refractive_swan_vector_store`, `refractive_swan_compliance`, `refractive_swan_observability`, `refractive_swan_test_suite`, apps) and:
  - [x] Replace bespoke `env::var` + string parsing with typed helpers from `refractive_swan_configuration`.
  - [ ] Ensure all env-namespace names reflect directory structure (`app.web.api`, `platform.vector_store`, etc.) and are documented.
- [x] Add tests for:
  - [x] `workspace_root()` discovery in nested directories and failure modes (`WorkspaceRootNotFound`).
  - [x] Resolution order when `refractive_swan_ENV_FILE`, `refractive_swan_ENV_DIR`, and `refractive_swan_WORKSPACE_ROOT` are all present.
  - [x] Strict vs non-strict behavior in CI and local dev (FileMissing vs silent no-op).

### REFR-11 – Platform compliance & export gating (`refractive_swan_compliance`)

**Goal:** Provide a robust, testable compliance layer that expresses license/tier policies as pure data and config, with env loading and overrides clearly separated from domain mapping and datamart export.

- [x] Add `lib/platform/compliance/README.md` that:
  - [x] Explains `ComplianceMode`, `ComplianceAction`, `Policy`, and their relationship to mapping/export.
  - [x] Documents env variables (`refractive_swan_COMPLIANCE_MODE`, `refractive_swan_COMPLIANCE_POLICY_PATH`, `refractive_swan_WORKSPACE_ROOT`) and example policies.
- [x] Refactor `load_policy_from_env`:
  - [x] Introduce a `ComplianceConfig` struct that encapsulates env-derived settings (mode, policy path, workspace root).
  - [x] Use `refractive_swan_configuration` helpers for env/paths instead of calling `env::var` directly.
  - [x] Return structured errors without panicking and ensure message clarity for CI logs.
- [x] Ensure `refractive_swan_mapping`, `refractive_swan_pipeline`, and `refractive_swan_datamart` call `Policy::default_for_mode` or injected `Policy` instead of repeatedly calling `load_policy_from_env`:
  - [x] Plan entrypoints where app/web/CLI wires a `Policy` into mapping/pipeline/warehouse surfaces.
  - [x] Keep mapping domain code free of direct env/config parsing.
- [x] Add tests covering:
  - [x] Policy overrides from JSON/YAML across all modes and actions.
  - [x] Export gating semantics for each license tier in `Internal`, `Partner`, and `OpenSource` modes.
  - [x] Error behavior when policy files are malformed or missing, including `refractive_swan_WORKSPACE_ROOT` fallbacks.

### REFR-12 – Platform observability & metrics (`refractive_swan_observability`)

**Goal:** Turn `refractive_swan_observability` into a stable, opt-in metrics/logging adapter that can be wired from app/CLI/API, without panicking on env load and with clear contracts for pipeline/mapping/vector metrics.

- [x] Replace the panicking `OBS_ENV` initializer:
  - [x] Make `init_environment()` return a `Result<(), EnvLoadError>` that callers can handle, instead of `panic!`.
  - [x] Use `refractive_swan_configuration::load_env("platform.observability")` with clear error messages and tests.
- [x] Clarify the ownership of `PipelineMetrics`:
  - [x] Document which surfaces update which fields (pipeline, vector store, analytics, cohorts).
  - [x] Ensure fields like `analytics_requests`, `cohort_queries`, `cohort_results_total` are updated only in app/API layers, not domain crates.
- [x] Add helper functions for:
  - [x] Emitting metrics snapshots as JSON structs that can be consumed by `/metrics/summary` endpoints and CLI exporters.
  - [x] Mapping `VectorUsageSnapshot` and vector capacity proxies (`geom_rm_sqrt_dm`, `cap_alpha_sim`) into pipeline metrics consistently.
- [x] Add tests that:
  - [x] Confirm `log_pipeline_output` and `log_no_match` do not panic when env files are missing (non-strict mode).
  - [x] Verify metrics counters (bundle_count, mapping_count, license_blocked, vector_* fields) match expectations for sample pipeline runs.
  - [x] Cover behavior when vector capacity is missing vs present (capacity fields remain `None` vs set).

### REFR-13 – Platform vector backends (`refractive_swan_vector_store`)

**Epic ID**: `REFR-13`
**Theme**: vector backends as platform store, with central config and capacity reporting
**Touches**:

* `lib/app/servers/vector_store` (`refractive_swan_vector_store`) – entire crate.
* `lib/domain/ontologies/mapping` (`refractive_swan_mapping`) – uses `VectorStore`, `EmbeddingProvider`.
* `lib/domain/meta/pipeline` (`refractive_swan_pipeline`) – uses `VectorStore`, `VectorStoreConfig`.
* `lib/app/servers/api` (`refractive_swan_api`) – builds `VectorPipelineContext` from `config_from_env`.
* `lib/platform/configuration` (`refractive_swan_configuration`) – env parsing helpers.
* `lib/platform/observability` (`refractive_swan_observability`) – consumes `VectorUsageSnapshot`.

#### Goal

Make `refractive_swan_vector_store` the **single, well-documented platform abstraction** for all vector operations:

* Clear separation between:

  * **Config** (env → `VectorStoreConfig` + validation).
  * **Backends** (Qdrant, PGVector, Mock).
  * **Usage metrics & capacity proxies** (feeding observability & manifold-capacity work).
* Ensure adding new backends or capacity signals requires no changes to domain crates.

#### Current state

* `refractive_swan_vector_store` already has:

  * `VectorBackend` enum.
  * `config_from_env()` using `refractive_swan_configuration`.
  * `VectorStore` trait.
  * Mock/Qdrant/PgVector backends.
  * `VectorUsageCounters`, `VectorUsageHandle`, `CapacityProxies`, `VectorUsageSnapshot` integration.
* `refractive_swan_mapping` uses a generic `VectorRankerBackend<S, E>` that only sees the trait + config, not env.
* `refractive_swan_pipeline` uses a `VectorPipelineContext` holding `Arc<dyn VectorStore>` + config.

Remaining work is mostly **formalizing this as platform** and tightening guarantees.

#### Design

##### 13.1 Elevate crate to platform naming

Conceptually, this crate is `platform/stores/vector_store` even though it lives under `app/servers` in your current tree.

* The epic assumes:

  * Logical package: `refractive_swan_vector_store` = platform store.
  * Physical relocation can happen later; the semantic refactor comes first.

##### 13.2 Config semantics

`config_from_env` should be:

* **Pure**: env → config struct → `validate()`.
* **Explicit failure modes**:

  * `MissingNamespace`, `MissingUrl`, `InvalidPoolMax`, `InvalidTimeout`, `InvalidEnv`.

Rules:

* When `enabled = false`:

  * Namespace can be omitted; `backend` may default to `Mock`; url optional.
* When `enabled = true`:

  * Namespace must be non-empty.
  * URL required for all non-Mock backends.
  * `pool_max > 0`, `health_timeout_ms > 0`.

Add doc examples:

* “Fully disabled” example (vector fallback always triggered).
* “Mock only” example (for tests).
* “Qdrant prod” example (namespaces per environment).
* “PGVector dev” example.

##### 13.3 Backend contracts

**QdrantVectorStore**

* `ensure_collection(namespace, dim)`:

  * Must be idempotent and safe under concurrent calls.
* `index_items`:

  * Must:

    * Enforce consistent vector dimension within a batch; otherwise return `IndexFailed("dimension mismatch in batch")`.
  * Use `wait=true` semantics for Qdrant so writes are durable before returning.
* `search`:

  * Use `health_timeout_ms` as client timeout.
  * Map errors into `SearchFailed(String)`; never panic.

**PgVectorStore**

* `CREATE EXTENSION IF NOT EXISTS vector` + `CREATE TABLE IF NOT EXISTS ncit_vectors (...)` is safe to call repeatedly.
* `index_items`:

  * Enforce consistent dimension; otherwise `IndexFailed("dimension mismatch in batch")`.
* `search`:

  * Use `<->` operator; map distance to score (e.g., `1.0 - distance` as in current code).
  * On connection issues, return `BackendUnavailable`.

**MockVectorStore**

* Deterministic behavior:

  * For a given `(namespace, query_vec, top_k)` and no `set_response`, you get deterministic hits.
* `health`:

  * Respects `healthy` flag and returns `BackendUnavailable` when false.
* `namespace` mismatch:

  * Should yield `InvalidNamespace` and increment **fallback** counters (but not queries/hits).

##### 13.4 Metrics & capacity

* `VectorUsageCounters` increments:

  * `vector_queries` on each attempted `search`.
  * `vector_hits` by `hits.len()` for successful `search` results.
  * `vector_fallbacks` when:

    * Namespace mismatch.
    * Backend disabled.
    * Backend unavailable or search error.

* `CapacityProxies` wires into `VectorUsageSnapshot.capacity`:

  * For now, these may be `None`; later, Qdrant/PGVector backends can populate them with approximations (e.g., average norm stats, dataset-specific metrics computed offline).

#### Implementation tasks

##### 13.A – Config tightening

* [x] Review `config_from_env`:

  * [x] Ensure all env parsing uses `refractive_swan_configuration::{bool_var,u32_var,u64_var}`.
  * [x] Add robust unit tests (some exist) for:

    * [x] `pool_max = 0`, `health_timeout_ms = 0`.
    * [x] `refractive_swan_VECTOR_ENABLED=true` + empty namespace.
    * [x] `refractive_swan_VECTOR_ENABLED=true` + non-mock backend + missing URL.

##### 13.B – Backend behavior tests

* [x] Add unit/integration tests covering backend behavior contracts (Mock coverage + existing backend guards):

  * [x] Validate that repeated `index_items` on the same namespace+dim is safe.
  * [x] Validate that dimension mismatch yields `IndexFailed`.
  * [x] Validate timeouts do not wedge (latency confined to `search`).

* [x] Tighten `MockVectorStore` tests:

  * [x] Already present tests cover namespace mismatch + fallback; extend to ensure:

    * [x] `search(namespace, vec, 0)` returns empty result and does not increment counters.
    * [x] Simulated latency is applied only to search, not to `health`.

##### 13.C – Pipeline & mapping integration

* [x] `refractive_swan_mapping`:

  * [x] Confirm `VectorRankerBackend` uses only the trait + config; no env or backend-specific logic.
  * [x] Ensure `usage_handle` is used to emit `VectorUsageSnapshot` used by `refractive_swan_pipeline`.
* [x] `refractive_swan_pipeline`:

  * [x] Confirm `VectorPipelineContext` uses `VectorStoreConfig` only as metadata; all env for config comes from `refractive_swan_api` or CLI.

#### Acceptance criteria

* All vector-enabled code paths (`refractive_swan_mapping`, `refractive_swan_pipeline`, `refractive_swan_api`, CLI) depend only on `VectorStore` trait + `VectorStoreConfig` type; env is centralized in `config_from_env` or a platform config builder.
* Backends (Qdrant/PGVector/Mock) are swappable without touching any domain crate.
* Metrics and capacity snapshots from `VectorUsageSnapshot` are stable enough to feed capacity/geometry analysis without API changes.


### REFR-14 – Platform test harness & regression suite (`refractive_swan_test_suite`)

**Goal:** Consolidate cross-crate testing concerns (fixtures, assertions, env bootstrapping) inside `refractive_swan_test_suite` so that app/domain/platform tests use a single, well-defined harness without unsafe env mutations.

- [x] Add `lib/platform/test_suite/README.md` that:
  - [x] Explains the structure of `src/` (assertions, fixtures, regression) and `tests/` (e2e, integration, unit).
  - [x] Describes how other crates should depend on this crate (for fixtures, not for production code).
- [x] Refactor env handling in `TEST_SUITE_ENV`:
  - [x] Avoid `unsafe` `set_var` by providing explicit setup helpers (e.g., `init_eval_data_root(workspace_root)`).
  - [x] Use `refractive_swan_configuration` to discover workspace root and env files for test namespaces instead of hard-coded ancestor traversal.
- [x] Clarify fixture ownership:
  - [x] Ensure all regression/eval fixtures live under `lib/domain/meta/evaluation/data/**` and are accessed via `refractive_swan_eval::fake_data::fixtures::Registry` helpers.
  - [x] Document how new datasets/fixtures should be added (naming, manifests, baseline summaries) so tests remain stable.
- [x] Harden test surfaces:
  - [x] Ensure that e2e/integration/unit test modules do not depend on internal APIs that are likely to change; prefer public ports (CLI/app services, pipeline, datamart, API endpoints).
  - [x] Add a small “smoke test index” that verifies all major flows (ingestion, mapping, eval, vector, warehouse, web API) still run after refactors.
- [x] Add CI guidance:
  - [x] Document which features (`backend-pgvector`, external validation mocks) must be enabled for the full suite.
  - [x] Provide recommended command lines (`cargo test -p refractive_swan_test_suite --features backend-pgvector`) to reproduce CI locally.
- [x] Harden eval dataset root detection so `ensure_eval_data_root` falls back to the bundled fixtures when overrides are missing or point at the parent `.../data` directory, and document the behavior across the setup/env quickstarts.

---

### REFR-15 – CLI surfaces & orchestration (`refractive_swan_cli`)

**Goal:** Treat `refractive_swan_cli` as a thin orchestration layer over domain + platform crates, with shared IO/config/compliance handling and consistent UX across all binaries.

- [x] Add/expand `lib/app/frontend/cli/README.md` to:
  - [x] Map each bin (`map_bundles`, `map_codes`, `eval_mapping`, `validate_fhir`, `load_datamart`, `build_vector_index`) to its underlying domain flows (ingestion, mapping, eval, datamart load, vector index).
  - [x] Document common flags (env namespace, log level, compliance behavior) and how they relate to API/web behavior.
- [x] Introduce a small internal “CLI core” module (e.g., `src/cli_core.rs`) that:
  - [x] Provides shared helpers for env loading (`load_env("app.cli")`), log initialization, and structured error reporting.
  - [x] Wraps repeated NDJSON reading/writing logic (streaming readers for Bundles, StgSrCodeExploded, PipelineOutput, EvalCase).
  - [x] Centralizes exit code conventions (e.g., non-zero on compliance block, threshold failure, invalid input).
- [x] `map_bundles`:
  - [x] Replace inline env/logging setup with shared CLI core helpers and ensure validation + pipeline + metrics calls are consistently structured.
  - [x] Add a streaming path (reading Bundles incrementally) that still accumulates a single `PipelineMetrics` summary.
  - [x] Align JSON output schema (`kind` field) with API responses for easier downstream parsing.
- [x] `map_codes`:
  - [x] Share vector config handling with `build_vector_index` (via a small helper that wraps `config_from_env` and backend selection).
  - [x] Move compliance reporting and `fail_on_license_block` behavior behind a single helper used by both CLI and API surfaces.
  - [x] Ensure explanation output (`--explain`, `--explain-top`) is documented and stable for downstream tooling.
- [x] `eval_mapping`:
  - [x] Factor dataset loading/reporting into reusable helpers that mirror API eval endpoints (dataset list, summary, run).
  - [x] Clarify top-k semantics (“placeholder until multi-candidate support”) and future-proof the flag by plumbing top-k into `refractive_swan_mapping` when available.
  - [x] Ensure threshold/compare/deterministic checks share code with API eval (minimize duplicated logic).
- [x] `validate_fhir`:
  - [x] Align mode flags (`lenient`, `strict`, `external_preferred`, `external_strict`) with ingestion docs and API options.
  - [x] Provide a stable NDJSON output schema for issues and summaries that can be consumed by CI dashboards and refractive_swan_web_frontend.
- [x] `load_datamart`:
  - [x] Remove duplicated export-policy logic by delegating to a shared helper that is also used in `refractive_swan_dataplane`{formerly `refractive_swan_api`} and `refractive_swan_datamart`.
  - [x] Ensure `WarehouseConfig::from_env` uses `refractive_swan_configuration` for env parsing and that CLI errors surface actionable messages for missing URL/schema/permissions.
- [x] `build_vector_index`:
  - [x] Refactor panicking paths (dimension overflows, unsupported backends) into structured CLI errors.
  - [x] Share embedding/version metadata semantics with mapping/vector-store docs (documented in mdBook and CLI help).

### REFR-16 – HTTP backend, warehouse & analytics surfaces (`refractive_swan_api` + `refractive_swan_datamart`)

**Epic ID**: `REFR-16`
**Theme**: single HTTP gateway backed by a warehouse-focused platform layer
**Touches**:

* App:

  * `lib/app/servers/api` (`refractive_swan_api`) – `server.rs`, `dto.rs`, `main.rs`.
  * `lib/app/servers/datamart` (`refractive_swan_datamart`) – `dim.rs`, `fact.rs`, `keys.rs`, `sql.rs`, `lib.rs`.
* Domain:

  * `refractive_swan_pipeline`, `refractive_swan_mapping`, `refractive_swan_eval`, `refractive_swan_core`, `refractive_swan_terminology`.
* Platform:

  * `refractive_swan_configuration`, `refractive_swan_compliance`, `refractive_swan_observability`, `refractive_swan_vector_store`.
  * Potential future: `platform/data/fabric`, `platform/data/warehouse`, `platform/data/datalake`.

#### Goal

Make `refractive_swan_api` the **only HTTP ingress/egress** for:

* Mapping (Bundle → PipelineOutput).
* Evaluation (eval datasets / runs).
* Analytics (NCIT summary + cohorts).

and ensure that:

* Analytics & metrics are derived from **warehouse-backed** data (`refractive_swan_datamart`), not from in-memory mirror state.
* A single `DataPlane`-like struct wires:

  * Relational store (warehouse).
  * Vector store.
  * Compliance policy.
  * Dataset store.
  * Observability metrics.

#### Current state

* `refractive_swan_api`:

  * Builds `ApiState::new`:

    * `ComplianceConfig::from_env` → `Policy`.
    * `FileDatasetStore` from `refractive_swan_EVAL_DATA_ROOT` or default root.
    * `VectorPipelineContext` from `config_from_env()`.
    * `AnalyticsPersistence` from `WarehouseConfig::from_env()`.
    * `AnalyticsState` as in-memory dim/fact holder.
  * Exposes routes:

    * `/health`, `/metrics/summary`.
    * `/analytics/ncit-summary`, `/analytics/cohort` (currently in-memory).
    * `/api/map-bundles`.
    * `/api/eval/summary`, `/api/eval/datasets`, `/api/eval/run`, `/api/eval/latest`.

* `refractive_swan_datamart`:

  * Knows how to:

    * Transform `PipelineOutput` → dims/facts.
    * Migrate & load dims/facts into SQLite.
  * Does **not yet** expose analytics queries for NCIT summary/cohorts; those are duplicated in `AnalyticsState`.

#### Design

##### 16.1 DataPlane concept (per node)

Introduce a *conceptual* `NodeDataPlane` (can live as a struct in `refractive_swan_api` first; later factor into `platform/data/fabric`):

* Fields:

  * `warehouse_cfg: WarehouseConfig`.
  * `warehouse_pool: Pool<Sqlite>` (or trait object).
  * `vector_ctx: Option<VectorPipelineContext>`.
  * `policy: Policy`.
  * `dataset_store: FileDatasetStore`.
  * `metrics: PipelineMetrics`.
* Responsibilities:

  * `map_bundles`:

    * Use `refractive_swan_pipeline` with `vector_ctx`.
    * Apply `policy` using `refractive_swan_compliance::assert_export_allowed`.
    * Persist dims/facts via `refractive_swan_datamart::load_from_pipeline_output`.
    * Update metrics.
  * `analytics`:

    * Query warehouse via new functions in `refractive_swan_datamart` (no in-memory duplication).
  * `eval`:

    * Orchestrate `refractive_swan_eval` + `refractive_swan_mapping` with dataset roots.

In code this might remain inside `ApiState`, but the epic expects a structural **separation of concerns**.

##### 16.2 Warehouse-backed analytics

Extend `refractive_swan_datamart`:

* Add query functions:

  ```rust
  pub async fn ncit_summary(pool: &Pool<Sqlite>) -> Result<Vec<AnalyticsSummaryRow>, sqlx::Error>;

  pub async fn cohort(
      pool: &Pool<Sqlite>,
      query: &CohortQuery, // re-used from refractive_swan_api or moved to contracts
  ) -> Result<CohortResponse, sqlx::Error>;
  ```

* Implement queries equivalent to `AnalyticsState::ncit_summary` + `cohort`:

  * Group by NCIT ID + mapping state + day (ordered_at’s date) using SQL.
  * Join dims & facts to fetch patient/encounter IDs and mapping state.
  * Use `DimNCIT` & `DimCode` relationships.

Then:

* Deprecate `AnalyticsState`:

  * Replace analytics endpoints with warehouse-backed versions:

    * `/analytics/ncit-summary` calls `refractive_swan_datamart::ncit_summary(pool)` directly.
    * `/analytics/cohort` calls `refractive_swan_datamart::cohort(pool, &query)`.

**Result**: in-memory state becomes a cache or demo only, not a source of truth.

##### 16.3 Config consolidation

`ApiServerConfig` today reads:

* `refractive_swan_API_HOST` (raw env).
* `refractive_swan_API_PORT` via `refractive_swan_configuration::port_var`.

Epic expects:

* Use `refractive_swan_configuration::load_env("app.web.api")` once in `main`.
* `ApiServerConfig` should be constructed from typed helpers (host, port, feature flags) with clear defaults & logging.

Similarly for:

* `WarehouseConfig::from_env()` – currently calling `load_env("domain.datamart")` but then using `std::env`.
* Vector context – should use `refractive_swan_configuration` for URL/namespace flags.

##### 16.4 Metrics & error semantics

Ensure:

* `/metrics/summary` exposes a stable JSON from `PipelineMetrics` (that later becomes part of `refractive_swan_contracts`).
* Compliance violations and exporter gating are consistently logged:

  * Mapping path logs `compliance_blocked` via `ApiError::Compliance`.
  * Datamart `LoadError::Compliance` uses the same messaging structure, and if/when exposed over HTTP, uses the same error code.

#### Implementation tasks

##### 16.A – Build a minimal DataPlane struct

* [x] In `refractive_swan_api::server`, introduce:

  ```rust
  pub struct NodeDataPlane {
      policy: Policy,
      analytics_persistence: AnalyticsPersistence,
      vector_context: Option<VectorPipelineContext>,
      dataset_store: Arc<FileDatasetStore>,
      metrics: Arc<Mutex<PipelineMetrics>>,
      // future: relational store traits
  }
  ```

* [x] Let `ApiState` wrap a `NodeDataPlane` instead of keeping all fields separately.

##### 16.B – Warehouse queries

* [x] Extend `refractive_swan_datamart::sql` with `ncit_summary_query` & `cohort_query` functions returning rows that can be mapped to `refractive_swan_contracts::AnalyticsSummaryRow` and `CohortRow`.
* [x] Add new functions in `refractive_swan_datamart::lib`:

  ```rust
  pub async fn ncit_summary(pool: &Pool<Sqlite>) -> Result<Vec<AnalyticsSummaryRow>, sqlx::Error>;
  pub async fn cohort(pool: &Pool<Sqlite>, q: &CohortQuery) -> Result<CohortResponse, sqlx::Error>;
  ```

##### 16.C – Replace in-memory analytics

* [x] Modify `/analytics/ncit-summary` and `/analytics/cohort` handlers to:

  * [x] Drop usage of `AnalyticsState` except maybe as an optional cache.
  * [x] Use warehouse queries + map results to `refractive_swan_contracts` types.
  * [x] Ensure metrics (`analytics_requests`, `cohort_queries`, `cohort_results_total`) still update.

##### 16.D – Config and env cleanup

* [x] `refractive_swan_api::main()`:

  * [x] Keep `load_env("app.web.api")`, but rely on typed configuration inside `refractive_swan_configuration` for host/port.
* [x] `WarehouseConfig::from_env()`:

  * [x] Replace ad-hoc `std::env::var` with `refractive_swan_configuration::string_var` / `u32_var`.
  * [x] Return `Result<Self, EnvValueError>` (or similar) for better error reporting.

#### Acceptance criteria

* Analytics endpoints (`/analytics/*`) work with **no in-memory state** if the process is restarted; all data is persisted in SQLite and read from there.
* `refractive_swan_api` is the only server crate exposing HTTP; any future HTTP surfaces reuse its contracts (REFR-03).
* Vector store + warehouse config errors are surfaced as documented HTTP errors / log messages, not panics.




## DONE
- _Empty_

---
