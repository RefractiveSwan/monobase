# refractive_swan_vector_port

**Path:** `code/lib/domain/vector_port`  
**Scope:** Domain-level abstraction for vector similarity search

This crate defines the **domain port** (abstract interface) for vector similarity search operations required by mapping and pipeline logic. It contains only trait definitions, errors, and embedding metadata types—no concrete backend implementations.

---

## Architecture

### Layer Separation

The vector subsystem follows a strict ports-and-adapters pattern with three layers:

| Layer | Crate | Responsibilities |
|-------|-------|------------------|
| **Domain** | `refractive_swan_vector_port` | Traits, errors, embedding metadata |
| **Store** | `refractive_swan_vector_store` | Qdrant/PGVector configs &drug drivers (conceptual location: `platform/store/vector_store`) |
| **Runtime** | `refractive_swan_mesh_node` | Choose backend, build pool/context per node (current: `refractive_swan_api::NodeDataPlane`) |

**Critical invariant**: No domain crate (`refractive_swan_mapping`, `refractive_swan_pipeline`, `refractive_swan_observability`) may depend directly on `refractive_swan_vector_store`. All domain code uses `refractive_swan_vector_port` exclusively.

### Trait Surface

#### `VectorStorePort`

The primary trait for vector search operations:

```rust
pub trait VectorStorePort: Send + Sync {
    async fn search(
        &self,
        query: &EmbeddingQuery,
    ) -> Result<Vec<ScoredMatch>, VectorStoreError>;
    
    async fn index(
        &self,
        embedding: &Embedding,
    ) -> Result<(), VectorStoreError>;
    
    async fn usage_snapshot(&self) -> Result<VectorUsageSnapshot, VectorStoreError>;
}
```

**Expected by**:
- `refractive_swan_mapping::vector_ranker` – searches for candidate mappings
- `refractive_swan_pipeline::DefaultPipeline` – passes vector runtime to mapping engine
- `refractive_swan_observability::PipelineMetrics` – captures usage snapshots

#### Supporting Types

- `EmbeddingQuery`: Query vector + optional filters (top-k, threshold, namespace)
- `Embedding`: Vector + metadata (code, system, display name)
- `ScoredMatch`: Match result with score, code, metadata
- `VectorUsageSnapshot`: Capacity metrics for observability
- `VectorStoreError`: Domain-friendly error enum (ConfigError, SearchError, IndexError, Unavailable)

---

## Implementations

### refractive_swan_vector_store (Platform Store)

**Current location**: `lib/platform/data/data-stores/vector_store`  
**Conceptual location**: `lib/platform/store/vector_store`

This crate provides concrete implementations of `VectorStorePort`:

- `QdrantBackend`: Qdrant HTTP/gRPC client
- `PGVectorBackend`: Postgres with pgvector extension

The store crate contains:
- Backend-specific configurations (`QdrantConfig`, `PGVectorConfig`)
- Connection pool/client builders
- Query translation (domain `EmbeddingQuery` → backend API)
- Error mapping (backend errors → `VectorStoreError`)

**Factory pattern (planned)**:
```rust
// In refractive_swan_vector_store (future)
pub fn from_config(cfg: VectorStoreRuntimeConfig) -> Arc<dyn VectorStorePort> {
    match cfg.backend {
        VectorBackend::Qdrant => Arc::new(QdrantBackend::connect(cfg.qdrant_url)),
        VectorBackend::PGVector => Arc::new(PGVectorBackend::connect(cfg.pg_url)),
    }
}
```

### NodeDataPlane Wiring

**Current**: `refractive_swan_api::server::NodeDataPlane` wires the vector runtime:

```rust
pub struct NodeDataPlane {
    pipeline: Arc<dyn PipelinePort>,
    vector_runtime: Arc<dyn VectorStorePort>, // ← from refractive_swan_vector_store
    datamart: Arc<dyn DatamartSink>,
    // …
}
```

**Future**: `refractive_swan_mesh_node::NodeDataPlane` will wire the same way, allowing:
- Per-node backend selection (Qdrant for node A, PGVector for node B)
- Runtime configuration without changing domain code
- Easier testing (mock implementations of `VectorStorePort`)

---

## Usage Guidelines

### For Domain Developers

If you're writing domain logic that needs vector search:

1. **Depend on `refractive_swan_vector_port` only**: Add `refractive_swan_vector_port = { path = "../vector_port" }` to your domain crate's `Cargo.toml`.
2. **Accept the trait**: Take `Arc<dyn VectorStorePort>` as a dependency injection parameter.
3. **Use domain errors**: Handle `VectorStoreError` (not Qdrant/PGVector errors).
4. **Don't configure backends**: Configuration and backend selection lives in runtime/platform layers.

Example:

```rust
// In refractive_swan_mapping/src/vector_ranker.rs
use refractive_swan_vector_port::{VectorStorePort, EmbeddingQuery, VectorStoreError};

pub async fn rank_candidates(
    query: &str,
    vector_store: &dyn VectorStorePort,
) -> Result<Vec<ScoredMatch>, VectorStoreError> {
    let embedding_query = EmbeddingQuery::new(query, top_k: 10);
    vector_store.search(&embedding_query).await
}
```

### For Platform Developers

If you're implementing a new vector store backend:

1. **Implement `VectorStorePort`**: In `refractive_swan_vector_store`, create a new backend struct.
2. **Add configuration**: Define a backend-specific config struct.
3. **Register in factory**: Update `from_config` to handle the new backend.
4. **Test with domain code**: Verify mapping/pipeline tests pass with your backend.

---

## Testing

### Unit Tests

Domain crates test vector logic with **mock implementations**:

```rust
// In refractive_swan_test_suite or inline test
struct MockVectorStore { /* … */ }

impl VectorStorePort for MockVectorStore {
    async fn search(&self, _: &EmbeddingQuery) -> Result<Vec<ScoredMatch>, VectorStoreError> {
        Ok(vec![/* fake matches */])
    }
    // …
}
```

### Integration Tests

`refractive_swan_vector_store` integration tests verify:
- Real Qdrant/PGVector connectivity
- Query translation accuracy
- Error handling for unavailable backends

---

## Migration Notes (MESH-025)

As part of the mesh-first architecture refactoring:

- **`refractive_swan_vector_port`**: Stays in `lib/domain/vector_port` (no changes).
- **`refractive_swan_vector_store`**:  
  - **Current**: `lib/platform/data/data-stores/vector_store`  
  - **Target**: `lib/platform/store/vector_store`
  - Will be moved in Phase 3 of MESH-025 migration.

- **`refractive_swan_mesh_node`**:  
  - Current `NodeDataPlane` in `refractive_swan_api` will be extracted into `lib/platform/mesh/node`.  
  - Vector runtime wiring will remain identical.

### No Domain Changes Required

Because domain code depends only on `refractive_swan_vector_port`, the migration is **transparent** to:
- `refractive_swan_mapping`
- `refractive_swan_pipeline`
- `refractive_swan_observability`

Only runtime/platform code (currently `refractive_swan_api`) needs updates.

---

## References

- **Vector Store Implementation**: `lib/app/servers/vector_store` (conceptual: `platform/store/vector_store`)
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
- **Node Runtime**: `docs/system-design/mesh/node-runtime.md` (to be created)
- **Contracts**: `lib/domain/contracts` (co-located DTOs for HTTP/eval surfaces)
