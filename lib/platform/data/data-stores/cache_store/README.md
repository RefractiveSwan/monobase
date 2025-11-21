# refractive_swan_relational_store (Platform Store)

**Conceptual location:** `lib/platform/store/relational_store`  
**Current physical location:** Embedded in `lib/app/servers/datamart/src/sql.rs`  
**Scope:** Backend-agnostic relational store abstraction with SQLx/Postgres/DuckDB drivers

This directory represents the **planned home** for relational database connection pooling, migration, and query execution abstraction. The current implementation is embedded in `refractive_swan_datamart`'s SQL wiring and will be extracted during MESH-025.

---

## Purpose

`refractive_swan_relational_store` provides:

1. **Backend enum**: `RelationalBackend` (Sqlite, Postgres, Duckdb, External)
2. **Connection traits**: `RelationalPool`, `RelationalMigrator`
3. **Configuration**: `RelationalConfig` with URL, pool size, timeouts, schema
4. **Error mapping**: Unified `RelationalError` wrapping SQLx/backend errors

**Goal**: Keep SQLx concretions behind this crate so `refractive_swan_datamart` and `refractive_swan_datawarehouse` depend on traits, not SQLx directly.

---

## Trait Design

### RelationalBackend

```rust
/// Supported relational database backends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalBackend {
    /// SQLite (file-based or in-memory)
    Sqlite,
    
    /// PostgreSQL
    Postgres,
    
    /// DuckDB (OLAP-oriented, columnar)
    Duckdb,
    
    /// External (custom connection string, no migration support)
    External,
}
```

### RelationalConfig

```rust
/// Configuration for relational store connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalConfig {
    /// Backend type
    pub backend: RelationalBackend,
    
    /// Connection URL (e.g., `postgres://...`, `sqlite::memory:`)
    pub url: String,
    
    /// Maximum pool size
    pub pool_max: u32,
    
    /// Connection timeout (seconds)
    pub connect_timeout_secs: u64,
    
    /// Optional schema name (Postgres-specific)
    pub schema: Option<String>,
}
```

### RelationalPool

```rust
/// Trait for managing a connection pool.
pub trait RelationalPool: Clone + Send + Sync {
    type Pool;
    
    /// Connect to the database and create a pool.
    fn connect(cfg: &RelationalConfig) -> Result<Self::Pool, RelationalError>;
    
    /// Close the pool and release connections.
    fn close(&self) -> Result<(), RelationalError>;
}
```

### RelationalMigrator

```rust
/// Trait for running schema migrations.
pub trait RelationalMigrator {
    type Pool;
    
    /// Run migrations on the pool.
    fn migrate(pool: &Self::Pool) -> Result<(), RelationalError>;
}
```

### RelationalError

```rust
#[derive(Debug, Error)]
pub enum RelationalError {
    #[error("invalid backend: {0}")]
    InvalidBackend(String),
    
    #[error("connection failed: {source}")]
    ConnectionFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("migration failed: {source}")]
    MigrationFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("query execution failed: {0}")]
    QueryFailed(String),
    
    #[error("relational store disabled")]
    Disabled,
}
```

---

## Current Implementation (in `refractive_swan_datamart`)

Today, `refractive_swan_datamart` (`lib/platform/data/data-plane/mart`) hosts the SQL wiring:

- `src/sql.rs`: 
  - `WarehouseConfig::from_env()` reads `refractive_swan_WAREHOUSE_URL`, etc.
  - `SqliteDatamart::from_env()` creates SQLx pool
  - Schema DDL (CREATE TABLE statements)
  - Raw SQL queries for `ncit_summary`, `cohort`, `persist`

### What Will Move to `refractive_swan_relational_store`

- **Backend selection logic**: SQLite vs Postgres vs DuckDB
- **Connection pool creation**: `sqlx::Pool<Sqlite>`, `sqlx::Pool<Postgres>`
- **Migration runner**: `sqlx::migrate!()` wrapper
- **Unified error mapping**: SQLx errors → `RelationalError`

### What Stays in `refractive_swan_datamart`

- **Schema definition**: Dim tables, Fact tables (warehouse-specific)
- **Business queries**: `ncit_summary`, `cohort` (analytics-specific)
- **Data loading**: `PipelineOutput` → warehouse row logic (domain mapping)

---

## Separation from Warehouse Layer

| Concern | Crate | Location |
|---------|-------|----------|
| **Connection pooling & drivers** | `refractive_swan_relational_store` | `platform/store/relational_store` |
| **Warehouse schema & queries** | `refractive_swan_datamart` | `platform/data/mart` |
| **Warehouse traits (role, loader, analytics)** | `refractive_swan_datawarehouse` | `platform/data/warehouse` (future) |

**Analogy**: Think of `refractive_swan_relational_store` as the "Postgres client library" and `refractive_swan_datamart` as the "warehouse schema + business logic."

---

## Per-Node Backend Selection

Different mesh nodes can use different relational backends:

- **Node A** (dev/test): SQLite (fast, no infra)
-**Node B** (production): Postgres (robust, concurrent)
- **Node C** (analytics research): DuckDB (OLAP-optimized, columnar)

The `WarehouseLoader` trait (in `refractive_swan_datawarehouse`) remains identical, so:
- Domain logic (`refractive_swan_pipeline` → warehouse persistence) doesn't change
- Hub orchestration doesn't care about backend
- Migration/upgrades are per-node

---

## Migration Plan (MESH-025)

### Phase 1: Conceptual Only (current)

- This README documents the **target** abstraction
- Code stays in `refractive_swan_datamart/src/sql.rs`
- No code changes

### Phase 2: Extract Traits

- Create `lib/platform/store/relational_store/src/traits.rs`
- Define `RelationalPool`, `RelationalMigrator`, `RelationalConfig`, `RelationalError`
- `refractive_swan_datamart` imports traits from `refractive_swan_relational_store`

### Phase 3: Extract Backend Implementations

- Move SQLx-specific connection logic into `refractive_swan_relational_store/src/backends/`
- `refractive_swan_datamart` becomes a consumer of `RelationalPool` trait
- Tests verify warehouse queries work with all backends

### Phase 4: Mesh Node Wiring

- `refractive_swan_mesh_node::NodeDataPlane` wires `RelationalPool` and passes to `refractive_swan_datamart`
- Node config selects backend (from env or `NodeConfig`)

---

## Example Usage (Future)

### In `refractive_swan_mesh_node`

```rust
use refractive_swan_relational_store::{RelationalConfig, RelationalBackend, RelationalPool};
use refractive_swan_datamart::Datamart;

pub struct NodeDataPlane {
    relational_pool: Box<dyn RelationalPool>, // ← backend-agnostic
    datamart: Arc<dyn DatamartSink>,
    // ...
}

impl NodeDataPlane {
    pub fn from_config(cfg: &NodeConfig) -> Self {
        let relational_cfg = RelationalConfig {
            backend: RelationalBackend::Postgres, // from env/node config
            url: cfg.warehouse_url.clone(),
            pool_max: 10,
            connect_timeout_secs: 30,
            schema: Some("public".to_string()),
        };

        let pool = refractive_swan_relational_store::connect(relational_cfg).unwrap();
        refractive_swan_relational_store::migrate(&pool).unwrap();

        let datamart = refractive_swan_datamart::SqlDatamart::new(pool);

        Self {
            relational_pool: Box::new(pool),
            datamart: Arc::new(datamart),
            // ...
        }
    }
}
```

### In `refractive_swan_datamart`

```rust
use refractive_swan_relational_store::{RelationalPool, RelationalError};

pub struct SqlDatamart<P: RelationalPool> {
    pool: P,
}

impl<P: RelationalPool> SqlDatamart<P> {
    pub fn new(pool: P) -> Self {
        Self { pool }
    }

    pub async fn ncit_summary(&self, filters: &Filters) -> Result<Vec<SummaryRow>, DatamartError> {
        let rows = sqlx::query_as::<_, SummaryRow>(
            "SELECT code, display_name, count FROM dim_ncit_code WHERE ..."
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
```

---

## Testing

### Unit Tests (in `refractive_swan_relational_store`)

- `RelationalConfig` validation
- Backend selection logic
- Error mapping (SQLx errors → `RelationalError`)

### Integration Tests (in `refractive_swan_relational_store`)

- Real SQLite connectivity (in-memory)
- Real Postgres connectivity (requires Docker/env)
- Migration runner with test schemas

### Warehouse Tests (in `refractive_swan_datamart`)

- Use SQLite backend for fast tests
- Verify schema DDL with all backends
- Analytics queries validated across backends (Postgres for CI, SQLite for unit tests)

---

## References

- **Current Implementation**: `lib/app/servers/datamart/src/sql.rs`
- **Planned Warehouse Traits**: `lib/platform/data/warehouse/README.md` (to be created)
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
- **Node Runtime**: `docs/system-design/mesh/node-runtime.md` (to be created)
