# Platform crates

Platform crates sit between the pure domain layers and app surfaces. They host
runtime adapters (env/config, observability/logging, compliance guards, shared
test fixtures) that need to touch the operating system or external services,
while keeping the domain crates completely environment-free.

Add a new platform crate when:
- The code depends on environment variables, filesystem paths, or process-level
  configuration (e.g., loading `.env` namespaces, wiring SQL pools).
- Multiple apps/CLIs need to share the same adapter logic (e.g., compliance
  policy loading, vector-store configs, datamart helpers, test utilities).
- The functionality represents “infrastructure glue” rather than pure domain
  behavior.

Prefer updating an existing domain crate when the change only affects pure
business logic or type modeling—they should never read from `std::env` or touch
I/O directly.

## Env namespaces

Each platform crate owns a namespace under `data/environment/` so we can source
settings consistently via `refractive_swan_configuration`:

| Namespace | Crate | Purpose |
|-----------|-------|---------|
| `platform.configuration` | `refractive_swan_configuration` | Workspace env loader + helpers (`string_var`, `bool_var`, `u32_var`, `workspace_root`, etc.) |
| `platform.compliance` | `refractive_swan_compliance` | Compliance policy config (`refractive_swan_COMPLIANCE_*`) + structured overrides |
| `platform.vector_store` | `refractive_swan_vector_store` | Vector backend config (`refractive_swan_VECTOR_*`) shared by apps/CLIs |
| `platform.observability` | `refractive_swan_observability` | Logging + metrics env (`OBS_ENV`, analytics counters) |
| `platform.test_suite` | `refractive_swan_test_suite` | Integration test scaffolding (`refractive_swan_EVAL_DATA_ROOT`, CLI helpers) |

Use `refractive_swan_configuration::load_env("<namespace>")` before reading env vars so
callers get consistent overrides (`refractive_swan_WORKSPACE_ROOT`, `refractive_swan_ENV_DIR`, etc.).

## Shared helpers

- **Configuration** – `refractive_swan_configuration` exposes typed env readers
  (`string_var`, `bool_var`, `u32_var`, `port_var`) so platform/app crates avoid
  bespoke parsing logic.
- **Compliance** – `ComplianceConfig::from_env()` captures policy mode + override
  files once, with `Policy::default_for_mode` providing the baseline guardrails.
- **Vector store** – `refractive_swan_vector_store::config_from_env()` normalizes backend
  configuration and returns a validated `VectorStoreConfig`.
- **Observability** – `refractive_swan_observability::init_environment()` loads the logging
  namespace once and `PipelineMetrics` keeps analytics counters aligned across
  apps, CLIs, and tests.
- **Test suite** – `refractive_swan_test_suite` supplies CLI and datamart helpers (temp
  SQLite warehouses, regression bundles, scoped env overrides) so downstream
  integration tests do not re-implement fixture logic.

Keep these crates small, typed, and well-documented; they are the seam between
application glue and the domain core.
