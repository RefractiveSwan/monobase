# refractive_swan_api (Axum backend)

Production-grade overview of the HTTP gateway that fronts the FHIR -> NCIt pipeline, analytics, and dataset admin surfaces.

## Endpoints

- Namespaces: `/api/v1/**` (REST), `/api/v2/**` (GraphQL placeholder), `/api/v3/**` (gRPC placeholder).

### (v1 REST)

- `POST /api/map-bundles` – Ingest Bundle object/array/NDJSON -> `PipelineOutput` contract + validation reports.
- `GET /metrics/summary`, `GET /health` – Ops probes, emits `PipelineMetrics`.
- Analytics: `GET /analytics/ncit-summary`, `GET /analytics/cohort`.
- Eval: `GET /api/eval/datasets`, `GET /api/eval/summary?dataset=…`, `POST /api/eval/run`, `GET /api/eval/latest`.
- Dataset admin: `POST /api/datasets/refresh`, `POST /api/datasets/upload`, `DELETE /api/datasets/:name`.
- Admin/insights: `GET /admin/terminology/insights`, `GET /admin/compliance/policy`, `GET /admin/ingestion/summary`, `GET /admin/vector/status`, `GET /admin/datamart/health`, `GET /admin/events`, `GET /admin/toggles`.

## Configuration

- Env loaded via `refractive_swan_configuration::load_env("app.web.api")`.
- Defaults: `refractive_swan_API_HOST=127.0.0.1`, `refractive_swan_API_PORT=8080`.
- Reuses shared namespaces: compliance (`refractive_swan_COMPLIANCE_*`), datamart/warehouse (`refractive_swan_WAREHOUSE_*`), vector (`refractive_swan_VECTOR_*`), eval data root (`refractive_swan_EVAL_DATA_ROOT`), etc.
- Code: `utils/config.rs` defines `ApiConfig`/`ApiServerConfig` with `from_env_or_exit()`.

## Architecture

- **Server entry**: `server.rs` builds the router and applies state via `router_with_state(ApiState)`. Served with `tower::make::Shared`.
- **State**: `utils/state.rs` owns `ApiState` (NodeDataPlane + metrics + dataset registry + admin events).
- **Error/response**: `utils/error.rs` defines `ApiError`/`ServerError` + `ErrorResponse`.
- **Controllers**: `server/controllers/**` contain the business logic per domain (infra, analytics, mapping, eval, datasets, admin).
- **Handlers**: `server/handlers/**` are thin Axum adapters delegating to controllers.
- **Types**: `types/**` holds request/response helpers for handlers/controllers.
- **Routing**: `server/routes/**` assembles versioned routers (v1 REST today).

## Mesh alignment

- Uses `refractive_swan_mesh_node::NodeDataPlane` (via `ApiState::from_plane_config`) to keep compliance, dataset store, vector context, and datamart wiring consistent with the mesh runtime plans.
- Mesh IDs exposed in metrics (`mesh_node_id`) and admin events to support per-node observability.

## Running locally

```bash
cd code
cargo run -p refractive_swan_api --bin refractive_swan_api
```

## Testing

- Unit/integration coverage lives in `lib/platform/test_suite/tests/integration/api/web_api.rs` (uses `router_with_state`).
- Standard checks: `cargo make fmt`, `cargo make clippy`, `cargo make test`.

## References

- Mesh runtime: `docs/system-design/mesh/node-runtime.md`
- Metrics/observability: `lib/platform/meta/observability`
- Contracts/DTOs: `lib/domain/meta/contracts`, `lib/dto/web`
- Dataset admin: `lib/domain/meta/evaluation` (manifests/NDJSON), `types/datasets.rs`
