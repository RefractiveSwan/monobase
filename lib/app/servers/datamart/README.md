# dfps_datamart (warehouse adapter)

Outbound adapter that maps `PipelineOutput` rows into the SQLite warehouse schema
and exposes analytics queries (NCIt summary + cohort explorer). This crate owns
the SQL schema, load routines, and the `DatamartSink` port used by `dfps_api`
and `dfps_cli`.

## Architecture

- `port.rs` defines the `DatamartSink` trait plus the `SqliteDatamart`
  implementation. The sink lazily connects to the configured SQLite database,
  runs migrations, and exposes async methods for `persist`, `ncit_summary`, and
  `cohort`. When warehouse env is missing the sink reports `DatamartError::Disabled`
  so callers can opt out of analytics.
- `sql.rs` contains the schema DDL, config loader (`WarehouseConfig::from_env`),
  and the raw SQL used by the sink.
- `dim.rs` / `fact.rs` / `keys.rs` convert `PipelineOutput` (re-exported from
  `dfps_contracts`) into warehouse-friendly dimension + fact rows. These helpers
  are pure domain mappers so the outbound adapter stays thin.

## Env

Env is loaded via `dfps_configuration::load_env("domain.datamart")`:

- `DFPS_WAREHOUSE_URL` (required) – SQLite connection string.
- `DFPS_WAREHOUSE_SCHEMA` (optional) – future hook for non-SQLite targets.
- `DFPS_WAREHOUSE_MAX_CONNECTIONS` (optional, default `5`) – connection pool size.

When the env namespace is absent, `SqliteDatamart::from_env()` returns a disabled
sink so `dfps_api`/CLI can continue operating without analytics persistence.
