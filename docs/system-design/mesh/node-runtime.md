# Node Runtime Design

**Path:** `code/docs/system-design/mesh/node-runtime.md`  
**Scope:** `refractive_swan_mesh_node` architecture and `NodeDataPlane` design  
**Tracking:** MESH-025 (feature/mesh-data-plane)

This document describes the design of the **mesh node runtime** (`refractive_swan_mesh_node`), the core component that wires together the domain, platform/data, and platform/store layers into a cohesive node deployment. The `NodeDataPlane` type now lives in `lib/platform/mesh/node/src/plane.rs` so HTTP adapters can depend on it directly.

---

## Overview

A **mesh node** is a sovereign deployment unit that:

1. **Runs the mapping pipeline** (`refractive_swan_pipeline`)
2. **Persists results** to a node-local warehouse (`refractive_swan_datamart`)
3. **Exposes HTTP APIs** for mapping jobs, analytics, and eval
4. **Enforces governance policies** (`refractive_swan_compliance`, `refractive_swan_mesh_governance`)
5. **Reports capabilities** to the mesh hub (if part of a federated deployment)

The `NodeDataPlane` struct is the central orchestration point that binds these components.

> Mesh-specific DTOs (node IDs, job descriptors/results) come from the
> `refractive_swan_mesh_dto` veneer so node/hub runtimes depend on a curated surface rather
> than the entire contracts crate.

---

## NodeDataPlane Design

### Struct Fields

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use refractive_swan_pipeline::PipelinePort;
use refractive_swan_datamart::DatamartSink;
use refractive_swan_relational_store::RelationalConfig;
use refractive_swan_vector_port::VectorStore;
use refractive_swan_compliance::Policy;
use refractive_swan_eval::DatasetStore;
use refractive_swan_observability::PipelineMetrics;
use refractive_swan_mesh_dto::MeshNodeId;

pub struct NodeDataPlane {
    /// Node identity
    pub node_id: MeshNodeId,
    
    /// Pipeline orchestration (mapping engine + stages)
    pub pipeline: Arc<dyn PipelinePort + Send + Sync>,
    
    /// Datamart sink (warehouse persistence)
    pub datamart: Arc<dyn DatamartSink + Send + Sync>,
    
    /// Relational store configuration
    pub relational_cfg: RelationalConfig,
    
    /// Vector runtime (implements refractive_swan_vector_port::VectorStore)
    pub vector_runtime: Arc<dyn VectorStore + Send + Sync>,
    
    /// Compliance policy (DP, export licensing, etc.)
    pub compliance_policy: Policy,
    
    /// Dataset store for eval harness
    pub dataset_store: Arc<dyn DatasetStore + Send + Sync>,
    
    /// Metrics collector
    pub metrics: Arc<Mutex<PipelineMetrics>>,
}
```

### Design Rationale

- **Trait-based dependencies**: All major components use traits (`PipelinePort`, `DatamartSink`, `VectorStore`, `DatasetStore`) for testability and backend flexibility.
- **Arc wrapping**: Components are `Arc`-wrapped for HTTP handler sharing (Axum/HTTP frameworks clone state).
- **Mutex for metrics**: Metrics are mutable state, requiring interior mutability.
- **Node ID**: Each node has a unique identifier for mesh coordination.

---

## Responsibilities

### 1. Run Mapping Jobs

```rust
impl NodeDataPlane {
    pub async fn run_mapping_job(
        &self,
        bundles: &[Bundle],
    ) -> Result<PipelineOutput, NodeError> {
        // Run pipeline
        let output = self.pipeline.run(bundles).await?;
        
        // Persist to warehouse (if enabled)
        if let Ok(summary) = self.datamart.persist(&output).await {
            tracing::info!("Persisted {} rows to warehouse", summary.rows_inserted);
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.record_pipeline_run(&output);
        }
        
        Ok(output)
    }
}
```

**Inputs**: FHIR ServiceRequest bundles  
**Outputs**: `PipelineOutput` (mapped codes, confidence scores, etc.)  
**Side effects**: Warehouse persistence, metrics updates

### 2. Run Analytics Jobs

```rust
impl NodeDataPlane {
    pub async fn run_analytics_job(
        &self,
        query: AnalyticsQuery,
    ) -> Result<serde_json::Value, NodeError> {
        match query {
            AnalyticsQuery::NcitSummary(filters) => {
                let response = self.datamart.ncit_summary(&filters).await?;
                Ok(serde_json::to_value(response)?)
            }
            AnalyticsQuery::Cohort(query) => {
                let response = self.datamart.cohort(&query).await?;
                Ok(serde_json::to_value(response)?)
            }
        }
    }
}
```

**Inputs**: Analytics query descriptor  
**Outputs**: JSON response (from `refractive_swan_contracts::analytics`)

### 3. Run Eval Jobs

```rust
impl NodeDataPlane {
    pub async fn run_eval_job(
        &self,
        request: EvalRunRequest,
    ) -> Result<EvalRunResponse, NodeError> {
        use refractive_swan_eval::run_eval_with_mapper;
        
        // Load dataset
        let dataset = self.dataset_store.load(&request.dataset_name).await?;
        
        // Run eval
        let report = run_eval_with_mapper(
            &dataset,
            &*self.pipeline,
            request.top_k,
        ).await?;
        
        Ok(EvalRunResponse {
            dataset_name: request.dataset_name,
            metrics: report.metrics,
            calibration: report.calibration,
        })
    }
}
```

**Inputs**: Eval dataset name, top-k parameter  
**Outputs**: `EvalRunResponse` (metrics, calibration buckets)

### 4. Enforce Governance

```rust
impl NodeDataPlane {
    pub async fn run_job_with_governance(
        &self,
        job: &MeshJobDescriptor,
    ) -> Result<MeshJobResult, NodeError> {
        use refractive_swan_mesh_governance::GovernanceEngine;
        
        // Check if job is allowed
        let decision = self.governance.evaluate(job, &self.compliance_policy)?;
        
        match decision {
            GovernanceDecision::Allow => {
                // Execute job
                let output = match job.job_type {
                    MeshJobType::EvalDataset => self.run_eval_job(&job.parameters).await?,
                    MeshJobType::AnalyticsQuery => self.run_analytics_job(&job.parameters).await?,
                    // ...
                };
                
                Ok(MeshJobResult {
                    job_id: job.job_id.clone(),
                    status: MeshJobStatus::Success,
                    output: Some(output),
                    error: None,
                })
            }
            GovernanceDecision::Deny(reason) => {
                Ok(MeshJobResult {
                    job_id: job.job_id.clone(),
                    status: MeshJobStatus::Denied,
                    output: None,
                    error: Some(MeshError::new(
                        MeshErrorKind::PolicyDenied,
                        MeshErrorCode::new("governance_denied"),
                        reason,
                    )),
                })
            }
        }
    }
}
```

**Inputs**: `MeshJobDescriptor` (from hub or direct API call)  
**Outputs**: `MeshJobResult` (success/denied/failed)

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
- **refractive_swan_cache_store** (future): Analytics query caching

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

- `GET /api/eval/datasets` → list available datasets
- `POST /api/eval/run` → `run_eval_job`

### Mesh Endpoints (Future)

- `POST /mesh/job` → `run_job_with_governance`
- `GET /mesh/capabilities` → return `NodeCapabilities`
- `GET /mesh/health` → node health check

---

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
