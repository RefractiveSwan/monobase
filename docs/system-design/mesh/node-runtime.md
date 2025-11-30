# Node Runtime Design

**Path:** `code/docs/system-design/mesh/node-runtime.md`  
**Scope:** `refractive_swan_mesh_node` architecture and `NodeDataPlane` design  
**Tracking:** MESH-025 (feature/mesh-data-plane), MESH-030 (mesh node/hub propagation)

This document describes the design of the **mesh node runtime** (`refractive_swan_mesh_node`), the core component that wires together the domain, platform/data, and platform/store layers into a cohesive node deployment. The `NodeDataPlane` type now lives in `lib/platform/mesh/node/src/plane.rs` so HTTP adapters can depend on it directly.

---

## Overview

A **mesh node** is a sovereign deployment unit that:

1. **Runs the mapping pipeline** (`refractive_swan_pipeline`)
2. **Persists results** to a node-local warehouse (`refractive_swan_datamart`)
3. **Exposes HTTP APIs** for mapping jobs, analytics, and eval
4. **Enforces governance policies** (`refractive_swan_compliance`, `refractive_swan_mesh_governance`)
5. **Reports capabilities** to the mesh hub (if part of a federated deployment)

The `NodeDataPlane` struct is the central orchestration point that binds these components and now powers the mesh endpoints exposed by `refractive_swan_api` (`/mesh/health`, `/mesh/capabilities`, `/mesh/governance`).

> Mesh-specific DTOs (node IDs, job descriptors/results) come from the
> `refractive_swan_mesh_dto` veneer so node/hub runtimes depend on a curated surface rather
> than the entire contracts crate.

---

## NodeDataPlane Design

### Struct Fields

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use refractive_swan_pipeline::{PipelinePort, VectorPipelineContext};
use refractive_swan_datamart::{DatamartSink, WarehouseConfig};
use refractive_swan_vector_store::VectorStoreConfig;
use refractive_swan_compliance::Policy;
use refractive_swan_eval::DatasetStore;
use refractive_swan_observability::PipelineMetrics;
use refractive_swan_mesh_dto::{MeshNodeId, NodeCapabilities};

pub struct NodeDataPlane {
    pub node_id: MeshNodeId,
    pub policy: Policy,
    pub vector_context: Option<VectorPipelineContext>,
    pub vector_config: Option<VectorStoreConfig>,
    pub dataset_store: Arc<dyn DatasetStore + Send + Sync>,
    pub metrics: Arc<Mutex<PipelineMetrics>>,
    pub pipeline: Arc<dyn PipelinePort + Send + Sync>,
    pub datamart: Arc<dyn DatamartSink + Send + Sync>,
    pub datamart_config: Option<WarehouseConfig>,
    pub max_dataset_size: u64,
    pub tags: Vec<String>,
}
```

### Design Rationale

- **Trait-based dependencies**: Pipeline/datamart/vector/dataset handles are trait objects so HTTP adapters stay thin and mocks remain easy to inject in tests.
- **Arc wrapping**: Components are `Arc`-wrapped for HTTP handler sharing (Axum clones state per request).
- **Metrics with mesh context**: `PipelineMetrics` is pre-seeded with `mesh_node_id` + `compliance_mode` so `/metrics/summary` and `/mesh/health` can emit tagged snapshots.
- **Config surface preserved**: `vector_config`, `datamart_config`, `max_dataset_size`, and `tags` are stored alongside live handles to power `NodeCapabilities`.
- **Deterministic identity**: `node_id` can be set via env (`refractive_swan_MESH_NODE_ID`) or `NodePlaneConfig::from_env_with_node_id` for multi-node tests.

### NodePlaneConfig env seams

- Namespace: `app.web.api` (loaded via `refractive_swan_configuration::load_env`).
- Optional namespacing for multi-node setups: `app.mesh.node.<suffix>` (reserve for future per-node overrides).
- Mesh identity/capacity: `refractive_swan_MESH_NODE_ID`, `refractive_swan_MESH_NODE_TAGS` (CSV), `refractive_swan_MESH_MAX_DATASET_SIZE`.
- Compliance: loaded once via `refractive_swan_compliance::ComplianceConfig::from_env`.
- Vector: `refractive_swan_VECTOR_*` + `VectorStoreConfig` (optional/disabled when `refractive_swan_VECTOR_ENABLED` is false).
- Datamart: `refractive_swan_WAREHOUSE_URL` (optional; disabled when missing).
- Cache: `refractive_swan_CACHE_BACKEND` (`disabled` | `inmemory` | `redis`), `refractive_swan_CACHE_URL` (required for Redis), `refractive_swan_CACHE_DEFAULT_TTL_SECS`.
---

## Responsibilities

### 1. Run Mapping Jobs

## Responsibilities

- **Pipeline orchestration** – HTTP controllers reuse the shared `pipeline` + `datamart` handles from the plane (persisting via `SqliteDatamart::from_optional_config` when enabled) and update the shared `metrics` handle.
- **Mesh capabilities** – `NodeDataPlane::capabilities(&self, base_url)` returns `NodeCapabilities` (vector backend, warehouse backend, compliance mode, max dataset size, tags) for `/mesh/capabilities` and hub discovery.
- **Health + observability** – `/metrics/summary` and `/mesh/health` expose `MetricsSnapshot` (`metrics_snapshot_json`) tagged with `mesh_node_id` and `compliance_mode`.
- **Eval datasets** – `dataset_store` and `policy` are shared with eval handlers so dataset manifests respect the same compliance mode.
- **Mesh jobs** - `NodeDataPlane::run_mesh_job` handles eval datasets, analytics queries, mapping health, and node introspection for `/mesh/job` (governance/export hooks will compose here once `refractive_swan_mesh_governance` lands).
- **Governance + cache** - `policy` is surfaced via `/mesh/governance`; DP budgets now read/write through `refractive_swan_cache_store` (in-memory/Redis) with a 24h TTL per `dp_budget:<node>:<date>` key, are durably recorded to `mesh_dp_budget` in the warehouse (sqlite today; Postgres/DuckDB once `refractive_swan_relational_store` backends land), and `/mesh/health` exposes `cache_backend`/`cache_health` alongside vector/warehouse health.
- **Regression health** – `MappingHealthCheck` runs the regression bundle (`fhir_bundle_sr`) through the pipeline, records vector usage/latency, attempts datamart persistence, and returns state counts plus backend labels, vector health, and warehouse health.
- **Mesh job schemas** – JSON schemas for `MeshJobDescriptor`, `MeshJobResult`, and per-type outputs (`mapping_health_check_report`, `export_job_summary`, `node_introspection_view`) live under `ci/contracts` (generate via `cargo run -p refractive_swan_contracts --bin contracts-schema`).
- **Warehouse roles** – Node datamart runs in `Operational` role (sqlite mart via `SqliteDatamart::from_optional_config`). Future `Reporting` / `Archival` roles will attach to a hub reporting warehouse or lake exports; lake hooks (`LakeWriter`/`LakeReader`) are optional injection points.

---

## Integration with Platform Layers

### Domain Layer

- **refractive_swan_pipeline**: Orchestrates ingestion → mapping → eval
- **refractive_swan_vector_port**: Abstract vector search interface
- **refractive_swan_contracts**: Canonical DTOs (`PipelineOutput`, `MeshJobDescriptor`, etc.)

### Platform Data Layer

- **refractive_swan_datamart**: Implements `DatamartSink` for node-local warehouse
- **refractive_swan_datawarehouse** (future): Abstract warehouse traits
- **refractive_swan_datalake** (future): Export snapshots with DP noise

### Platform Store Layer

- **refractive_swan_relational_store**: Connection pooling for warehouse
- **refractive_swan_vector_store**: Concrete vector backend (Qdrant/PGVector)
- **refractive_swan_cache_store**: In-memory cache today for DP budgets/rate limits; Redis planned for shared counters and analytics caching

### Platform Mesh Layer

- **refractive_swan_mesh_governance**: Policy evaluation for job requests
- **refractive_swan_mesh_hub** (future): Orchestrates cross-node jobs

---

## Current vs Future

### Current Reality (`refractive_swan_api`)

Today, `lib/app/servers/api/src/server.rs` contains a proto-`NodeDataPlane`:

```rust
// In refractive_swan_api::server
pub struct NodeDataPlane {
    pipeline: Arc<dyn PipelinePort>,
    datamart: Arc<dyn DatamartSink>,
    vector_runtime: Arc<dyn VectorStore>,
    policy: Policy,
    dataset_store: Arc<dyn DatasetStore>,
    metrics: Arc<Mutex<PipelineMetrics>>,
}
```

**Limitations**:
- Fused with HTTP concerns (Axum router in same crate)
- No mesh coordination (no `MeshNodeId`, no governance integration)
- No hub communication

### Future (`refractive_swan_mesh_node`)

**Location**: `lib/platform/mesh/node`

**Improvements**:
- **Decoupled**: `NodeDataPlane` is pure business logic, HTTP handlers are thin wrappers
- **Mesh-aware**: Exposes `MeshNodeId`, implements `NodeCapabilities` reporting
- **Governance-integrated**: Evaluates policies before executing jobs
- **Hub-ready**: Can register with `refractive_swan_mesh_hub` for federated deployments

---

## Configuration

### NodeConfig

```rust
pub struct NodeConfig {
    pub node_id: Option<MeshNodeId>, // If None, generate random UUID
    pub pipeline_config: PipelineConfig,
    pub relational_config: RelationalConfig,
    pub vector_config: VectorStoreRuntimeConfig,
    pub compliance_config: ComplianceConfig,
    pub dataset_root: PathBuf,
    pub hub_url: Option<String>, // If Some, register with hub
}
```

### From Environment

```rust
impl NodeConfig {
    pub fn from_env(profile: &str) -> Result<Self, ConfigError> {
        refractive_swan_configuration::load_env(&format!("mesh.node.{}", profile))?;
        
        Ok(Self {
            node_id: std::env::var("refractive_swan_NODE_ID").ok().map(MeshNodeId::from_string),
            pipeline_config: PipelineConfig::from_env()?,
            relational_config: RelationalConfig::from_env()?,
            vector_config: VectorStoreRuntimeConfig::from_env()?,
            compliance_config: ComplianceConfig::from_env()?,
            dataset_root: std::env::var("refractive_swan_DATASET_ROOT")?.into(),
            hub_url: std::env::var("refractive_swan_HUB_URL").ok(),
        })
    }
}
```

---

## HTTP API Surface

The node exposes HTTP endpoints (currently in `refractive_swan_api`, future in thin HTTP adapter):

### Mapping Endpoints

- `POST /api/map-bundles` → `run_mapping_job`
- `POST /api/map-single` → `run_mapping_job` (single bundle)

### Analytics Endpoints

- `GET /analytics/ncit-summary` → `run_analytics_job(NcitSummary)`
- `GET /analytics/cohort` → `run_analytics_job(Cohort)`

### Eval Endpoints

- `GET /api/eval/datasets` - list available datasets
- `POST /api/eval/run` - `run_eval_job`

### Mesh Endpoints (current surface)

- `POST /mesh/job` - `run_job_with_governance`
- `GET /mesh/capabilities` - return `NodeCapabilities`
- `GET /mesh/health` - node health check with cache/vector/warehouse labels

### Hub Surface (stubbed in `refractive_swan_api`)

- `GET /hub/nodes` - returns `NodeView` entries (node metadata + last `MetricsSnapshot`, cache/vector/warehouse labels).
- `POST /hub/jobs/analytics/ncit-summary` - dispatches `MeshJobType::AnalyticsQuery` across the registry and aggregates into `AnalyticsSummaryResponse`.
- `POST /hub/jobs/eval?dataset=<name>` - dispatches `MeshJobType::EvalDataset` across the registry and returns `FederatedEvalView { dataset, aggregated, per_node: [{ node_id, summary }] }` keyed by mesh node ID.

## Testing

### Unit Tests

- `NodeDataPlane::from_config` with various backends
- Job execution with mock dependencies

### Integration Tests

- Full pipeline → warehouse flow
- Analytics queries on real data
- Eval harness with test datasets

### End-to-End Tests

- HTTP API calls → `NodeDataPlane` → warehouse → analytics response
- Governance policy enforcement (deny export jobs)

---

## Migration Path

### Phase 1: Conceptual Design (current)

- This document describes the target architecture
- `refractive_swan_api` remains current proto-node

### Phase 2: Extract NodeDataPlane

- Create `lib/platform/mesh/node/src/data_plane.rs`
- Move business logic from `refractive_swan_api::server` to `NodeDataPlane`
- `refractive_swan_api` becomes a thin HTTP adapter

### Phase 3: Add Mesh Coordination

- Implement `MeshNodeId`, `NodeCapabilities`
- Add governance integration (`refractive_swan_mesh_governance`)
- Support hub registration (optional)

### Phase 4: Deprecate refractive_swan_api

- Create `refractive_swan_api` shim that wraps `refractive_swan_mesh_node`
- Document migration path for existing deployments

---

## References

- **Protocol Contracts**: `lib/domain/contracts/src/mesh.rs`
- **Current Implementation**: `lib/app/servers/api/src/server.rs`
- **Governance**: `lib/platform/mesh/governance/README.md` (to be created)
- **Hub**: `lib/platform/mesh/hub/README.md` (to be created)
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`



