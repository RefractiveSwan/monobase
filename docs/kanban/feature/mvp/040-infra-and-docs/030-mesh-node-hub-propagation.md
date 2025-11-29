# 030 — Mesh node/hub propagation & server/frontend integration

**Theme:** Mesh node/hub propagation & server/frontend integration  
**Branch:** `feature/app/BE-029-backend-expansion`  

> Status: **INPROGRESS**  
> Branch target version: `Unreleased`  
> Introduced in: `v0.1.0`  
> Last updated in: `v0.1.0`  
> Branch note: branch name does not include the 030 epic ID; using `feature/app/BE-029-backend-expansion` for this work.

**Related crates / paths:**

- **Domain & ports**
  - `lib/domain/meta/contracts` (`refractive_swan_contracts`)
  - `lib/domain/meta/evaluation` (`refractive_swan_eval`)
  - `lib/domain/meta/ingestion` (`refractive_swan_ingestion`)
  - `lib/domain/meta/pipeline` (`refractive_swan_pipeline`)
  - `lib/domain/ontologies/mapping` (`refractive_swan_mapping`)
  - `lib/domain/ontologies/terminology` (`refractive_swan_terminology`)
  - `lib/domain/ports/**` (`refractive_swan_*_port`)

- **Platform / mesh / data-plane**
  - `lib/platform/mesh/node` (`refractive_swan_mesh_node`)
  - `lib/platform/mesh/hub` (`refractive_swan_mesh_hub`)
  - `lib/platform/mesh/governance` (`refractive_swan_mesh_governance`)
  - `lib/app/servers/api` (`refractive_swan_api`)
  - `lib/platform/data/data-plane/mart` (`refractive_swan_datamart`)
  - `lib/platform/data/data-plane/warehouse` (`refractive_swan_datawarehouse` – design)
  - `lib/platform/data/data-plane/lake` (`refractive_swan_datalake` – design)
  - `lib/platform/data/data-stores/vector_store` (`refractive_swan_vector_store`)
  - `lib/platform/data/data-stores/relational_store` (`refractive_swan_relational_store` – design)
  - `lib/platform/data/data-stores/cache_store` (`refractive_swan_cache_store` – design)

- **Platform / meta**
  - `lib/platform/meta/configuration` (`refractive_swan_configuration`)
  - `lib/platform/meta/compliance` (`refractive_swan_compliance`)
  - `lib/platform/meta/observability` (`refractive_swan_observability`)

- **Frontend**
  - Web frontend (“mesh dashboard”) consuming `/api/v1/**`, `/mesh/**`, `/hub/**`

---

## 1. Problem & goal

We have:

- A **domain layer** that already provides:
  - Ingestion: `refractive_swan_ingestion` -> `StgServiceRequestFlat`, `StgSrCodeExploded`, `ValidationReport`.
  - Mapping: `refractive_swan_mapping` -> NCIt-aligned `MappingResult`, `DimNCITConcept`, `MappingSummary`.
  - Pipeline: `refractive_swan_pipeline::DefaultPipeline` -> `PipelineExecution` / `PipelineOutput`.
  - Eval: `refractive_swan_eval` -> `EvalCase`, `EvalSummary`, `FileDatasetStore`, `DatasetManifest`.
  - Contracts: `refractive_swan_contracts` -> `AnalyticsSummaryResponse`, `CohortResponse`, `EvalRunResponse`, `LoadSummary`, `Mesh*` DTOs, `ErrorCode/Kind`, `AdminEvent`.

- A **node runtime** (`NodeDataPlane` in `refractive_swan_mesh_node`) that wires:
  - `Policy` from `refractive_swan_compliance`.
  - Datasets via `EvalDatasetConfig` -> `FileDatasetStore`.
  - Datamart via `WarehouseConfig` -> `SqliteDatamart` implementing `DatamartSink`.
  - Vector backend via `VectorStoreConfig` -> `QdrantVectorStore` / `PgVectorStore` / `MockVectorStore`.
  - Metrics via `PipelineMetrics` from `refractive_swan_observability`.

- A **gateway server** (`refractive_swan_api`) that already exposes mapping, analytics, eval, dataset admin over `/api/v1/**`.

- Conceptual **hub** and **governance** runtimes (READMEs, DTOs), plus future `warehouse`/`lake`/`cache` abstractions, but not yet wired into a coherent, federated mesh.

We want a **concrete node/hub propagation system** such that:

1. Each **node** can:
   - self-host data (datamart, vector store, eval datasets, future lake exports),
   - expose a stable **mesh surface** (`/mesh/job`, `/mesh/capabilities`, `/mesh/health`),
   - implement mesh jobs in terms of **domain crates** (pipeline, eval, datamart).

2. One or more **hubs** can:
   - discover & register nodes (`NodeCapabilities`, `NodeMetadata`),
   - dispatch `MeshJobDescriptor` jobs to nodes and aggregate `MeshJobResult`s,
   - enforce **governance & DP budgets** consistently with `refractive_swan_mesh_governance` + `refractive_swan_compliance`,
   - optionally ingest DP-safe exports from a **lake** into a hub reporting warehouse.

3. The **frontend** can:
   - talk to a **single HTTP surface** to:
     - visualize node status, governance, vector/datamart health,
     - run local or federated analytics/eval jobs,
     - inspect mesh topology and admin events.

**Success =** A dev can spin up ≥2 nodes + 1 hub, hit the existing API/mesh/hub endpoints, and:

- map bundles at each node independently,
- run **federated NCIt summary** across nodes,
- run **federated eval** on `refractive_swan_eval` datasets,
- see node-specific metrics, compliance mode, vector/datamart health,
- see hub aggregate analytics, node registry, and governance-enforced denials.

---

## 2. Out of scope (for this kanban)

- Production-grade **authN/authZ** between hub and nodes (JWT/mTLS): design hooks only.
- Scheduling & orchestration of long-running, async jobs (e.g., distributed training).
- Full-blown data lake & reporting warehouse implementation (we design traits and stubs; real implementation may live in later epics).
- Non-HTTP transports (gRPC / v3 is reserved, not implemented here).

---

## 3. High-level phases

0. **P0 — Domain, contracts, config & cross-cutting alignment**  
1. **P1 — Node mesh surface & node data-plane wiring**  
2. **P2 — Hub runtime crate & reporting warehouse/lake integration**  
3. **P3 — Governance & DP budgets integration (node + hub)**  
4. **P4 — Frontend mesh dashboard (server ↔ frontend connection)**  
5. **P5 — E2E multi-node/hub test harness & runbook**

---

## 4. Stories & tasks

---

### P0 — Domain, contracts, config & cross-cutting alignment

#### P0.1 — Mesh contracts ↔ domain contracts mapping

- **Status:** TODO  
- **Path:** `lib/domain/meta/contracts/src/mesh.rs`, `{analytics,eval,pipeline}.rs`

**Tasks**

- [x] Ensure (with tests) `MeshJobType` variants map directly onto existing contracts:

  - `EvalDataset` -> `EvalRunResponse` (dataset name + `EvalSummary`).
  - `AnalyticsQuery` -> `AnalyticsSummaryResponse` / `CohortResponse`.
  - `MappingHealthCheck` -> small struct derived from `PipelineMetrics` (and maybe sample eval results).
  - `ExportJob` -> `LoadSummary` / export metadata.
  - `NodeIntrospection` -> JSON view combining `PipelineMetrics` + `VectorUsageSnapshot` + any node-local state (dataset manifests, feature toggles).

  - [x] Document expected `MeshJobDescriptor.parameters` shapes per job type.
  - `EvalDataset`: `{ "dataset": "bronze_pet_ct_small", "top_k": 5 }`
  - `AnalyticsQuery`: `{ "query_type": "ncit_summary" }` or `{ "query_type": "cohort", "filters": { ... } }`
  - `MappingHealthCheck`: `{}`
  - `ExportJob`: `{ "export": "ncit_summary" }` (DP/export policy applies)
  - `NodeIntrospection`: `{}`
  - [x] Publish JSON schema for `MeshJobResult.output` per job type under `ci/contracts`.
  - Parameters/output reference:

    | MeshJobType            | Parameters example                                            | Output schema / contract                              |
    | ---------------------- | ------------------------------------------------------------- | ----------------------------------------------------- |
    | `EvalDataset`          | `{ "dataset": "bronze_pet_ct_small", "top_k": 5 }`            | `eval_run_response.schema.json` (`EvalRunResponse`)   |
    | `AnalyticsQuery`       | `{ "query_type": "ncit_summary" }` or `{ "query_type": "cohort", "filters": { ... } }` | `analytics_summary_response.schema.json` / `cohort_response.schema.json` |
    | `MappingHealthCheck`   | `{}`                                                          | `mapping_health_check_report.schema.json`             |
    | `ExportJob`            | `{ "export": "ncit_summary" }`                                | `export_job_summary.schema.json` (`LoadSummary`-backed) |
    | `NodeIntrospection`    | `{}`                                                          | `node_introspection_view.schema.json`                 |
    
- [x] Define a minimal canonical schema for `MeshJobResult.output` per `MeshJobType` (JSON schemas can live under `ci/contracts`).
  - [x] Generate `mesh_job_descriptor.schema.json` and `mesh_job_result.schema.json` via `contracts_schema` bin; add parameters table above.
  - [x] Add meshes doc cross-reference from `node-runtime.md` and schema generator runbook.

---

#### P0.2 — Domain helpers for mesh jobs (eval + analytics)

- **Status:** INPROGRESS  
- **Path:** `lib/domain/meta/pipeline/src/lib.rs`, `lib/domain/meta/evaluation/src/lib.rs`, `lib/platform/data/data-plane/mart/src/lib.rs`

**Tasks**

- [x] Add or document helpers:

  - `run_eval_dataset_with_pipeline(store: &FileDatasetStore, dataset: &str, pipeline: &dyn PipelinePort) -> EvalRunResponse`.
  - `node_ncit_summary(datamart: &dyn DatamartSink) -> AnalyticsSummaryResponse`.
  - `node_cohort(datamart: &dyn DatamartSink, filters: CohortFilters) -> CohortResponse`.

- [x] Make these the **canonical building blocks** used in node mesh job execution.

---

#### P0.3 — Terminology & mapping observability for federated reports

- **Status:** INPROGRESS  
- **Path:** `lib/domain/ontologies/mapping`, `lib/domain/ontologies/terminology`, `lib/domain/meta/evaluation`

**Tasks**

- [x] Confirm `MappingSummary` and `EvalSummary` capture:

  - distribution by `CodeKind` (`KnownLicensedSystem`, `KnownOpenSystem`, `OboBacked`, etc.),
  - distribution by license tier (`licensed`, `open`, `internal_only`).

- [x] Decide which slices of these metrics are meaningful at **hub** level (e.g., aggregated by system, by tier) and consider exposing them via `refractive_swan_contracts` if the hub needs a typed federated summary.
  - Added federated contracts/helpers: `MappingSummarySlice`, `FederatedMappingSummary`, `FederatedEvalSummary`, `aggregate_mapping_summaries`, `aggregate_eval_summaries` in `refractive_swan_contracts::federated`.
  - Hub eval endpoint aggregates per-node eval summaries and returns aggregated + per-node payloads; mapping aggregation ready for hub wiring.

---

#### P0.4 — Config, compliance, observability baselines

- **Status:** INPROGRESS  
- **Path:**  
  - `lib/platform/meta/configuration`  
  - `lib/platform/meta/compliance`  
  - `lib/platform/meta/observability`

**Tasks**

- [x] Standardize env/profile loading usage:

  - Nodes: `load_env("app.web.api")` -> `ApiConfig::from_env()` -> `NodePlaneConfig::from_env("app.web.api")`.
  - Hub: `load_env("platform.mesh.hub")` -> `HubConfig::from_env("platform.mesh.hub")`.
  - Vector store: `load_env("platform.vector_store")` via `config_from_env()`.
  - Compliance: `load_env("platform.compliance")` via `ComplianceConfig::from_env()`.
  - Observability: one-time `refractive_swan_observability::init_environment()` (non-fatal if env missing in non-strict mode).

- [ ] Ensure (with tests) `NodeDataPlane` and hub runtime each receive a **pre-built** `Policy`:
  - no domain crate calls `load_policy_from_env` directly.
- [x] Ensure (with tests) all mesh endpoints use `PipelineMetrics` + `metrics_snapshot_json` for `/metrics/summary`, `/mesh/health`, `/hub/nodes/:id` views.
- [ ] Document `heavy-tests` toggle for slow mesh/CLI tests and align CI usage.

---

### P1 — Node mesh surface & node data-plane wiring

#### P1.1 — Extract mesh node config + state wiring

- **Status:** INPROGRESS  
- **Path:**  
  - `lib/platform/mesh/node/src/config.rs`  
  - `lib/platform/mesh/node/src/plane.rs`  
  - `lib/app/servers/api/src/utils/state.rs`

**Goal:** Make `NodePlaneConfig` + `NodeDataPlane` the **single source** for node-local wiring (compliance, eval datasets, vector, datamart).

**Tasks**

- [x] Confirm `NodePlaneConfig::from_env("app.web.api")` constructs:

  - `Policy` via `ComplianceConfig::from_env()?.load_policy()`.
  - `FileDatasetStore` via `EvalDatasetConfig::from_env().dataset_store()`.
  - `VectorStoreConfig` via `refractive_swan_vector_store::config_from_env()`.
  - `WarehouseConfig` for `SqliteDatamart::from_optional_config`.

- [ ] Document env namespaces for nodes:

  - current: `app.web.api`,
  - optional future: `app.mesh.node.<suffix>`.
  - Documented in `docs/system-design/mesh/node-runtime.md` (env seams section).

- [x] Add `NodePlaneConfig::from_env_with_node_id(namespace: &str, node_id: MeshNodeId)` to allow deterministic IDs.

- [x] Ensure (with tests) `ApiState::from_plane_config` is the *only* state constructor for the node server; no duplicate config logic.

---

#### P1.2 — Mesh node capabilities DTO & `/mesh` API wiring

- **Status:** INPROGRESS  
- **Path:**  
  - Contracts: `lib/domain/meta/contracts/src/mesh.rs` (`NodeCapabilities`, `MeshNodeId`)  
  - Node: `lib/platform/mesh/node/src/plane.rs`  
  - API: `lib/app/servers/api/src/server/handlers`, `routes/v1/mesh.rs`

**Tasks**

- [x] Add `NodeDataPlane::capabilities(&self, base_url: &str) -> NodeCapabilities`:

  - `node_id` from `MeshNodeId`,
  - `vector_backend` from `VectorStoreConfig.backend`,
  - `warehouse_backend` (e.g., `"sqlite"` for now),
  - `compliance_mode` from `policy.mode.as_str()`,
  - `max_dataset_size` (configurable, default reasonable),
  - `tags` (e.g., `["dev"]`, `["research"]`, `["prod"]`).

- [x] Add handlers to `refractive_swan_api`:

  - `GET /mesh/health` -> combine `infra::health` + `metrics_summary` into a health payload.
  - `GET /mesh/capabilities` -> returns `NodeCapabilities`.
  - `GET /mesh/governance` -> returns compliance mode + any DP budget info (when available).

- [x] Add `routes/v1/mesh.rs` and merge into `routes/v1/mod.rs` and top-level `routes::build_router()`.

---

#### P1.3 — Generic mesh job endpoint at node

- **Status:** INPROGRESS  
- **Path:** Contracts: `mesh.rs`; Node: `refractive_swan_mesh_node`; API: mesh job handler & controller.

**Tasks**

- [x] Finalize `MeshJobType` variants:

  - `EvalDataset`
  - `AnalyticsQuery`
  - `MappingHealthCheck`
  - `ExportJob`
  - `NodeIntrospection`

- [x] Implement `NodeDataPlane::run_mesh_job(job: &MeshJobDescriptor) -> MeshJobResult`:

  - `EvalDataset`:
    - parse `{ "dataset": "<name>", "top_k": <usize> }`,
    - call `FileDatasetStore::load_dataset_with_manifest` + `run_eval_with_mapper`,
    - embed `EvalRunResponse` in `MeshJobResult.output`.

  - `AnalyticsQuery`:
    - support `"ncit_summary"` and `"cohort"` with `CohortFilters`,
    - call datamart: `SqliteDatamart::ncit_summary` / `cohort`,
    - return appropriate analytics DTO.

  - `MappingHealthCheck`:
    - run a small pipeline on a regression bundle from `lib/domain/meta/evaluation/data/regression/`,
    - summarize metrics (e.g., `auto_mapped`, `no_match`, errors) into a health JSON, plus vector namespace/latency and datamart persistence status.

  - `ExportJob` (v1):
    - call `ncit_summary` and package into an “export summary” (later: call lake writer); enforce compliance export policy.

  - `NodeIntrospection`:
    - return `PipelineMetrics` snapshot + vector usage + dataset manifests.

- [x] Add `POST /mesh/job` in `refractive_swan_api`:

  - deserialize `MeshJobDescriptor`,
  - call `NodeDataPlane::run_mesh_job`,
  - respond with `MeshJobResult`.

- [ ] Ensure (with tests) `/api/v1/**` behavior is unchanged; reuse domain controllers where possible.

---

#### P1.4 — Node data-plane alignment: mart / warehouse / vector / lake hooks

- **Status:** INPROGRESS  
- **Path:**  
  - `lib/platform/data/data-plane/mart` (`refractive_swan_datamart`)  
  - `lib/platform/data/data-plane/warehouse` (`refractive_swan_datawarehouse` – design)  
  - `lib/platform/data/data-plane/lake` (`refractive_swan_datalake` – design)  
  - `lib/platform/data/data-stores/vector_store` (`refractive_swan_vector_store`)

**Tasks**

- [x] Confirm `SqliteDatamart::from_optional_config(NodePlaneConfig::datamart_config())` is the node’s primary `DatamartSink` (tests cover disabled/enabled warehouse label).

- [x] Ensure (with tests) `VectorPipelineContext` is constructed from:

  - `VectorStoreConfig` via `config_from_env()`,
  - concrete store (`QdrantVectorStore::from_config` or `PgVectorStore::from_config`),
  - `top_k` defaults defined in mapping/vector ranker types.

- [x] Prepare for `ExportJob` integration with `refractive_swan_datalake`:

  - treat `LakeWriter` / `LakeReader` as optional injection points in `NodeDataPlane`,
  - design `WarehouseSnapshot` (`fact_rows`, `dim_rows`) based on `fact_service_request` / dims.

- [x] Document how node roles map to `WarehouseRole::Operational` (mart) vs future `Reporting`/`Archival`.

---

### P2 — Hub runtime crate & reporting warehouse/lake integration

#### P2.1 — Scaffold `refractive_swan_mesh_hub` crate

- **Status:** INPROGRESS  
- **Path:** `lib/platform/mesh/hub/`

**Tasks**

- [x] Add `Cargo.toml` for `refractive_swan_mesh_hub` with deps:

  - `refractive_swan_contracts` (mesh, analytics, eval),
  - `refractive_swan_mesh_governance` (when implemented),
  - `reqwest`, `tokio`, `serde`, `thiserror`.

- [x] Implement `HubConfig::from_env("platform.mesh.hub")`:

  - `hub_id`,
  - HTTP timeout/backoff/concurrency settings,
  - optional discovery config.

- [x] Implement `NodeRegistry`:

  - `NodeMetadata` = `MeshNodeId`, `url`, `NodeCapabilities`, `last_seen`, `NodeStatus`, stats (status defaulted; basic registry in place).

---

#### P2.2 — Hub job queue & node dispatch

- **Status:** INPROGRESS  
- **Path:** `lib/platform/mesh/hub/src/job_queue.rs`, `node_registry.rs`

**Tasks**

- [x] Implement `JobQueue`:

  - `dispatch(job: MeshJobDescriptor, target_nodes: Option<Vec<MeshNodeId>>) -> Vec<MeshJobResult>`,
  - POST `job` to each `"{node.url}/mesh/job"` using `reqwest::Client`.

- [x] Apply `HubConfig`:

  - timeouts, retry/backoff, concurrency (e.g., `futures::stream::buffer_unordered`).

- [x] Track per-node stats in `NodeRegistry` (success/failure counters, last error).

---

#### P2.3 — Federated analytics & eval in hub

- **Status:** INPROGRESS  
- **Path:** `lib/platform/mesh/hub/src/analytics.rs`, `eval.rs`

**Tasks**

- [x] Implement `global_ncit_summary(queue: &JobQueue) -> AnalyticsSummaryResponse`:

  - build `MeshJobDescriptor { job_type: AnalyticsQuery, parameters: { "query_type": "ncit_summary" } }`,
  - dispatch to all online nodes,
  - aggregate `rows` by `(ncit_id, preferred_name, mapping_state, time_bucket)`.

- [x] Implement `federated_eval(queue: &JobQueue, dataset: &str) -> FederatedEvalReport`:

  - dispatch `MeshJobType::EvalDataset` to all nodes,
  - aggregate `EvalSummary` metrics (weighted by `total_cases`) into a federated view,
  - include per-node metrics for debugging.

- [x] Define `FederatedEvalReport` in `refractive_swan_contracts` or hub DTO module.

---

#### P2.4 — Hub-side node registration & health-check loop

- **Status:** TODO  
- **Path:** `lib/platform/mesh/hub/src/registry.rs`, `health.rs`

**Tasks**

- [ ] Implement `register_node(NodeCapabilities)` for nodes that proactively register.

- [ ] Implement `health_check_nodes()`:

  - call `GET /mesh/health` and optionally `/mesh/capabilities`,
  - update `NodeStatus`, `last_seen`, error info.

- [ ] Add `HubRuntime` owning `HubConfig`, `NodeRegistry`, `JobQueue`:

  - methods: `list_nodes`, `global_ncit_summary`, `federated_eval`.

---

#### P2.5 — Hub HTTP surface (API server for frontend)

- **Status:** TODO  
- **Path:**  
  - either new `lib/app/servers/mesh_hub_api`,  
  - or hub-mode router inside `refractive_swan_api`.

**Tasks**

- [ ] Decide deployment: standalone hub API vs `HUB_MODE` env flag in `refractive_swan_api`.

- [ ] Expose endpoints:

  - [x] `GET /hub/nodes` -> list of `NodeMetadata`.
  - [x] `GET /hub/nodes/:id` -> node details + last metrics snapshot.
  - [x] `POST /hub/jobs/analytics/ncit-summary` -> `global_ncit_summary`.
  - [x] `POST /hub/jobs/eval` -> `federated_eval`.

- [ ] Ensure (with tests) hub API uses only official contracts (`refractive_swan_contracts` types).

---

#### P2.6 — Hub reporting warehouse & lake integration (future path)

- **Status:** TODO  
- **Path:**  
  - `lib/platform/data/data-plane/warehouse` (`refractive_swan_datawarehouse` – design)
  - `lib/platform/data/data-plane/lake` (`refractive_swan_datalake` – design)
  - `lib/platform/data/data-stores/relational_store`, `cache_store` (design)

**Tasks**

- [ ] Define minimal concrete `WarehouseAnalytics` for hub:

  - either reuse `SqliteDatamart` with a “reporting” role, or
  - add a separate reporting warehouse backed by `refractive_swan_relational_store`.

- [ ] Implement first `LakeWriter`/`LakeReader` (even just local Parquet files) in line with `refractive_swan_datalake` README.

- [ ] Extend `MeshJobType::ExportJob`:

  - Node: write a snapshot via `LakeWriter` and return `SnapshotMetadata` / `WriteSummary`.
  - Hub: `ingest_node_snapshots` reads snapshots and bulk loads into hub reporting warehouse (subject to governance and DP checks).

- [ ] Optionally allow `global_ncit_summary` to run over:

  - live node datamarts (real-time),
  - hub reporting mart (lake-ingested, for historical/global views).

---

### P3 — Governance & DP budgets

#### P3.1 — Embed governance engine into node mesh jobs

- **Status:** TODO  
- **Path:**  
  - `lib/platform/mesh/governance`  
  - `lib/platform/mesh/node/src/plane.rs` (or separate `jobs.rs`)

**Tasks**

- [ ] Implement `QueryDescriptor::from_job(job: &MeshJobDescriptor)`:

  - map job type + parameters to class/cardinality/time range/requester.

- [ ] Implement `NodePolicy` loading from env:

  - per-node `dp_budget_daily`, `dp_budget_consumed`,
  - `max_cardinality_no_dp`, `export_allowed`, `hub_registration_allowed`.

- [ ] Wrap `NodeDataPlane::run_mesh_job` with `GovernanceEngine`:

  - evaluate descriptor + policy,
  - for `AllowWithNoise { epsilon }`:
    - run job,
    - apply DP to analytics/export outputs via `Policy::apply_dp_noise`,
    - call `consume_budget(epsilon)`.

  - for `Deny(reason)`:
    - return `MeshJobResult` with `status = Denied` and `MeshError { kind = PolicyDenied, code = "policy_denied:...", message: reason }`.

---

#### P3.2 — Hub-side governance for ingestion & export

- **Status:** TODO  
- **Path:** `lib/platform/mesh/hub/src/governance_adapter.rs`

**Tasks**

- [ ] Define hub-level rules:

  - which job types can be run against which nodes (based on `tags`, `compliance_mode`, license tiers),
  - which lake snapshots may be ingested into the reporting warehouse.

- [ ] Implement:

  - `fn allow_federated_job(job: &MeshJobDescriptor, node: &NodeMetadata, node_policy: &NodePolicy) -> bool`.

- [ ] Apply checks in `JobQueue::dispatch`:

  - skip nodes that fail hub policy before HTTP calls,
  - mark per-node `MeshJobResult.error` when the hub denies dispatch.
- [ ] Ensure hub runtime accepts an injected pre-built `Policy`/`NodePolicy` (no env loading inside the hub crate) and add a unit test mirroring the node plane policy coverage.

---

#### P3.3 — Governance + cache + relational store hooks

- **Status:** TODO  
- **Path:**  
  - `lib/platform/data/data-stores/cache_store`  
  - `lib/platform/data/data-stores/relational_store`  
  - `lib/platform/mesh/governance`

**Tasks**

- [ ] Plan `CacheStore` usage:

  - per-node rate-limits (`CacheStore::incr("rate_limit:<node>:<window>", 1)`),
  - optional DP budget “hot cache” (with relational store as durable record).

- [ ] Plan `RelationalStore` adoption:

  - treat current SQLite wiring in `SqliteDatamart` as a precursor to `RelationalBackend::Sqlite`,
  - design how node/hub might switch to Postgres/Duckdb with minimal domain changes.

- [ ] Ensure (with tests) governance error codes align with `ComplianceAction` and license tiers (e.g., `policy_denied:dp_budget_exceeded`, `policy_denied:export_blocked_for_tier_licensed`).

---

### P4 — Frontend mesh dashboard (server ↔ frontend connection)

#### P4.1 — Extend DTOs for web consumption

- **Status:** TODO  
- **Path:** `lib/dto/web`, `lib/dto/mesh` (or directly via `refractive_swan_contracts`)

**Tasks**

- [ ] Add lightweight view types:

  - `NodeView` = subset of `NodeMetadata` + last `MetricsSnapshot`.
  - `FederatedEvalView` from `FederatedEvalReport`.
  - `MeshTogglesView` (extends `/admin/toggles` with mesh & hub info).

- [ ] Ensure (with tests) API responses for:

  - `/admin/toggles`,
  - `/mesh/health`, `/mesh/capabilities`,
  - `/hub/nodes`, `/hub/jobs/*`  
  are aligned with these DTOs.

---

#### P4.2 — API client layer in frontend

- **Status:** TODO  
- **Path:** `lib/app/frontend/web` (e.g. `services/mesh.ts`)

**Tasks**

- [ ] Implement typed clients:

  - `getNodeToggles()`,
  - `getNodeCapabilities()`,
  - `getHubNodes()`,
  - `runGlobalNcitSummary()`,
  - `runFederatedEval(dataset: string)`.

- [ ] Map TS types to Rust DTOs (`refractive_swan_web_dto` or manually maintained types).
- [ ] Add minimal error/loading handling.

---

#### P4.3 — Mesh dashboard UI

- **Status:** TODO  
- **Path:** `lib/app/frontend/web` (e.g. `/mesh`, `/admin/mesh`)

**Tasks**

- [ ] Nodes page:

  - table of nodes: `node_id`, `status`, `last_seen`, `vector_backend`, `warehouse_backend`, `compliance_mode`, `tags`,
  - detail view: `PipelineMetrics`, recent `AdminEvent`s, eval datasets, datamart/health.

- [ ] Hub analytics page:

  - global NCIt summary chart (federated / reporting mart),
  - federated eval: dataset selector + aggregated metrics view.

- [ ] Governance view:

  - per-node DP budgets (if available),
  - recent policy denials (node + hub).

---

### P5 — E2E multi-node/hub validation

#### P5.1 — Local multi-node docker-compose / `cargo-make`

- **Status:** TODO  
- **Path:** `code/ci/`, `docs/runbook/mesh-quickstart.md`

**Tasks**

- [ ] Define dev topology:

  - `node-a`, `node-b`, `hub-1`,
  - env namespaces: `app.web.api.node_a`, `app.web.api.node_b`, `platform.mesh.hub`.

- [ ] Provide:

  - `docker-compose.yml` or `cargo make mesh-dev`:
    - runs 2 node APIs + 1 hub,
    - sets vector/datamart env, seeds eval datasets,
    - exposes mesh/hub endpoints for local UI.

  - quickstart doc showing:

    - mapping at each node,
    - federated NCIt summary,
    - federated eval,
    - at least one governance-denied scenario.

---

#### P5.2 — Integration tests for federated flows

- **Status:** TODO  
- **Path:** `lib/platform/test_suite/tests/integration/mesh/`

**Tasks**

- [ ] Write tests that:

  - spin up two in-process axum routers (via `router_with_state(ApiState)`), each with distinct `NodePlaneConfig` (different datamart/vector configs),
  - simulate a hub:

    - calls `/mesh/capabilities` + `/mesh/job` on both,
    - aggregates analytics and eval outputs into federated reports.

- [ ] Test cases:

  - governance DP-blocking (DP budget exceeded -> `MeshJobStatus::Denied`),
  - NCIt summary aggregation across nodes with different datasets,
  - eval metrics averaging across nodes with different eval tiers.

---

## 5. Definition of done

- **Node:**

  - `/mesh/health`, `/mesh/capabilities`, `/mesh/job` implemented and tested.
  - Existing `/api/v1/**` behavior unchanged.
  - `NodeDataPlane` / `NodePlaneConfig` are the single sources of node-local wiring; no duplicate ad-hoc config code.

- **Hub:**

  - `refractive_swan_mesh_hub` crate exists with `HubConfig`, `NodeRegistry`, `JobQueue`, `global_ncit_summary`, `federated_eval`.
  - A hub HTTP surface exposes `/hub/nodes`, `/hub/jobs/analytics/ncit-summary`, `/hub/jobs/eval`.

- **Data plane:**

  - Node-local datamart (`SqliteDatamart`) receives `PipelineOutput` from pipeline and powers `/analytics/*` and mesh jobs.
  - Vector backends are wired through `VectorStoreConfig` + `VectorPipelineContext`.
  - Lake/warehouse traits are defined and stubbed with a clear migration path for reporting marts.

- **Governance:**

  - All mesh jobs at a node pass through `GovernanceEngine` with `Policy` + `NodePolicy`.
  - DP budgets and license-tier rules are enforced for analytics/export; denials are visible in `MeshJobResult.error` and admin views.

- **Frontend:**

  - Mesh dashboard presents node list, node detail (metrics/events/datasets), federated analytics and eval views, and governance decisions.
  - Frontend uses only documented HTTP APIs (`/api/v1/**`, `/mesh/**`, `/hub/**`).

- **E2E:**

  - A documented dev setup (2 nodes + 1 hub) runs end-to-end:

    - local mapping at each node,
    - federated NCIt summary,
    - federated eval,
    - at least one governance-denied job.

---
