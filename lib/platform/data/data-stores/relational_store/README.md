# refractive_swan_relational_store (Platform Store)

**Conceptual location** `lib/platform/store/relational_store`  
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

## Migration note (P3.3)

- `WarehouseConfig` (used by `SqliteDatamart`) maps 1:1 onto a future `RelationalConfig` (`backend`, `url`, `pool_max`, `schema`). The mesh node and hub can switch from SQLite to Postgres/DuckDB by swapping the backend without changing domain DTOs.
- `CacheStore` + DP budget hot-cache now persist budgets durably via this crate (sqlite today) into `mesh_dp_budget`; Postgres/DuckDB backends will take over once wired.

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

## Current Implementation ​(in `refractive_swan_datamart`)

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
- **Node B** (production): Postgres (robust, concurrent)
- **Node C** (analytics research): DuckDB (OLAP-optimized, columnar)

The `WarehouseLoader` trait (in `refractive_swan_datawarehouse`) remains identical, so:
- Domain logic (`refractive_swan_pipeline` → warehouse persistence) doesn't change
- Hub orchestration doesn't care about backend
- Migration/upgrades are per-node

---

## References

- **Current Implementation**: `lib/platform/data/data-plane/mart/src/sql.rs`
- **Planned Warehouse Traits**: `lib/platform/data/warehouse/README.md` (to be created)
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
- **Node Runtime**: `docs/system-design/mesh/node-runtime.md` (to be created)
