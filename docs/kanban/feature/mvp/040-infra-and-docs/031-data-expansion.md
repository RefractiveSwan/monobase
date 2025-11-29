# 031 — Data-plane expansion: warehouse, vector, lake, cache

**Status:** TODO  
**Related crates / paths:**

- `lib/platform/data/data-plane/mart` (`refractive_swan_datamart`)
- `lib/platform/data/data-plane/warehouse` (`refractive_swan_datawarehouse` – design)
- `lib/platform/data/data-plane/lake` (`refractive_swan_datalake` – design)
- `lib/platform/data/data-stores/vector_store` (`refractive_swan_vector_store`)
- `lib/platform/data/data-stores/cache_store` (`refractive_swan_cache_store` – design)
- `lib/platform/mesh/node` (`refractive_swan_mesh_node`)
- `lib/platform/mesh/hub` (`refractive_swan_mesh_hub`)

---

## 1. Problem & goal

We need a cohesive data-plane story across node and hub:

- Nodes: `SqliteDatamart` as the operational sink, vector search wired via `VectorPipelineContext`, optional lake/cache hooks for export/DP governance.
- Hub: reporting warehouse/lake ingestion path for federated analytics/eval, consistent env/policy loading, and typed contracts for metrics.
- Shared: documented roles (`Operational`, future `Reporting`/`Archival`), CI/testing knobs, and schema generation for data-plane contracts.

**Success =** a single playbook for wiring warehouse/vector/lake/cache across node/hub with env seams, contracts, and tests covering the integration.

---

## 2. Tasks

### P1 — Node data-plane completeness

- [ ] Confirm `SqliteDatamart::from_optional_config(NodePlaneConfig::datamart_config())` is the node’s primary `DatamartSink` (tests for enabled/disabled labels).
- [ ] Vector context: construct `VectorPipelineContext` from `config_from_env()` + concrete store (`QdrantVectorStore::from_config` / `PgVectorStore::from_config`), honoring ranker `top_k` defaults; add health/label tests.
- [ ] Lake/cache hooks: keep `LakeWriter` / `LakeReader` optional in `NodeDataPlane`; sketch cache hooks for rate limits/DP budgets (stub traits acceptable).
- [ ] Roles: document node role as `WarehouseRole::Operational`; note future `Reporting`/`Archival` paths for lake/warehouse exports.

### P2 — Hub reporting warehouse/lake

- [ ] Decide reporting warehouse backend (`Sqlite` precursor or Postgres/DuckDB) and add a minimal `WarehouseAnalytics` trait.
- [ ] Implement lake ingestion stubs in `refractive_swan_mesh_hub` using lake reader + governance checks (DP required).
- [ ] Add contracts for `WarehouseSnapshot`/`IngestSummary` (align with lake design) and export JSON schemas.

### P3 — Env/config/policy consistency

- [ ] Standardize hub env namespace (`platform.mesh.hub`) and ensure no crate loads policy from env directly; inject pre-built `Policy`/`NodePolicy` into hub dispatch.
- [ ] Document env seams for data-plane: `app.web.api`, `app.mesh.node.<suffix>`, `platform.vector_store`, `platform.compliance`, lake/warehouse/cache namespaces.

### P4 — Observability & CI

- [ ] Expose data-plane metrics (datamart health, vector usage, lake export attempts) via `metrics_snapshot_json` for `/mesh/health` and hub dashboards.
- [ ] Run heavy data-plane/CLI/lake tests under `--features heavy-tests` on protected branches; document contributor expectations.

### P5 — Docs & schemas

- [ ] Add system-design notes for data-plane roles and export flows (node-runtime, lake README).
- [ ] Generate/refresh schemas for new contracts (warehouse snapshot/ingest summary) via `contracts-schema`; link locations in runbooks.

---

## 3. Definition of done

- Node data-plane uses a single wiring surface with vector/datamart/lake/cache hooks and tests for enabled/disabled paths.
- Hub can ingest/export via reporting warehouse/lake stubs with governance-ready seams.
- Env/policy loading is centralized; no crate loads policy directly from env.
- Observability captures data-plane metrics; heavy tests are gated and scheduled on protected branches.
- Contracts/schemas/docs updated for data-plane roles and flows.
