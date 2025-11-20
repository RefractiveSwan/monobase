# Kanban – feature/mesh-data-plane (05x)

**Theme:** Post-refactor data/mesh architecture – node runtime, mesh governance, platform data & store layers  
**Branch:** `feature/meta/MESH-05x-mesh-data-plane`  
**Goal:** Design (and then iteratively implement) a **mesh-first** data system where each deployment is a sovereign **node runtime** with a clear separation of **domain**, **platform data**, **platform stores**, and **mesh orchestration**, evolving the codebase toward:

```text
lib/
  domain/
    core/
    mapping/
    pipeline/          (dfps_pipeline)
    eval/
    contracts/         (dfps_contracts: shared DTOs)
  platform/
    mesh/ 
      node/            (dfps_mesh_node)       <- node runtime & governance integration
      hub/             (dfps_mesh_hub)        <- optional research/orchestrator / FL coordinator
      governance/      (dfps_mesh_governance) <- mesh-level policies, DP/query model, node descriptors
    data/
      mart/            (dfps_datamart)        <- dim/fact logic inside node
      warehouse/       (dfps_datawarehouse)   <- backend-agnostic relational warehouse traits
      lake/            (dfps_datalake)        <- local snapshots / Parquet/Delta lake inside node
    store/
      relational_store (dfps_relational_store) <- SQLx/Postgres/DuckDB drivers, per node
      vector_store     (dfps_vector_store)     <- Qdrant/PGVector, per node
      cache_store      (dfps_cache_store)      <- Redis, per node
      graph_store      (dfps_graph_store)      <- IndraDB / graph store, per node
```

> **Assumptions:** REFR-03 (contracts), REFR-09 (pipeline), REFR-13 (vector_store), and REFR-16 (HTTP+warehouse) are at least functionally complete: domain crates are env-free, vector backends are centralized, and analytics are warehouse-backed.

---

## Columns

* **TODO** – Not started yet; design + spike work lives here until first PRs.
* **INPROGRESS** – Design or implementation underway.
* **REVIEW** – Awaiting review; may include design docs and PoCs.
* **DONE** – Shipped into `main` (or equivalent).

---

## TODO

### MESH-01 – Domain surface & contracts alignment for mesh

**Goal:** Lock in a **node-neutral domain layer** (`lib/domain/**`) and **contract layer** (`dfps_contracts`) that can be reused by multiple node runtimes (hospital node, research test harness, mesh hub) without leaking transport or infra details.

**Scope:** `lib/domain/core`, `lib/domain/ontologies/mapping`, `lib/domain/pipeline`, `lib/domain/eval`, `lib/domain/contracts` (new), `dfps_terminology`

* [ ] **MESH-01A – Domain tree stabilization**

  * [ ] Create a domain-level **index document** (e.g., `docs/system-design/base/domain-layout.md`) that states the target domain tree:

    ```text
    lib/domain
      core/         (dfps_core)
      mapping/      (dfps_mapping)
      pipeline/     (dfps_pipeline)
      eval/         (dfps_eval)
      contracts/    (dfps_contracts)
      terminology/  (dfps_terminology) (optional alias; current path remains ontologies/terminology)
    ```

  * [ ] Decide whether to **physically move** `lib/domain/ontologies/mapping` → `lib/domain/mapping` in this phase or:

    * [ ] Introduce a thin alias crate `lib/domain/mapping` that re-exports the existing `dfps_mapping` crate (keeping `Cargo.toml` package names stable).
    * [ ] Do the same for terminology (`dfps_terminology`), if moving: `lib/domain/ontologies/terminology` → `lib/domain/terminology`.

  * [ ] Update `dfps_core::lib.rs` and `prelude.rs` to:

    * [ ] Treat `mapping`, `interop`, `clinical`, `primitives` as stable, self-contained domains.
    * [ ] Point docs to the new “mesh-aware” design doc.

* [ ] **MESH-01B – Contracts as mesh boundary**

  * [ ] Extend `dfps_contracts` (introduced in REFR-03) to be explicitly **mesh-aware**:

    * [ ] Label contracts that cross **node ↔ hub** boundaries (e.g., `NodeCapabilities`, `JobDescriptor`, `JobResult`).
    * [ ] Distinguish between **intra-node** contracts (e.g., `PipelineOutput`, `LoadSummary`) and **inter-node** contracts (FL jobs, aggregate responses).

  * [ ] Add a `contracts::mesh` module containing:

    * [ ] `NodeId`, `NodeMetadata` (e.g., capabilities, compliance mode, license tier mix, store backends).
    * [ ] `MeshJobDescriptor` (job type, input parameters, required capabilities).
    * [ ] `MeshJobResult` (aggregate metrics, error summary, optional DP metadata).
    * [ ] `MeshErrorKind` / `MeshErrorCode` for hub ↔ node interactions, mapping back to local error kinds.

  * [ ] Ensure these mesh contracts **do not depend** on concrete HTTP/transport (no Axum types, no reqwest types).

* [ ] **MESH-01C – Node-agnostic eval & mapping interfaces**

  * [ ] Define in `dfps_contracts` or `dfps_eval`:

    * [ ] A **node-agnostic** eval request/response shape:

      * `EvalNodeRequest` (dataset, top_k, mapping mode flags).
      * `EvalNodeResponse` (EvalSummary + metadata: mapping version, vector backend).

    * [ ] A **hub-level** eval aggregation contract:

      * `EvalMeshAggregate` (per-node stats, mesh-wide confidence intervals, outlier flags).

  * [ ] Ensure `dfps_mapping::MappingSummary` and `dfps_eval::EvalSummary` have **versioned** fields referenced from contracts and are safe for cross-node serialization.

---

### MESH-02 – Platform store layer (relational/vector/cache/graph)

**Goal:** Extract and standardize **store abstractions** under `lib/platform/store/**` so that mesh nodes and hub share the same primitives for relational, vector, cache, and graph storage.

**Scope:** new crates under `lib/platform/store`, plus migration from current `dfps_datamart` and `dfps_vector_store`.

* [ ] **MESH-02A – Relational store abstraction (`dfps_relational_store`)**

  * [ ] Create `lib/platform/store/relational_store`:

    * [ ] `RelationalBackend` enum (`Sqlite`, `Postgres`, `Duckdb`, `External`).
    * [ ] `RelationalConfig { url, pool_max, connect_timeout, schema }` driven entirely by `dfps_configuration`.
    * [ ] Traits:

      * `RelationalPool` (abstract pool handle).
      * `RelationalMigrator` (run migrations + DDL).
      * Optional `Warehouse` trait moved from `dfps_datamart::WarehouseConfig`/`load_from_pipeline_output`.

  * [ ] Extract SQLx-specific logic from `dfps_datamart::sql` into `dfps_relational_store`:

    * [ ] Provide helper functions for acquiring `Pool<Sqlite>` / `Pool<Postgres>` with consistent error reporting.
    * [ ] Keep DDL in datamart but use **relational pool** creation & connection here.

* [ ] **MESH-02B – Vector store as platform store (`dfps_vector_store`)**

  * [ ] Treat `dfps_vector_store` as `lib/platform/store/vector_store` conceptually; plan physical move when refactor is stable.

  * [ ] Ensure:

    * [ ] No direct reference to Axum or any HTTP-specific code.
    * [ ] All env config uses `dfps_configuration::load_env("platform.store.vector")` or similar namespaced keys.
    * [ ] Provide non-env constructors (`VectorStoreConfig::from_parts`) for hub orchestration or test harnesses.

* [ ] **MESH-02C – Cache store abstraction (`dfps_cache_store`)**

  * [ ] Introduce `lib/platform/store/cache_store` (initially minimal):

    * [ ] Define `CacheStore` trait with operations like `get`, `set`, `delete`, `incr`, `ttl`.
    * [ ] Implement a `RedisCacheStore` backend behind a feature flag (no runtime required yet).
    * [ ] Provide a `CacheConfig` with `url`, `pool_max`, `namespace`, `strict` flags.

  * [ ] Use case in mesh:

    * [ ] Node-level caching of expensive analytics or eval results.
    * [ ] Hub-level caching of node capability snapshots.

* [ ] **MESH-02D – Graph store abstraction (`dfps_graph_store`)**

  * [ ] Introduce `lib/platform/store/graph_store`:

    * [ ] Define `GraphStore` trait (nodes/edges CRUD, neighborhood queries, simple pattern queries).
    * [ ] Provide a `GraphConfig` (backend, url, namespace).
    * [ ] Implement first backend as an in-memory or file-backed stub, with an eye toward IndraDB (or equivalent) later.

  * [ ] Use case in mesh:

    * [ ] Node-level representation of ontology graphs (NCIT, MONDO), mapping neighborhoods, and concept co-occurrence.
    * [ ] Potential hub-level cross-node link graph (which nodes have which code/ontology capabilities).

---

### MESH-03 – Platform data layer (mart/warehouse/lake)

**Goal:** Separate **dim/fact logic** (datamart) from **warehouse backend implementation** and **lake storage**, so each node can choose local storage patterns but still speak the same “data product” language.

**Scope:** `lib/platform/data/mart`, `lib/platform/data/warehouse`, `lib/platform/data/lake`, refactor of `dfps_datamart`

* [ ] **MESH-03A – Mart crate (`dfps_datamart` relocation & redesign)**

  * [ ] Move `lib/app/servers/datamart` to `lib/platform/data/mart` conceptually:

    * [ ] Keep `Dim*` structs and `FactServiceRequest` as the canonical **mart schema**, not tied to SQLite specifically.
    * [ ] Expose a `Mart` API that:

      * [ ] Accepts `PipelineOutput` and a `RelationalPool` (from `dfps_relational_store`).
      * [ ] Produces `LoadSummary` + potential row-level errors.
      * [ ] Provides analytic queries (`ncit_summary`, `cohort`) that work against any relational backend.

  * [ ] Create a sub-module `mart/sqlite` that wires the current SQLx SQLite logic as **one** Mart backend implementation.

* [ ] **MESH-03B – Warehouse crate (`dfps_datawarehouse`)**

  * [ ] Introduce `lib/platform/data/warehouse`:

    * [ ] `WarehouseRole` concept (operational mart, reporting mart, long-term archive).
    * [ ] `WarehouseConfig` which composes `RelationalConfig` + `role` + optional `lake` references.
    * [ ] Traits:

      * `WarehouseLoader` – entrypoints for `PipelineOutput` → dims/facts (delegate to `dfps_datamart`).
      * `WarehouseAnalytics` – analytic queries over dim/fact (delegate to SQL or dbt-based models).

  * [ ] `dfps_mesh_node` and `dfps_mesh_hub` will use `dfps_datawarehouse` as the primary interface for data persistence.

* [ ] **MESH-03C – Lake crate (`dfps_datalake`)**

  * [ ] Introduce `lib/platform/data/lake`:

    * [ ] `LakeConfig` (root path, format = Parquet/Delta, retention policy, partitioning rules).
    * [ ] `LakeWriter` trait (append snapshots from dims/facts).
    * [ ] `LakeReader` trait (load snapshots for offline analytics or hub queries).

  * [ ] Define first use cases:

    * [ ] Node-level periodic snapshots of fact_service_request for offline replays.
    * [ ] Hub-level ingestion of node snapshot exports (if policy allows).

---

### MESH-04 – Mesh node runtime & governance (`dfps_mesh_node`, `dfps_mesh_governance`)

**Goal:** Turn the existing `dfps_api` into a **mesh node runtime** (`dfps_mesh_node`) wired through a `NodeDataPlane`, with a dedicated `dfps_mesh_governance` crate representing **query-level** and **FL job-level** policies (distinct from license/export-only `dfps_compliance`).

**Scope:** `lib/platform/mesh/node`, `lib/platform/mesh/governance`, `dfps_api` refactor

* [ ] **MESH-04A – NodeDataPlane design**

  * [ ] Draft a design doc `docs/system-design/base/node-runtime.md` that defines:

    * [ ] `NodeDataPlane` struct responsibilities:

      * Owns **stores**: relational, vector, cache, graph.
      * Owns **data**: local warehouse/mart/lake.
      * Owns **domain services**: ingestion, pipeline, mapping, eval.
      * Owns **policies**: compliance policy (`dfps_compliance::Policy`) + mesh governance policy (below).
      * Exposes **ports**:

        * `run_mapping_job(bundle_batch)` → `PipelineOutput + LoadSummary`.
        * `run_eval_job(EvalNodeRequest)` → `EvalNodeResponse`.
        * `run_analytics_job(query)` → `AnalyticsSummaryResponse`/`CohortResponse`.

  * [ ] Identify which pieces of `ApiState` move into `NodeDataPlane` vs remain as HTTP-adapter concerns.

* [ ] **MESH-04B – Governance crate (`dfps_mesh_governance`)**

  * [ ] Introduce `lib/platform/mesh/governance`:

    * [ ] Model **query-level policy**:

      * `QueryClass` (mapping, eval, analytics, export, federated training, node status).
      * `QueryPolicy` (allowed parameters, DP budget constraints, max cardinality, allowed time ranges).

    * [ ] Model **node descriptor**:

      * `NodeCapabilities` (supported store backends, vector capacity, compliance mode, max dataset size, max eval load).
      * `NodePolicy` (local overrides, e.g., “allow eval only from these hub IDs”).

    * [ ] Provide evaluation functions:

      * `authorize_query(node_policy, query_descriptor) -> GovernanceDecision` (Allow/Deny/AllowWithDPNoise).
      * `track_budget(node_state, decision)` for DP budget enforcement (hooks only; implementation can be incremental).

  * [ ] Integrate governance decision points:

    * [ ] In `map_bundles` handler: classify job as `QueryClass::Mapping`, ask governance before proceeding.
    * [ ] In analytics/eval handlers: enforce `QueryClass::Analytics/<Eval>` limits (e.g., no “unbounded cohort with ID filter” if not allowed).

* [ ] **MESH-04C – Rename/refactor `dfps_api` → `dfps_mesh_node`**

  * [ ] Introduce `lib/platform/mesh/node` (package: `dfps_mesh_node`) that:

    * [ ] Owns `NodeDataPlane` struct and all Axum handlers.
    * [ ] Depends on:

      * Domain: `dfps_core`, `dfps_ingestion`, `dfps_mapping`, `dfps_pipeline`, `dfps_eval`.
      * Platform: `dfps_datawarehouse`, `dfps_datamart`, `dfps_datalake`, `dfps_relational_store`, `dfps_vector_store`, `dfps_compliance`, `dfps_mesh_governance`, `dfps_configuration`, `dfps_observability`.

  * [ ] Keep `dfps_api` as a **shim crate** during migration:

    * [ ] Re-export `dfps_mesh_node::run`, `ApiServerConfig`, DTOs.
    * [ ] Deprecate `dfps_api` in docs, pointing to `dfps_mesh_node`.

---

### MESH-05 – Mesh hub (`dfps_mesh_hub`)

**Goal:** Define a minimal **hub runtime** that can coordinate federated jobs across multiple nodes using the `dfps_contracts::mesh` contracts and governance.

**Scope:** `lib/platform/mesh/hub`, hub-facing node APIs, FL job model

* [ ] **MESH-05A – Hub domain model**

  * [ ] Introduce `lib/platform/mesh/hub` crate:

    * [ ] `HubConfig` – known nodes, authentication model, timeouts, backoff.
    * [ ] `NodeRegistry` – in-memory or pluggable store of `NodeId` → `NodeMetadata` (from contracts).
    * [ ] `JobQueue` – ephemeral representation of “jobs to run on nodes”, with states (Pending, Running, Completed, Failed).

  * [ ] Define **job types**:

    * `MappingEvalJob` (invoke eval on multiple nodes, aggregate results).
    * `AnalyticsScanJob` (NCIT summary across nodes for a constrained cohort).
    * Future: `ModelTrainingJob` (federated training tasks).

* [ ] **MESH-05B – Node ↔ hub protocol (HTTP-level mapping)**

  * [ ] Decide on HTTP API:

    * Node provides:

      * `POST /mesh/jobs/run` or reuses existing endpoints (`/api/eval/run`, `/analytics/*`) with additional query params specifying “hub job id”.
      * `GET /mesh/nodes/capabilities` or reuse `/metrics/summary` enriched with `NodeCapabilities`.

  * [ ] Map `dfps_contracts::MeshJobDescriptor` and `MeshJobResult` onto HTTP routes:

    * Hub constructs `MeshJobDescriptor`, posts to nodes or maps them to existing API endpoints.
    * Nodes package their result in `MeshJobResult` payloads.

* [ ] **MESH-05C – Hub governance integration**

  * [ ] Use `dfps_mesh_governance` to:

    * [ ] Decide which nodes can be targeted for a given job (based on `NodePolicy` + `NodeCapabilities`).
    * [ ] Enforce global budgets (e.g., not hitting the same node with too many jobs in a window).

---

### MESH-06 – Migration, compatibility & docs

**Goal:** Provide a smooth migration path from pre-mesh to post-mesh architecture and document the mental model so future work doesn’t drift.

**Scope:** docs, alias crates, runbooks, diagrams

* [ ] **MESH-06A – Compatibility shims**

  * [ ] Keep `dfps_api` crate as deprecated alias for `dfps_mesh_node` until major version bump.
  * [ ] Provide alias crates or module paths if physical relocations occur (e.g., `dfps_datamart` → `platform/data/mart`).
  * [ ] Provide `dfps_vector_store` re-exports under `platform/store/vector_store` if/when you move it.

* [ ] **MESH-06B – Docs & diagrams**

  * [ ] Add `docs/system-design/mesh/node-and-hub-architecture.md`:

    * [ ] Node runtime: diagram of `NodeDataPlane` with domain/platform/store crates.
    * [ ] Hub runtime: diagram of `MeshHub` interacting with nodes.
    * [ ] Governance flows: request path → `dfps_mesh_governance` → `dfps_compliance` → DataPlane.

  * [ ] Add mermaid/Graphviz diagrams showing:

    * [ ] New lib tree (`domain`, `platform/data`, `platform/store`, `platform/mesh`).
    * [ ] Job lifecycles: mapping, analytics, eval, federated eval.

* [ ] **MESH-06C – Runbooks**

  * [ ] Write `docs/runbook/mesh-node-quickstart.md`:

    * [ ] How to run a single node with SQLite + Qdrant or PGVector.
    * [ ] How to configure compliance + governance via env and policy files.
    * [ ] Steps for switching between “standalone” and “mesh-controlled” modes.

  * [ ] Write `docs/runbook/mesh-hub-quickstart.md` (after MESH-05):

    * [ ] How to configure a hub to talk to multiple nodes.
    * [ ] Example `MappingEvalJob` and how results are aggregated.

---

## INPROGRESS

* *Empty* (this board is post-refactor planning; populate as you start work).

---

## REVIEW

* *Empty*

---

## DONE

* *Empty* (intentionally; this kanban tracks **post-REFR** work only)
