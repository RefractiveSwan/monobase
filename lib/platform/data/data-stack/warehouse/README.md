# dfps_datawarehouse

**Conceptual location:** `lib/platform/data/warehouse`  
**Current physical location:** Not yet implemented (traits will be extracted from `dfps_datamart`)  
**Scope:** Backend-agnostic warehouse abstraction with role-based configurations

This directory will host the **warehouse trait layer** that abstracts over different warehouse roles (operational mart, reporting mart, archival) and backends.

---

## Purpose

`dfps_datawarehouse` provides:

1. **WarehouseRole**: Operational, reporting, archival
2. **WarehouseConfig**: Composes `RelationalConfig` + optional `LakeConfig`
3. **WarehouseLoader**: `PipelineOutput` → dim/fact upsert
4. **WarehouseAnalytics**: NCIt summary, cohort, derived views

**Goal**: Separate warehouse interface from implementation, enabling per-node warehouse configurations.

---

## Trait Design

### WarehouseRole

```rust
/// Warehouse deployment role.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarehouseRole {
    /// Operational mart: real-time pipeline persistence
    Operational,
    
    /// Reporting mart: optimized for analytics queries
    Reporting,
    
    /// Archival: long-term retention with compression
    Archival,
}
```

**Use cases**:
- **Operational**: Node-local datamart (current `dfps_datamart`)
- **Reporting**: Hub-aggregated read replicas (future)
- **Archival**: Compressed Parquet/Delta exports (future)

### WarehouseConfig

```rust
use dfps_relational_store::RelationalConfig;
use dfps_datalake::LakeConfig;

/// Configuration for warehouse instance.
#[derive(Clone, Debug)]
pub struct WarehouseConfig {
    /// Warehouse role
    pub role: WarehouseRole,
    
    /// Relational backend config
    pub relational: RelationalConfig,
    
    /// Optional lake config (for archival/export)
    pub lake: Option<LakeConfig>,
    
    /// Materialized view refresh interval (seconds)
    pub refresh_interval_secs: Option<u64>,
}
```

### WarehouseLoader Trait

```rust
use dfps_contracts::PipelineOutput;

/// Trait for loading pipeline outputs into warehouse.
#[async_trait]
pub trait WarehouseLoader: Send + Sync {
    /// Load a single pipeline output (upsert dims + insert fact).
    async fn load(&self, output: &PipelineOutput) -> Result<LoadSummary, WarehouseError>;
    
    /// Load a batch of outputs (optimized for bulk loads).
    async fn load_batch(&self, outputs: &[PipelineOutput]) -> Result<LoadSummary, WarehouseError>;
}

#[derive(Clone, Debug)]
pub struct LoadSummary {
    pub rows_inserted: u64,
    pub rows_updated: u64,
    pub duration_ms: u64,
}
```

### WarehouseAnalytics Trait

```rust
use dfps_contracts::{AnalyticsSummaryResponse, CohortResponse};

/// Trait for warehouse analytics queries.
#[async_trait]
pub trait WarehouseAnalytics: Send + Sync {
    /// NCIt code summary (top concepts).
    async fn ncit_summary(&self, filters: &Filters) -> Result<AnalyticsSummaryResponse, WarehouseError>;
    
    /// Cohort explorer (filtered service requests).
    async fn cohort(&self, query: &CohortQuery) -> Result<CohortResponse, WarehouseError>;
    
    /// Custom derived view query (future extensibility).
    async fn query_view(&self, view_name: &str, params: &serde_json::Value) -> Result<serde_json::Value, WarehouseError>;
}
```

### WarehouseError

```rust
#[derive(Debug, Error)]
pub enum WarehouseError {
    #[error("warehouse disabled")]
    Disabled,
    
    #[error("load failed: {source}")]
    LoadFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("query failed: {0}")]
    QueryFailed(String),
    
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}
```

---

## Relationship to Other Layers

| Layer | Crate | Responsibilities |
|-------|-------|------------------|
| **Domain** | `dfps_contracts` | `PipelineOutput`, analytics DTOs |
| **Warehouse (abstract)** | `dfps_datawarehouse` | `WarehouseLoader`, `WarehouseAnalytics` traits |
| **Warehouse (concrete)** | `dfps_datamart` | Star schema, SQL queries, implements traits |
| **Store** | `dfps_relational_store` | Connection pooling, migrations |

---

## Node vs Hub Usage

### Node-Local Warehouse (`dfps_mesh_node`)

```rust
use dfps_datawarehouse::{WarehouseConfig, WarehouseRole};
use dfps_datamart::SqlDatamart;
use dfps_relational_store::RelationalConfig;

pub struct NodeDataPlane {
    warehouse: Arc<dyn WarehouseLoader + WarehouseAnalytics>,
    // ...
}

impl NodeDataPlane {
    pub fn from_config(cfg: &NodeConfig) -> Self {
        let warehouse_cfg = WarehouseConfig {
            role: WarehouseRole::Operational,
            relational: RelationalConfig {
                backend: RelationalBackend::Sqlite,
                url: cfg.warehouse_url.clone(),
                pool_max: 10,
                connect_timeout_secs: 30,
                schema: None,
            },
            lake: None, // No lake for operational nodes
            refresh_interval_secs: None,
        };

        let warehouse = SqlDatamart::new(warehouse_cfg);

        Self {
            warehouse: Arc::new(warehouse),
            // ...
        }
    }

    pub async fn run_mapping_job(&self, bundles: &[Bundle]) -> Result<PipelineOutput, NodeError> {
        let output = self.pipeline.run(bundles).await?;
        
        // Persist to node-local warehouse
        self.warehouse.load(&output).await?;
        
        Ok(output)
    }
}
```

### Hub Reporting Warehouse (`dfps_mesh_hub`)

```rust
// Future: hub reads aggregated views, never raw facts
pub struct HubDataPlane {
    reporting_warehouse: Arc<dyn WarehouseAnalytics>, // read-only
    // ...
}

impl HubDataPlane {
    pub async fn global_ncit_summary(&self) -> Result<AnalyticsSummaryResponse, HubError> {
        // Query reporting mart (aggregated from nodes)
        self.reporting_warehouse.ncit_summary(&Filters::default()).await
    }
}
```

---

## Warehouse Roles in Detail

### Operational Mart

- **Purpose**: Real-time pipeline persistence (node-local)
- **Backend**: SQLite or Postgres
- **Schema**: Full star schema (dims + facts)
- **Refresh**: N/A (always current)
- **Queries**: NCIt summary, cohort explorer

### Reporting Mart

- **Purpose**: Aggregated analytics (hub-level or node read replicas)
- **Backend**: Postgres or DuckDB
- **Schema**: Materialized views, pre-aggregated metrics
- **Refresh**: Periodic (hourly/daily)
- **Queries**: Global NCIt summary, cross-node cohorts

### Archival

- **Purpose**: Long-term retention with compression
- **Backend**: Parquet/Delta lake (via `dfps_datalake`)
- **Schema**: Partitioned by date, minimal indexing
- **Refresh**: Nightly snapshots
- **Queries**: Historical trend analysis, compliance audits

---

## Migration Plan (MESH-025)

### Phase 1: Design Only (current)

- This README documents the planned abstraction
- No implementation yet

### Phase 2: Extract Traits

- Create `lib/platform/data/warehouse/src/traits.rs`
- Define `WarehouseRole`, `WarehouseConfig`, `WarehouseLoader`, `WarehouseAnalytics`, `WarehouseError`
- `dfps_datamart` implements these traits

### Phase 3: Wire to Mesh Node

- `dfps_mesh_node::NodeDataPlane` accepts `Arc<dyn WarehouseLoader + WarehouseAnalytics>`
- Node config selects role and relational backend
- Tests verify operational mart behavior

### Phase 4: Hub Reporting (Future)

- Implement reporting mart with materialized views
- Hub queries aggregated metrics without accessing node fact tables

---

## Testing

### Unit Tests (in `dfps_datawarehouse`)

- `WarehouseConfig` validation
- Trait definitions compile

### Integration Tests (in `dfps_datamart`)

- Implement `WarehouseLoader` and verify `load_batch` correctness
- Implement `WarehouseAnalytics` and verify queries

### End-to-End Tests (in `dfps_mesh_node`)

- Pipeline → warehouse → analytics flow
- Operational mart persistence latency

---

## References

- **Datamart Implementation**: `lib/platform/data/mart/README.md`
- **Relational Store**: `lib/platform/store/relational_store/README.md`
- **Lake**: `lib/platform/data/lake/README.md` (to be created)
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
