# refractive_swan_mesh_node

**Conceptual location:** `lib/platform/mesh/node`  
**Current physical location:** Crate exists here; HTTP handlers still live in `refractive_swan_api`  
**Scope:** Mesh node runtime, data plane orchestration, HTTP API surface

This directory represents the **planned home** for the mesh node runtime. The current implementation lives in `refractive_swan_api` and will be extracted here during Phase 4 of the MESH-025 migration.

---

## Purpose

See `docs/system-design/mesh/node-runtime.md` for detailed design.

### Quick Summary

`refractive_swan_mesh_node` is the **node runtime** that:

1. Wires domain (`refractive_swan_pipeline`) + data (`refractive_swan_datamart`) + store (`refractive_swan_vector_store`, `refractive_swan_relational_store`)
2. Exposes HTTP APIs for mapping, analytics, eval
3. Enforces governance policies (`refractive_swan_mesh_governance`)
4. Optionally registers with hub (`refractive_swan_mesh_hub`)

---

## Current Reality

**Location**: `lib/app/servers/api/src/server.rs`  
**Struct**: `NodeDataPlane`

Today, the node runtime is embedded in `refractive_swan_api` with Axum HTTP handlers in the same crate. This works but couples business logic with transport.

---

## Future Structure

```
lib/platform/mesh/node/
  src/
    data_plane.rs        # NodeDataPlane struct
    config.rs            # NodeConfig, env loading
    http/                # HTTP adapter (Axum/Actix)
      routes.rs
      handlers.rs
    lib.rs
  Cargo.toml
  README.md (this file)
```

---

## Current Status

- `src/plane.rs` defines the reusable `NodeDataPlane` struct backed by the mesh
  DTO veneer (`refractive_swan_mesh_dto`).
- `src/config.rs` exposes `NodePlaneConfig::from_env` so HTTP adapters can reuse
  the same policy/dataset/vector/datamart wiring logic.
- `refractive_swan_api` imports this crate today; future mesh runtimes will reuse the same
  type instead of embedding their own orchestration logic.

---

## Migration Path

### Phase 1: Conceptual Design

- `docs/system-design/mesh/node-runtime.md` describes target architecture
- `refractive_swan_api` remains current implementation

### Phase 2: Extract NodeDataPlane

- Move business logic from `refractive_swan_api::server` to `refractive_swan_mesh_node::plane`
- `refractive_swan_api` becomes thin HTTP adapter (imports `NodeDataPlane`)

### Phase 3: Add Mesh Coordination

- Implement `MeshNodeId`, `NodeCapabilities` (imported via `refractive_swan_mesh_dto`)
- Add `/mesh/job`, `/mesh/capabilities`, `/mesh/health` endpoints
- Integrate `refractive_swan_mesh_governance`

### Phase 4: Deprecate refractive_swan_api

- Create `refractive_swan_api` shim that wraps `refractive_swan_mesh_node`
- Existing deployments continue working (backward compat)

---

## References

- **Design Document**: `docs/system-design/mesh/node-runtime.md`
- **Current Implementation**: `lib/app/servers/api/src/server.rs`
- **Mesh DTO Veneer**: `lib/dto/mesh` (`refractive_swan_mesh_dto`)
- **Canonical Contracts**: `lib/domain/meta/contracts/src/mesh.rs`
- **Governance**: `lib/platform/mesh/governance/README.md`
- **Hub**: `lib/platform/mesh/hub/README.md`
