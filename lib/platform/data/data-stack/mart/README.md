# dfps_datamart

**Conceptual location:** `lib/platform/data/mart`  
**Current physical location:** `lib/app/servers/datamart`  
**Scope:** Dimensional modeling and fact tables for node-local analytics

This directory represents the **conceptual home** for the datamart component. The actual implementation currently lives at `lib/app/servers/datamart` and will be moved here during Phase 3 of the MESH-025 migration.

---

## Purpose

The datamart is the **operational warehouse** within a mesh node. It provides:

1. **Dimensional schema**: `Dim*` tables (code systems, organizations, licenses)
2. **Fact tables**: `FactServiceRequest` capturing pipeline outputs
3. **Analytics queries**: NCIt summary, cohort explorer
4. **Data loading**: Transform `PipelineOutput` → warehouse rows

---

## Current Implementation

**Location**: `lib/app/servers/datamart`

### Schema Structure

The mart follows a **star schema** pattern:

#### Dimension Tables

- `DimNcitCode`: NCIt concepts (code, display_name, category)
- `DimServiceCodeSystem`: Source code systems (CPT, HCPCS, SNOMED, etc.)
- `DimOrganization`: Healthcare organizations
- `DimLicenseTier`: License categories (controlled, export-restricted, etc.)

#### Fact Tables

- `FactServiceRequest`: Central fact table linking:
  - Service request ID
  - NCIt mapped code (FK to DimNcitCode)
  - Source code + system (FK to DimServiceCodeSystem)
  - Organization (FK to DimOrganization)
  - License tier (FK to DimLicenseTier)
  - Mapping metadata (confidence score, rank, match type)
  - Temporal dimensions (request timestamp)

### Key Components

- `port.rs`: `DatamartSink` trait, `SqliteDatamart` implementation
- `sql.rs`: Schema DDL, connection pooling, raw SQL queries
- `dim.rs`, `fact.rs`, `keys.rs`: Row mapping helpers (`PipelineOutput` → warehouse rows)

---

## DatamartSink Trait

```rust
#[async_trait]
pub trait DatamartSink: Send + Sync {
    /// Persist pipeline output to warehouse.
    async fn persist(&self, output: &PipelineOutput) -> Result<LoadSummary, DatamartError>;
    
    /// NCIt summary analytics query.
    async fn ncit_summary(&self, filters: &Filters) -> Result<Vec<SummaryRow>, DatamartError>;
    
    /// Cohort explorer query.
    async fn cohort(&self, query: &CohortQuery) -> Result<Vec<CohortRow>, DatamartError>;
}
```

**Used by**:
- `dfps_api::NodeDataPlane` – wires datamart for persistence
- `dfps_cli` – standalone analytics
- `dfps_web_frontend` – dashboard queries

---

## Separation from Warehouse Layer

| Concern | Crate | Location |
|---------|-------|----------|
| **Mart schema & queries** | `dfps_datamart` | `platform/data/mart` (conceptual) |
| **Warehouse traits** | `dfps_datawarehouse` | `platform/data/warehouse` (future) |
| **Relational driver** | `dfps_relational_store` | `platform/store/relational_store` |

**Analogy**:
- `dfps_datamart` = "star schema + business metrics"
- `dfps_datawarehouse` = "abstract warehouse interface (WarehouseLoader, WarehouseAnalytics)"
- `dfps_relational_store` = "SQLx connection pools"

---

## Relationship to PipelineOutput

The mart **consumes** domain contracts:

```rust
// From dfps_contracts::pipeline
pub struct PipelineOutput {
    pub request_id: String,
    pub source_code: String,
    pub source_system: String,
    pub ncit_code: Option<String>,
    pub ncit_display_name: Option<String>,
    pub confidence_score: f64,
    pub license_tier: Option<String>,
    // ...
}
```

The mart transforms this into warehouse rows:

```rust
// In dfps_datamart/src/fact.rs
impl From<&PipelineOutput> for FactServiceRequest {
    fn from(output: &PipelineOutput) -> Self {
        Self {
            request_id: output.request_id.clone(),
            ncit_code_id: hash_ncit_code(&output.ncit_code),
            source_code_id: hash_source_code(&output.source_code, &output.source_system),
            confidence_score: output.confidence_score,
            // ...
        }
    }
}
```

---

## Per-Node Warehouse Backend

The mart layer is **backend-agnostic**. Different nodes can use:

- **Node A** (dev): SQLite (fast, no setup)
- **Node B** (production): Postgres (concurrent, robust)
- **Node C** (analytics): DuckDB (OLAP-optimized)

The mart's SQL queries are **portable** (using ANSI SQL + minimal dialect-specific code).

---

## Analytics Queries

### NCIt Summary

Aggregates service requests by NCIt code:

```sql
SELECT 
    d.code,
    d.display_name,
    COUNT(f.request_id) as count
FROM fact_service_request f
JOIN dim_ncit_code d ON f.ncit_code_id = d.id
GROUP BY d.code, d.display_name
ORDER BY count DESC;
```

**Use case**: Top mapped NCIt concepts dashboard

### Cohort Explorer

Filters service requests by attributes:

```sql
SELECT 
    f.request_id,
    d.code,
    d.display_name,
    s.source_system,
    f.confidence_score
FROM fact_service_request f
JOIN dim_ncit_code d ON f.ncit_code_id = d.id
JOIN dim_service_code_system s ON f.source_code_id = s.id
WHERE d.category = ? AND f.confidence_score > ?;
```

**Use case**: Export cohorts for research/review

---

## Migration Plan (MESH-025)

### Phase 1: Conceptual Only (current)

- This README documents the **target** location
- Code stays at `lib/app/servers/datamart`
- No code changes

### Phase 2: Extract Warehouse Traits

- Create `dfps_datawarehouse` with `WarehouseLoader`, `WarehouseAnalytics` traits
- `dfps_datamart` implements these traits
- Existing code still works (minimal refactoring)

### Phase 3: Physical Move

- Move `lib/app/servers/datamart/*` → `lib/platform/data/mart/*`
- Update `Cargo.toml` workspace members
- Verify all tests pass

### Phase 4: Mesh Node Wiring

- `dfps_mesh_node::NodeDataPlane` wires:
  - `RelationalPool` from `dfps_relational_store`
  - `DatamartSink` from `dfps_datamart`
- Node config selects relational backend

---

## Testing

### Unit Tests (in `dfps_datamart`)

- Dim/fact row mapping from `PipelineOutput`
- SQL query construction
- Error handling (disabled datamart, missing fields)

### Integration Tests (in `dfps_datamart`)

- Real SQLite connectivity (in-memory)
- Schema migrations
- Analytics queries with realistic data

### End-to-End Tests (in `dfps_api` or future `dfps_mesh_node`)

- Pipeline → datamart → analytics flow
- Verify NCIt summary on real mapping outputs
- Cohort queries with various filters

---

## References

- **Current Implementation**: `lib/app/servers/datamart`
- **Warehouse Traits**: `lib/platform/data/warehouse/README.md` (to be created)
- **Relational Store**: `lib/platform/store/relational_store/README.md`
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
