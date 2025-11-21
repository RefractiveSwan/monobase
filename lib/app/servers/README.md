# App Servers (refractive_swan_web_backend)

This directory houses the HTTP and warehouse adapters that live in the app layer.
Each crate owns a specific boundary and only wires domain/platform ports together.

## `refractive_swan_api` (`lib/app/servers/api`)

- Acts as the single inbound HTTP gateway (Axum) for bundle mapping, eval orchestration,
  and analytics queries.
- Reads configuration from the `app.web.api` namespace via `refractive_swan_configuration`. Runtime
  settings include:
  - Network: `refractive_swan_API_HOST`, `refractive_swan_API_PORT`.
  - Dataset store root: `refractive_swan_EVAL_DATA_ROOT` (relative paths resolve against the workspace).
  - Vector store wiring: `platform.vector_store` (`refractive_swan_VECTOR_*`).
  - Compliance policy: `platform.compliance` (`refractive_swan_COMPLIANCE_*`).
  - Warehouse (optional): `domain.datamart` (`refractive_swan_WAREHOUSE_*`) – disables analytics routes
    when not provided.
- The `NodeDataPlane` struct owns the injected policy, dataset store, datamart sink, vector
  context, metrics counters, and pipeline port; handlers only talk to these interfaces.

## `refractive_swan_datamart`

- **Location:** `lib/platform/data/data-plane/mart`
- Outbound adapter for persisting/querying analytics data. Provides `DatamartSink` (SQLite)
  plus dim/fact builders.
- Env namespace: `domain.datamart` with `refractive_swan_WAREHOUSE_URL`, `refractive_swan_WAREHOUSE_SCHEMA`,
  `refractive_swan_WAREHOUSE_MAX_CONNECTIONS`.
- Consumed by `refractive_swan_api` and CLI tooling (`load_datamart`). When the warehouse URL is absent
  the sink stays disabled, allowing analytics calls to short-circuit gracefully.

## `refractive_swan_vector_store`

- **Location:** `lib/platform/data/data-stores/vector_store`
- Platform adapters for vector backends (Qdrant, pgvector, mocks) behind the `refractive_swan_vector_port`
  trait.
- Shares the `platform.vector_store` namespace (`refractive_swan_VECTOR_*`) and exposes typed configs used
  by both API and CLI surfaces.
