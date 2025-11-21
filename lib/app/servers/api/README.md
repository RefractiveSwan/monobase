# dfps_api (Axum backend)

Endpoints:
- `POST /api/map-bundles` – accepts Bundles/arrays/NDJSON and returns the canonical `PipelineOutput` contract.
- `/analytics/ncit-summary`, `/analytics/cohort` – warehouse-backed reporting aligned with the same DTOs consumed by the web frontend.
- `/api/eval/*` – dataset listing/summary/run endpoints backed by `dfps_eval`.
- `/metrics/summary`, `/health` – operational surfaces for frontend/CLI health checks.

Env: loaded via `dfps_configuration::load_env("app.web.api")`. Defaults:
`DFPS_API_HOST=127.0.0.1`, `DFPS_API_PORT=8080`. Warehouse + compliance +
vector configuration reuse the same env namespaces documented in
`dfps_configuration` (`DFPS_WAREHOUSE_*`, `DFPS_COMPLIANCE_*`,
`DFPS_VECTOR_*`, etc.).

## Architecture

- `server.rs` wires the Axum router to a `NodeDataPlane`. The plane holds the
  injected ports:
  - `dfps_pipeline::DefaultPipeline` (`PipelinePort`) for Bundle → `PipelineOutput` orchestration.
  - `dfps_datamart::SqliteDatamart` (`DatamartSink`) for persistence + analytics
    queries. When warehouse env is absent the sink reports `DatamartError::Disabled`
    so handlers can degrade gracefully.
  - Shared policy, dataset store, and metrics handles so transports stay thin.
- Handlers translate HTTP payloads into the canonical `dfps_contracts` DTOs.
  No bespoke structs live in this crate; the frontend/CLI deserialize identical
  payloads for analytics/eval/mapping.

## Relation to dfps_mesh_node

**Current status**: `dfps_api` serves as the **proto-node runtime** for the mesh architecture.

### NodeDataPlane

The `NodeDataPlane` struct in `server.rs` is the early implementation of what will become `dfps_mesh_node::NodeDataPlane`:

```rust
pub struct NodeDataPlane {
    pipeline: Arc<dyn PipelinePort>,
    datamart: Arc<dyn DatamartSink>,
    vector_runtime: Arc<dyn VectorStore>,
    policy: Policy,
    dataset_store: Arc<dyn DatasetStore>,
    metrics: Arc<Mutex<PipelineMetrics>>,
}
```

**Future**: This struct will be moved to `lib/platform/mesh/node` as `dfps_mesh_node::NodeDataPlane`.

### ApiState ~ NodeDataPlane

`ApiState` wraps `NodeDataPlane` for Axum's state management:

```rust
pub struct ApiState {
    plane: NodeDataPlane,
}
```

**Future**: HTTP state will be separated from business logic. `dfps_api` will become a thin HTTP adapter over `dfps_mesh_node`.

### Migration Path

The migration follows the MESH-025 plan (see `docs/system-design/mesh/migration-plan.md`):

1. **Phase 1-3**: `dfps_api` remains unchanged while design docs and skeleton crates are created
2. **Phase 4**: Extract `NodeDataPlane` to `dfps_mesh_node`, `dfps_api` becomes a compatibility shim

### Mesh Coordination (Future)

Once `dfps_mesh_node` is extracted, it will gain:
- `MeshNodeId`: Unique node identifier
- `NodeCapabilities`: Advertise backends and compliance mode to hub
- Governance integration: Evaluate policies before executing jobs
- Hub registration: Optional registration with `dfps_mesh_hub` for federated deployments

**Current**: Node operates standalone, no mesh coordination  
**Future**: Node can participate in federated mesh deployments

### References

- **Node Runtime Design**: `docs/system-design/mesh/node-runtime.md`
- **Mesh Contracts**: `lib/domain/contracts/src/mesh.rs`
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`

---

Run:
```bash
cd code
cargo run -p dfps_api --bin dfps_api
```
