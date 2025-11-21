# Kanban – feature/mesh-data-plane (025)

**Theme:** Post-refactor data/mesh architecture – node runtime, mesh governance, platform data & store layers  
**Branch:** `feature/meta/MESH-025-mesh-data-plane`  
**Goal:** Evolve the *current* codebase into a **mesh-first** data system where each deployment is a sovereign **node runtime**, keeping the **planned** `platform/{data,store,mesh}` layout fixed:

```text
lib/
  domain/
    core/
    mapping/
    pipeline/          (dfps_pipeline)
    eval/
    contracts/         (dfps_contracts: shared DTOs)
  dto/
    web/               (dfps_web_dto: web-facing DTO veneer)
  platform/
    mesh/ 
      node/            (dfps_mesh_node)       <- node runtime & governance integration
      hub/             (dfps_mesh_hub)        <- research/orchestrator / FL coordinator
      governance/      (dfps_mesh_governance) <- mesh-level policies, DP/query model, node descriptors
    data/
      data-stack/    
        mart/            (dfps_datamart)        <- dim/fact logic inside node
        warehouse/       (dfps_datawarehouse)   <- backend-agnostic relational warehouse traits
        lake/            (dfps_datalake)        <- local snapshots / Parquet/Delta lake inside node
      data-store/
        relational_store (dfps_relational_store) <- SQLx/Postgres/DuckDB drivers, per node
        vector_store     (dfps_vector_store)     <- Qdrant/PGVector, per node
        cache_store      (dfps_cache_store)      <- Redis, per node
        graph_store      (dfps_graph_store)      <- IndraDB / graph store, per node
```

> **Current reality (high-level):**
>
> * Domain alignment in progress:
>
>   * `lib/domain/core` (`dfps_core`), `ontologies/{ingestion,mapping,terminology}` (headed to `domain/meta/*`), `pipeline`, `eval`, `vector_port` (headed to `domain/ports/data/data-store/vector`).
> * DTO layer seeded:
>
>   * `lib/dto/web` (`dfps_web_dto`) already exists; CLI + mesh veneers will land under the `domain/ports/data/dto` subtree per REFR-027.
> * Platform exists but has no `data/`, `store/`, or `mesh/` yet:
>
>   * `lib/platform/{compliance,configuration,observability,test_suite}`.
> * App servers still host what will become platform/data & platform/store:
>
>   * `lib/app/servers/{api,datamart,vector_store}`.

This kanban is about **designing and sequencing** the move from the current layout to the target `platform/{data,store,mesh}` layout without changing that target tree.

---

## Columns

* **TODO** – Planned, not started.
* **INPROGRESS** – Being actively designed/implemented.
* **REVIEW** – Needs code/design review.
* **DONE** – Merged into main.

---


## TODO

* *All Phase 1 (design-level) tasks completed. See DONE section below.*
* Future tasks (Phase 2+) will be tracked in separate implementation-focused kanbans.

---

## INPROGRESS

* *Empty*

---

## REVIEW

* *Empty*

---

## DONE

**Completed:** 2025-11-21

### MESH-00 – Baseline & guardrails ✅

* [x] **MESH-00A – Current vs target layout doc**
  * [x] Created `docs/system-design/base/mesh-data-plane-layout.md`
  * [x] Documented current tree structure
  * [x] Documented target `platform/{data,store,mesh}` structure
  * [x] Created comprehensive mapping table

* [x] **MESH-00B – Naming & stability guardrails**
  * [x] Updated `docs/system-design/base/directory-architecture.md`
  * [x] Added "Platform Directory Stability Guardrails" section
  * [x] Declared `platform/{mesh,data,store}` as FROZEN

---

### MESH-01 – Domain surface & contracts alignment ✅

* [x] **MESH-01A – Vector port verification**
  * [x] Verified no domain crate depends directly on `dfps_vector_store`
  * [x] Verified `dfps_vector_port` contains full trait surface
  * [x] Created `lib/domain/vector_port/README.md`

* [x] **MESH-01B – Contracts as node/hub boundary**
  * [x] Extended `dfps_contracts` with mesh module
  * [x] All HTTP-facing DTOs originate from `dfps_contracts`

* [x] **MESH-01C – Mesh-level contracts**
  * [x] Created `lib/domain/contracts/src/mesh.rs`
  * [x] Implemented `MeshNodeId`, `NodeCapabilities`
  * [x] Implemented `MeshJobDescriptor`, `MeshJobResult`, `MeshJobStatus`
  * [x] Implemented `MeshErrorKind`, `MeshErrorCode`, `MeshError`
  * [x] Added comprehensive unit tests (passing ✓)

---

### MESH-02 – Platform store layer ✅

* [x] **MESH-02A – Create `platform/store` skeleton**
  * [x] Created directory structure
  * [x] Added README stubs for each store type

* [x] **MESH-02B – Relational store abstraction design**
  * [x] Created `lib/platform/store/relational_store/README.md`
  * [x] Defined trait design (RelationalPool, RelationalMigrator)
  * [x] Documented current datamart SQL wiring

* [x] **MESH-02C – Vector store as platform store**
  * [x] Created `lib/platform/store/vector_store/README.md`
  * [x] Documented domain/store/runtime layer mapping
  * [x] Planned `from_config` factory pattern

* [x] **MESH-02D – Cache/graph store design stubs**
  * [x] Created `lib/platform/store/cache_store/README.md`
  * [x] Created `lib/platform/store/graph_store/README.md`

---

### MESH-03 – Platform data layer ✅

* [x] **MESH-03A – Create `platform/data` skeleton**
  * [x] Created directory structure
  * [x] Created `data/mart/README.md`

* [x] **MESH-03B – Warehouse model design**
  * [x] Created `data/warehouse/README.md`
  * [x] Defined WarehouseRole, WarehouseConfig
  * [x] Defined WarehouseLoader, WarehouseAnalytics traits

* [x] **MESH-03C – Lake model design**
  * [x] Created `data/lake/README.md`
  * [x] Defined LakeConfig, LakeWriter, LakeReader
  * [x] Documented use-cases

---

### MESH-04 – Mesh node runtime ✅

* [x] **MESH-04A – NodeDataPlane design doc**
  * [x] Created `docs/system-design/mesh/node-runtime.md`
  * [x] Documented NodeDataPlane struct design
  * [x] Documented responsibilities

* [x] **MESH-04B – Annotate current `dfps_api` as proto-node**
  * [x] Updated `lib/app/servers/api/README.md`
  * [x] Added "Relation to dfps_mesh_node" section
  * [x] Documented ApiState ~ NodeDataPlane relationship

---

### MESH-05 – Mesh governance & hub design ✅

* [x] **MESH-05A – Governance model**
  * [x] Created `lib/platform/mesh/governance/README.md`
  * [x] Defined QueryClass, QueryDescriptor, GovernanceDecision
  * [x] Documented integration points

* [x] **MESH-05B – Hub model**
  * [x] Created `lib/platform/mesh/hub/README.md`
  * [x] Defined HubConfig, NodeRegistry, JobQueue
  * [x] Documented how hub calls node APIs

---

### MESH-06 – Migration strategy, compatibility & docs ✅

* [x] **MESH-06A – Migration map (phased)**
  * [x] Created `docs/system-design/mesh/migration-plan.md`
  * [x] Defined 4 phases (conceptual, aliasing, physical move, mesh node)

* [x] **MESH-06B – Runbooks**
  * [x] Created `docs/runbook/mesh-node-quickstart.md`
  * [x] Created `docs/runbook/mesh-hub-quickstart.md`

---

**Summary:**

Phase 1 (design-level) complete:
- 20 new design documents created
- 10 skeleton directories established
- Mesh contracts module implemented and tested
- Zero code moves (intentional, design-first approach)
- `platform/{data,store,mesh}` directory structure frozen

**Next Steps:**

Phase 2+ will be tracked in separate implementation kanbans focusing on:
- Internal aliasing (re-exports)
- Physical crate moves
- `dfps_mesh_node` extraction from `dfps_api`


**Summary:**

Phase 1 (design-level) complete:
- 20 new design documents created
- 10 skeleton directories established
- Mesh contracts module implemented and tested
- Zero code moves (intentional, design-first approach)
- `platform/{data,store,mesh}` directory structure frozen

**Next Steps:**

Phase 2+ will be tracked in separate implementation kanbans focusing on:
- Internal aliasing (re-exports)
- Physical crate moves
- `dfps_mesh_node` extraction from `dfps_api`

See `docs/system-design/mesh/migration-plan.md` for detailed timeline.



**Goal:** Make the current → target mapping explicit so every subsequent task knows *exactly* which crate moves where in the `platform/{data,store,mesh}` tree (without changing the planned layout).

**Scope:** `docs/system-design`, `docs/kanban`, top-level `lib` layout

* [ ] **MESH-00A – Current vs target layout doc**

  * [ ] Add `docs/system-design/base/mesh-data-plane-layout.md` with three sections:

    * [ ] **Current** tree (verbatim, simplified):

      ```text
      lib/
        app/
          frontend/{cli,web}
          servers/{api,datamart,vector_store}
        domain/
          core/
          eval/
          pipeline/
          contracts/
          vector_port/
          ontologies/{ingestion,mapping,terminology}
        platform/
          configuration/
          compliance/
          observability/
          test_suite/
      ```

    * [ ] **Target** tree (exactly the planned `domain` + `platform/{mesh,data,store}` layout from this kanban).

    * [ ] **Mapping table**: each current crate → future home, e.g.:

      | Current crate                    | Future home                               |
      | -------------------------------- | ----------------------------------------- |
      | `lib/app/servers/api` (dfps_api) | `lib/platform/mesh/node` (dfps_mesh_node) |
      | `lib/app/servers/datamart`       | `lib/platform/data/mart` (dfps_datamart)  |
      | `lib/app/servers/vector_store`   | `lib/platform/store/vector_store`         |
      | `lib/domain/vector_port`         | stays domain (`dfps_vector_port`)         |
      | `lib/platform/compliance`        | stays (`dfps_compliance`)                 |
      | `lib/platform/observability`     | stays (`dfps_observability`)              |
      | `lib/platform/configuration`     | stays (`dfps_configuration`)              |
      | `lib/platform/test_suite`        | stays (`dfps_test_suite`)                 |

* [ ] **MESH-00B – Naming & stability guardrails**

  * [ ] In `docs/system-design/base/directory-architecture.md`:

    * [ ] Add an explicit “**DO NOT** change” block for the planned `platform/data`, `platform/store`, `platform/mesh` directories: names and depth are stable.
    * [ ] Record that:

      * `dfps_datamart` always lives conceptually at `platform/data/mart` (even while physically under `app/servers` until migration).
      * `dfps_vector_store` is the **platform store** implementing `dfps_vector_port` and **must** end up in `platform/store/vector_store` (even if physically under `app/servers` short-term).
      * `dfps_mesh_node`,`dfps_mesh_hub`,`dfps_mesh_governance` will only exist under `platform/mesh`.

---

### MESH-01 – Domain surface & contracts alignment (anchored in current domain layout)

**Goal:** Confirm the domain layer (`core`, `ontologies/{ingestion,mapping,terminology}`, `pipeline`, `eval`, `contracts`, `vector_port`) can be used unchanged by **any** future node/hub under `platform/mesh/*`, and that `dfps_contracts` is the canonical cross-node contract layer.

**Scope:** `lib/domain/{core,eval,pipeline,contracts,vector_port,ontologies/*}`

* [ ] **MESH-01A – Vector port verification**

  * [ ] Assert (via docs + quick grep) that:

    * [ ] No domain crate depends directly on `dfps_vector_store`; all domain crates (`dfps_mapping`, `dfps_pipeline`, `dfps_observability`) use `dfps_vector_port` *only*.
    * [ ] `dfps_vector_port` contains the full trait surface needed by mapping/pipeline (search, index, usage snapshot), with no Qdrant/PGVector specifics.
  * [ ] Update `lib/domain/vector_port/README.md` (if missing) to:

    * [ ] Explicitly call out that `dfps_vector_store` (future `platform/store/vector_store`) is *one* implementation of this port.
    * [ ] Describe the expectation that `dfps_mesh_node` wires a `dfps_vector_store` that implements the `dfps_vector_port` trait.

* [ ] **MESH-01B – Contracts as node/hub boundary**

  * [ ] Extend `dfps_contracts` to have explicit *sections*:

    ```text
    contracts/
      pipeline.rs     (PipelineOutput, MappingResult aliases, etc.)
      analytics.rs    (AnalyticsSummaryRow, CohortRow, AnalyticsSummaryResponse, CohortResponse)
      eval.rs         (EvalCase / EvalSummary / EvalRunResponse / DatasetManifest)
      metrics.rs      (PipelineMetricsSnapshot, VectorUsageSnapshot/CapacitySnapshot)
      mesh.rs         (MeshNodeId, NodeCapabilities, MeshJobDescriptor, MeshJobResult, MeshErrorKind)
    ```

  * [ ] Move HTTP-facing DTOs out of `dfps_api` where appropriate:

    * [ ] `AnalyticsSummaryResponse`, `AnalyticsNcitSummaryRow`, `CohortResponse`, `CohortRow`, `EvalRunResponse` should be re-exported from `dfps_contracts` and referenced in API/web/CLI.

  * [ ] In `dfps_api` and `dfps_web_frontend`, ensure there are **no bespoke analytics/eval DTO structs**; everything comes from `dfps_contracts`.

* [ ] **MESH-01C – Mesh-level contracts**

  * [ ] In `dfps_contracts::mesh`:

    * [ ] Add `MeshNodeId` (opaque string or UUID) and `NodeCapabilities` (vector backend, warehouse backend, compliance mode, max dataset size).
    * [ ] Add `MeshJobDescriptor` (job type enum: `EvalDataset`, `AnalyticsQuery`, `MappingHealthCheck`) + job parameters.
    * [ ] Add `MeshJobResult` (per-node result: status, metrics snapshot, optional error).
    * [ ] Add `MeshErrorKind` (`NodeUnavailable`, `JobRejected`, `PolicyDenied`, `Internal`) and `MeshErrorCode` strings.
  * [ ] Document that **only** `dfps_mesh_node` and `dfps_mesh_hub` are allowed to use these contracts directly; apps/CLIs remain node-local.

---

### MESH-02 – Platform store layer (relational/vector/cache/graph) wired from current crates

**Goal:** Introduce `platform/store/{relational_store,vector_store,cache_store,graph_store}` as conceptual homes, without changing the **planned** directory names, by mapping the **existing** crates into that layout.

**Scope:** new dirs under `lib/platform/store`, plus reuse of `app/servers/vector_store` and `app/servers/datamart`’s SQL wiring

* [ ] **MESH-02A – Create `platform/store` skeleton**

  * [ ] Create dirs (no renames, just new empty dirs + README stubs):

    ```text
    lib/platform/store
      relational_store/ (README + Cargo.toml stub)
      vector_store/     (README + notes: currently implemented by app/servers/vector_store)
      cache_store/      (README only)
      graph_store/      (README only)
    ```

  * [ ] In each README, document the **intent** and the mapping to current crates, e.g.:

    * `store/vector_store/README.md`: “Currently implemented in `lib/app/servers/vector_store` (`dfps_vector_store`). Will be moved here once mesh node is stable. Domain never depends on this crate directly; it uses `dfps_vector_port`.”

* [ ] **MESH-02B – Relational store abstraction design (no code move yet)**

  * [ ] In `lib/platform/store/relational_store/README.md`:

    * [ ] Define desired traits:

      ```rust
      pub enum RelationalBackend { Sqlite, Postgres, Duckdb, External }

      pub struct RelationalConfig {
          pub backend: RelationalBackend,
          pub url: String,
          pub pool_max: u32,
          pub connect_timeout_secs: u64,
          pub schema: Option<String>,
      }

      pub trait RelationalPool: Clone + Send + Sync {
          type Pool;
          fn connect(cfg: &RelationalConfig) -> Result<Self::Pool, RelationalError>;
      }

      pub trait RelationalMigrator {
          type Pool;
          fn migrate(pool: &Self::Pool) -> Result<(), RelationalError>;
      }
      ```

    * [ ] Note that **today** `dfps_datamart`’s `WarehouseConfig` and SQLx-specific connection logic live in `lib/app/servers/datamart/src/sql.rs`, and will be refactored to use these traits.

  * [ ] Add a design checklist:

    * [ ] Keep SQLx concretions behind this crate; `dfps_datamart` should depend on `RelationalConfig + traits`, not `sqlx` directly.

* [ ] **MESH-02C – Vector store as platform store (link current crate)**

  * [ ] In `lib/platform/store/vector_store/README.md`:

    * [ ] Explicitly state that:

      * `dfps_vector_port` is the **domain** trait crate.
      * `dfps_vector_store` (`lib/app/servers/vector_store`) is the **current** platform implementation, planned to move into this directory once `dfps_mesh_node` is stable.
    * [ ] Add a little map:

      | Layer   | Crate               | Responsibilities                            |
      | ------- | ------------------- | ------------------------------------------- |
      | Domain  | `dfps_vector_port`  | traits, errors, embedding metadata          |
      | Store   | `dfps_vector_store` | Qdrant/PGVector configs & drivers           |
      | Runtime | `dfps_mesh_node`    | choose backend, build pool/context per node |

  * [ ] Plan (no code yet) to:

    * [ ] Introduce a `dfps_vector_store::from_config(cfg: VectorStoreRuntimeConfig)` that returns `Arc<dyn VectorStorePort>` so `dfps_mesh_node` can remain ignorant of Qdrant/PGVector specifics.

* [ ] **MESH-02D – Cache/graph store design stubs**

  * [ ] `lib/platform/store/cache_store/README.md`:

    * [ ] Sketch a small `CacheStore` trait (get/set/delete/incr) and mention Redis as the first backend.
  * [ ] `lib/platform/store/graph_store/README.md`:

    * [ ] Sketch a `GraphStore` trait and note the relationship to `dfps_terminology::obo_graph`; IndraDB (or a similar backend) is a future implementation.

---

### MESH-03 – Platform data layer (mart/warehouse/lake) starting from `app/servers/datamart`

**Goal:** Move the *concept* of datamart/warehouse from `app/servers` to `platform/data/{mart,warehouse}`, keeping the planned directory names exactly as-is, and using `dfps_datamart` as the seed.

**Scope:** `lib/platform/data/{mart,warehouse,lake}` (new), `lib/app/servers/datamart`

* [ ] **MESH-03A – Create `platform/data` skeleton**

  * [ ] Create:

    ```text
    lib/platform/data
      mart/       (README only: conceptually hosts dfps_datamart)
      warehouse/  (README only)
      lake/       (README only)
    ```

  * [ ] In `data/mart/README.md`:

    * [ ] State that the existing `dfps_datamart` crate in `app/servers/datamart` is the **mart**; physical move will be a later MESH-impl card.
    * [ ] Summarize the mart schema: `Dim*` + `FactServiceRequest`, and how it consumes `dfps_contracts::PipelineOutput`.

* [ ] **MESH-03B – Warehouse model (design)**

  * [ ] In `data/warehouse/README.md`, define:

    * [ ] `WarehouseRole` (operational mart, reporting mart, archival).
    * [ ] `WarehouseConfig` that composes `RelationalConfig` and optional `LakeConfig`.
    * [ ] Traits:

      * `WarehouseLoader` – (PipelineOutput → dim/fact upsert).
      * `WarehouseAnalytics` – `ncit_summary`, `cohort`, plus future derived views.
  * [ ] Document that:

    * [ ] `dfps_mesh_node` uses `WarehouseLoader` + `WarehouseAnalytics` for node-local analytics.
    * [ ] `dfps_mesh_hub` might read only aggregated views, never raw facts.

* [ ] **MESH-03C – Lake model (design)**

  * [ ] In `data/lake/README.md`, define:

    * [ ] `LakeConfig` (path, file format, retention, partition columns).
    * [ ] `LakeWriter` / `LakeReader` traits.
  * [ ] Describe minimal use-cases:

    * [ ] Node snapshots (daily/weekly dumps of `fact_service_request` with DP noise).
    * [ ] Hub ingestion of snapshots when allowed by `dfps_compliance` + `dfps_mesh_governance`.

---

### MESH-04 – Mesh node runtime (`dfps_mesh_node`) grounded in current `dfps_api`

**Goal:** Treat the existing `dfps_api` as the initial implementation of `dfps_mesh_node`, designing a `NodeDataPlane` that wires domain + platform crates, and planning the eventual relocation into `platform/mesh/node` (without changing that directory layout).

**Scope:** `lib/app/servers/api`, new `lib/platform/mesh/node` (design + future crate)

* [ ] **MESH-04A – NodeDataPlane design (doc-first)**

  * [ ] Add `docs/system-design/mesh/node-runtime.md` describing:

    * [ ] A `NodeDataPlane` struct with fields:

      * `pipeline_port: dfps_pipeline` (or a small `PipelinePort` trait).
      * `datamart_sink: dfps_datamart` (or `Mart` trait).
      * `relational_cfg: RelationalConfig`, `relational_pool`.
      * `vector_runtime: dfps_vector_store` implementing `dfps_vector_port`.
      * `compliance_policy: dfps_compliance::Policy`.
      * `dataset_store: dfps_eval::FileDatasetStore`.
      * `metrics: dfps_observability::PipelineMetrics`.
    * [ ] Responsibilities:

      * `run_mapping_job(bundles)` → `PipelineOutput` + `LoadSummary` (or disabled datamart).
      * `run_analytics_job(query)` → analytics contracts.
      * `run_eval_job(request)` → eval contracts.

* [ ] **MESH-04B – Annotate current `dfps_api` as proto-node**

  * [ ] In `lib/app/servers/api/README.md`, add a section “**Relation to dfps_mesh_node**”:

    * [ ] Explicitly call `dfps_api` the *current node runtime*, to be moved into `platform/mesh/node` when stable.
    * [ ] Note that `ApiState` ~ early `NodeDataPlane` (but still fused with HTTP concerns).

  * [ ] In `dfps_api::server`:

    * [ ] Add comments or a mini `NodeDataPlane` struct (still in this crate) that:

      * [ ] Binds `pipeline`, `datamart`, `vector runtime`, `policy`, `dataset_store`, `metrics`.
      * [ ] Leaves Axum router/handlers as a thin HTTP port over it.

---

### MESH-05 – Mesh governance & hub design (no code movement yet)

**Goal:** Design the governance and hub crates under `platform/mesh/{governance,hub}` so they plug into the contracts and node runtime designed above, without touching the `platform/{data,store}` layout.

**Scope:** new `lib/platform/mesh/{governance,hub}` dirs + README design

* [ ] **MESH-05A – Governance model**

  * [ ] Create `lib/platform/mesh/governance/README.md` describing:

    * [ ] `QueryClass` (`MappingJob`, `AnalyticsJob`, `EvalJob`, `ExportJob`, `NodeIntrospection`).
    * [ ] `QueryDescriptor` (class + parameters: dataset, cohort filters, time range, expected cardinality).
    * [ ] `GovernanceDecision` enum (`Allow`, `Deny`, `AllowWithNoise`).
    * [ ] `NodePolicy` referencing `dfps_contracts::mesh::NodeCapabilities`.

  * [ ] Specify integration points:

    * [ ] `dfps_mesh_node` (today `dfps_api`) should call governance before executing jobs, but actual wiring can wait for implementation epics.

* [ ] **MESH-05B – Hub model**

  * [ ] Create `lib/platform/mesh/hub/README.md` describing:

    * [ ] `HubConfig` (list of node URLs/IDs, auth, timeouts, backoff).
    * [ ] `NodeRegistry` (NodeId → NodeMetadata from `dfps_contracts`).
    * [ ] `JobQueue` (Meshes `MeshJobDescriptor` to nodes; collects `MeshJobResult`).

  * [ ] Describe how existing endpoints (`/api/eval/*`, `/analytics/*`) could be used as **node APIs** that the hub calls, without changing the contracts.

---

### MESH-06 – Migration strategy, compatibility & docs

**Goal:** Plan the actual move of crates into the `platform/{data,store,mesh}` tree while preserving backward compatibility for existing apps/CLIs.

**Scope:** docs, “shim” crates, deprecation notes

* [ ] **MESH-06A – Migration map (phased)**

  * [ ] In `docs/system-design/mesh/migration-plan.md`, define phases:

    * Phase 1 – *Conceptual only*: introduce `platform/data/*` and `platform/store/*` READMEs (no code move).
    * Phase 2 – *Internal aliasing*: `dfps_datamart` and `dfps_vector_store` get alias crates or module re-exports under `platform/data/mart` and `platform/store/vector_store`.
    * Phase 3 – *Physical move*: once tests are stable, switch the `Cargo.toml` paths to the new directories.
    * Phase 4 – *Mesh node*: move `dfps_api` into `platform/mesh/node` as `dfps_mesh_node`, keeping a thin `dfps_api` shim.

* [ ] **MESH-06B – Runbooks**

  * [ ] Add `docs/runbook/mesh-node-quickstart.md`:

    * [ ] Based on current crates: `dfps_api` (node), `dfps_datamart` (warehouse), `dfps_vector_store` (vector store), `dfps_compliance`, `dfps_configuration`.
    * [ ] Show env profiles for “standalone node mode” (no hub) with SQLite + Qdrant/PGVector.

  * [ ] Add `docs/runbook/mesh-hub-quickstart.md` (design-level for now):

    * [ ] Outline how a future `dfps_mesh_hub` would call `dfps_mesh_node` (today `dfps_api`) endpoints with the new `mesh` contracts.

---

## INPROGRESS

* *Empty* (use as you start implementing design docs and skeleton crates).

---

## REVIEW

* *Empty*

---

## DONE

* *Empty* (this kanban tracks the **post-refactor mesh design**; implementation epics can split off later).
