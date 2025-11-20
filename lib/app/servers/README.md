# App Servers (dfps_web_backend)

This directory houses the HTTP and warehouse adapters that live in the app layer.
Each crate owns a specific boundary and only wires domain/platform ports together.

## `dfps_api` (`lib/app/servers/api`)

- Acts as the single inbound HTTP gateway (Axum) for bundle mapping, eval orchestration,
  and analytics queries.
- Reads configuration from the `app.web.api` namespace via `dfps_configuration`. Runtime
  settings include:
  - Network: `DFPS_API_HOST`, `DFPS_API_PORT`.
  - Dataset store root: `DFPS_EVAL_DATA_ROOT` (relative paths resolve against the workspace).
  - Vector store wiring: `platform.vector_store` (`DFPS_VECTOR_*`).
  - Compliance policy: `platform.compliance` (`DFPS_COMPLIANCE_*`).
  - Warehouse (optional): `domain.datamart` (`DFPS_WAREHOUSE_*`) – disables analytics routes
    when not provided.
- The `NodeDataPlane` struct owns the injected policy, dataset store, datamart sink, vector
  context, metrics counters, and pipeline port; handlers only talk to these interfaces.

## `dfps_datamart` (`lib/app/servers/datamart`)

- Outbound adapter for persisting/querying analytics data. Provides `DatamartSink` (SQLite)
  plus dim/fact builders.
- Env namespace: `domain.datamart` with `DFPS_WAREHOUSE_URL`, `DFPS_WAREHOUSE_SCHEMA`,
  `DFPS_WAREHOUSE_MAX_CONNECTIONS`.
- Consumed by `dfps_api` and CLI tooling (`load_datamart`). When the warehouse URL is absent
  the sink stays disabled, allowing analytics calls to short-circuit gracefully.

## `dfps_vector_store` (`lib/app/servers/vector_store`)

- Platform adapters for vector backends (Qdrant, pgvector, mocks) behind the `dfps_vector_port`
  trait.
- Shares the `platform.vector_store` namespace (`DFPS_VECTOR_*`) and exposes typed configs used
  by both API and CLI surfaces.
