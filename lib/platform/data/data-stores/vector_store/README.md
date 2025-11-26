# refractive_swan_vector_store (Platform Store)

**Conceptual location:** `lib/platform/store/vector_store`  
**Current physical location:** `lib/platform/data/data-stores/vector_store`  
**Scope:** Concrete implementations of `refractive_swan_vector_port::VectorStore` trait

This directory houses the platform vector-store implementations (Qdrant, PGVector,
mock). Domain crates consume only the traits in `refractive_swan_vector_port`; runtime
adapters call into this crate to build vector contexts.

---

## Domain / Store / Runtime Layer Mapping

The vector subsystem follows a strict three-layer architecture:

| Layer | Location | Crate | Responsibilities |
|-------|----------|-------|------------------|
| **Domain** | `lib/domain/vector_port` | `refractive_swan_vector_port` | Abstract traits, errors, embedding metadata |
| **Store** | `lib/platform/store/vector_store` | `refractive_swan_vector_store` | Qdrant/PGVector/Milvus configs & drivers |
| **Runtime** | `lib/platform/mesh/node` (target) | `refractive_swan_mesh_node` | Choose backend, build pool/context per node |

### Current Reality

- **Domain**: `refractive_swan_vector_port` lives under `lib/domain/ports/data/data-store/vector`.
- **Store**: `refractive_swan_vector_store` now lives here under `platform/data/data-stores/vector_store`.
- **Runtime**: `NodeDataPlane` (in `refractive_swan_mesh_node`) consumes this crate; HTTP adapters (CLI/API) simply pass configs through.

### Contract

Domain crates (`refractive_swan_mapping`, `refractive_swan_pipeline`, `refractive_swan_observability`) **must** depend only on `refractive_swan_vector_port`, never `refractive_swan_vector_store` directly. This ensures:

1. **Testability**: Domain logic can use mock implementations
2. **Backend flexibility**: Nodes can choose Qdrant, PGVector, Milvus, or custom backends
3. **Migration safety**: Moving `refractive_swan_vector_store` doesn't break domain code

---

## Components

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

To keep `refractive_swan_mesh_node` (runtime) ignorant of backend specifics, `refractive_swan_vector_store` will provide a factory:

```rust
// In refractive_swan_vector_store (future)
use refractive_swan_vector_port::{VectorStore, VectorBackend};
use std::sync::Arc;

/// Runtime configuration for vector store backends.
pub struct VectorStoreRuntimeConfig {
    pub backend: VectorBackend, // from refractive_swan_vector_port
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
// In refractive_swan_mesh_node (future) or refractive_swan_api (current)
use refractive_swan_vector_store::{from_config, VectorStoreRuntimeConfig};
use refractive_swan_vector_port::VectorBackend;

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
            vector_runtime: refractive_swan_vector_store::from_config(vector_cfg),
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
- Domain code (`refractive_swan_mapping`) is identical across nodes
- Hub orchestration (`refractive_swan_mesh_hub`) doesn't care about backend
- Migration/upgrades happen per-node without service-wide disruption

---

## Migration Plan (MESH-025)

- **Phase 1 (complete)** – target location documented.
- **Phase 2 (complete)** – alias crate allowed downstream crates to switch paths.
- **Phase 3 (complete)** – implementation now physically resides under `platform/data/data-stores/vector_store`.
- **Phase 4 (in progress)** – `refractive_swan_mesh_node` consumes this crate; `refractive_swan_api` is a thin HTTP adapter.

---

## Testing

### Unit Tests (in `refractive_swan_vector_store`)

- Backend-specific query translation
- Connection pool/client lifecycle
- Error mapping (Qdrant/PGVector errors → `VectorStoreError`)

### Integration Tests (in `refractive_swan_vector_store`)

- Real Qdrant connectivity (requires Docker/env)
- Real PGVector connectivity (requires Postgres + pgvector extension)
- Health checks, capacity snapshots

### Domain Tests (in `refractive_swan_mapping`, `refractive_swan_pipeline`)

- Use `MockBackend` for fast, deterministic tests
- Never depend on `refractive_swan_vector_store` directly

---

## References

- **Domain Port**: `lib/domain/ports/data/data-store/vector/README.md`
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
- **Node Runtime Design**: `docs/system-design/mesh/node-runtime.md`
