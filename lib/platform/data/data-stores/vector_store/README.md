# dfps_vector_store (Platform Store)

**Conceptual location:** `lib/platform/store/vector_store`  
**Current physical location:** `lib/app/servers/vector_store`  
**Scope:** Concrete implementations of `dfps_vector_port::VectorStore` trait

This directory represents the **conceptual home** for the vector store platform component. The actual implementation currently lives at `lib/app/servers/vector_store` and will be moved here during Phase 3 of the MESH-025 migration.

---

## Domain / Store / Runtime Layer Mapping

The vector subsystem follows a strict three-layer architecture:

| Layer | Location | Crate | Responsibilities |
|-------|----------|-------|------------------|
| **Domain** | `lib/domain/vector_port` | `dfps_vector_port` | Abstract traits, errors, embedding metadata |
| **Store** | `lib/platform/store/vector_store` (target) | `dfps_vector_store` | Qdrant/PGVector/Milvus configs & drivers |
| **Runtime** | `lib/platform/mesh/node` (target) | `dfps_mesh_node` | Choose backend, build pool/context per node |

### Current Reality

- **Domain**: `dfps_vector_port` already exists at `lib/domain/vector_port` ✓
- **Store**: `dfps_vector_store` currently at `lib/app/servers/vector_store` (will move here)
- **Runtime**: `NodeDataPlane` currently in `dfps_api::server` (will become `dfps_mesh_node`)

### Contract

Domain crates (`dfps_mapping`, `dfps_pipeline`, `dfps_observability`) **must** depend only on `dfps_vector_port`, never `dfps_vector_store` directly. This ensures:

1. **Testability**: Domain logic can use mock implementations
2. **Backend flexibility**: Nodes can choose Qdrant, PGVector, Milvus, or custom backends
3. **Migration safety**: Moving `dfps_vector_store` doesn't break domain code

---

## Current Implementation (`lib/app/servers/vector_store`)

The `dfps_vector_store` crate currently provides:

### Backends

- **QdrantBackend**: HTTP/gRPC client for Qdrant vector database
- **PGVectorBackend**: Postgres with pgvector extension
- **MilvusBackend**: (planned) Milvus vector database
- **MockBackend**: In-memory fake for testing

### Main Components

- `backends/qdrant.rs`: Qdrant client, query translation, error mapping
- `backends/pgvector.rs`: Postgres + pgvector client
- `config.rs`: `VectorStoreRuntimeConfig` (backend enum + URL/pool settings)
- `lib.rs`: Re-exports and factory pattern

---

## Planned Factory Pattern (`from_config`)

To keep `dfps_mesh_node` (runtime) ignorant of backend specifics, `dfps_vector_store` will provide a factory:

```rust
// In dfps_vector_store (future)
use dfps_vector_port::{VectorStore, VectorBackend};
use std::sync::Arc;

/// Runtime configuration for vector store backends.
pub struct VectorStoreRuntimeConfig {
    pub backend: VectorBackend, // from dfps_vector_port
    pub url: String,
    pub namespace: String,
    pub pool_size: u32,
    pub timeout_secs: u64,
}

/// Factory that returns a trait object, hiding backend details.
pub fn from_config(cfg: VectorStoreRuntimeConfig) -> Arc<dyn VectorStore> {
    match cfg.backend {
        VectorBackend::Qdrant => Arc::new(QdrantBackend::connect(cfg)),
        VectorBackend::PgVector => Arc::new(PGVectorBackend::connect(cfg)),
        VectorBackend::Milvus => Arc::new(MilvusBackend::connect(cfg)),
        VectorBackend::Mock => Arc::new(MockBackend::default()),
    }
}
```

### Usage in `NodeDataPlane`

```rust
// In dfps_mesh_node (future) or dfps_api (current)
use dfps_vector_store::{from_config, VectorStoreRuntimeConfig};
use dfps_vector_port::VectorBackend;

pub struct NodeDataPlane {
    vector_runtime: Arc<dyn VectorStore>, // ← trait object, backend-agnostic
    // ...
}

impl NodeDataPlane {
    pub fn from_config(cfg: &NodeConfig) -> Self {
        let vector_cfg = VectorStoreRuntimeConfig {
            backend: VectorBackend::Qdrant, // from env or node config
            url: cfg.vector_store_url.clone(),
            namespace: "default".to_string(),
            pool_size: 5,
            timeout_secs: 30,
        };

        Self {
            vector_runtime: dfps_vector_store::from_config(vector_cfg),
            // ...
        }
    }
}
```

---

## Per-Node Backend Selection

The mesh architecture allows **different nodes** to use **different backends**:

- **Node A** (research): Qdrant (high-performance, cloud-native)
- **Node B** (production): PGVector (existing Postgres infra)
- **Node C** (testing): MockBackend (fast, deterministic)

All nodes expose the same `VectorStore` trait surface, so:
- Domain code (`dfps_mapping`) is identical across nodes
- Hub orchestration (`dfps_mesh_hub`) doesn't care about backend
- Migration/upgrades happen per-node without service-wide disruption

---

## Migration Plan (MESH-025)

### Phase 1: Conceptual Only (current)

- This README documents the **target** location
- Actual code stays at `lib/app/servers/vector_store`
- No code changes

### Phase 2: Internal Aliasing

- Create `lib/platform/store/vector_store/Cargo.toml` that re-exports `dfps_vector_store`
- Allows code to reference either path during transition

### Phase 3: Physical Move

- Move `lib/app/servers/vector_store/*` → `lib/platform/store/vector_store/*`
- Update `Cargo.toml` workspace members
- Verify all tests pass

### Phase 4: Mesh Node

- Move `dfps_api::NodeDataPlane` → `dfps_mesh_node`
- `dfps_mesh_node` depends on `dfps_vector_store` via stable `platform/store/vector_store` path

---

## Testing

### Unit Tests (in `dfps_vector_store`)

- Backend-specific query translation
- Connection pool/client lifecycle
- Error mapping (Qdrant/PGVector errors → `VectorStoreError`)

### Integration Tests (in `dfps_vector_store`)

- Real Qdrant connectivity (requires Docker/env)
- Real PGVector connectivity (requires Postgres + pgvector extension)
- Health checks, capacity snapshots

### Domain Tests (in `dfps_mapping`, `dfps_pipeline`)

- Use `MockBackend` for fast, deterministic tests
- Never depend on `dfps_vector_store` directly

---

## References

- **Domain Port**: `lib/domain/vector_port/README.md`
- **Current Implementation**: `lib/app/servers/vector_store`
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
- **Node Runtime Design**: `docs/system-design/mesh/node-runtime.md` (to be created)
