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

Run:
```bash
cd code
cargo run -p dfps_api --bin dfps_api
```
