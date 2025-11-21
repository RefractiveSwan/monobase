# dfps_mesh_node

**Conceptual location:** `lib/platform/mesh/node`  
**Current physical location:** Currently embedded in `lib/app/servers/api` as `NodeDataPlane`  
**Scope:** Mesh node runtime, data plane orchestration, HTTP API surface

This directory represents the **planned home** for the mesh node runtime. The current implementation lives in `dfps_api` and will be extracted here during Phase 4 of the MESH-025 migration.

---

## Purpose

See `docs/system-design/mesh/node-runtime.md` for detailed design.

### Quick Summary

`dfps_mesh_node` is the **node runtime** that:

1. Wires domain (`dfps_pipeline`) + data (`dfps_datamart`) + store (`dfps_vector_store`, `dfps_relational_store`)
2. Exposes HTTP APIs for mapping, analytics, eval
3. Enforces governance policies (`dfps_mesh_governance`)
4. Optionally registers with hub (`dfps_mesh_hub`)

---

## Current Reality

**Location**: `lib/app/servers/api/src/server.rs`  
**Struct**: `NodeDataPlane`

Today, the node runtime is embedded in `dfps_api` with Axum HTTP handlers in the same crate. This works but couples business logic with transport.

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

## Migration Path

### Phase 1: Conceptual Design

- `docs/system-design/mesh/node-runtime.md` describes target architecture
- `dfps_api` remains current implementation

### Phase 2: Extract NodeDataPlane

- Move business logic from `dfps_api::server` to `dfps_mesh_node::data_plane`
- `dfps_api` becomes thin HTTP adapter (imports `NodeDataPlane` from `dfps_mesh_node`)

### Phase 3: Add Mesh Coordination

- Implement `MeshNodeId`, `NodeCapabilities`
- Add `/mesh/job`, `/mesh/capabilities`, `/mesh/health` endpoints
- Integrate `dfps_mesh_governance`

### Phase 4: Deprecate dfps_api

- Create `dfps_api` shim that wraps `dfps_mesh_node`
- Existing deployments continue working (backward compat)

---

## References

- **Design Document**: `docs/system-design/mesh/node-runtime.md`
- **Current Implementation**: `lib/app/servers/api/src/server.rs`
- **Mesh Contracts**: `lib/domain/contracts/src/mesh.rs`
- **Governance**: `lib/platform/mesh/governance/README.md`
- **Hub**: `lib/platform/mesh/hub/README.md`
