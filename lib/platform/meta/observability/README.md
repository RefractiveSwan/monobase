# dfps_observability – Metrics & logging adapter

Home for shared observability helpers that wire the Bundle → NCIt pipeline into structured logs and JSON metrics. See:

- `docs/system-design/clinical/ncit/behavior/sequence-servicerequest.md`
- `docs/runbook/040-warehouse-and-bi/bi-integration-quickstart.md`
- `docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-12--platform-observability--metrics-dfps_observability`

## Environment loading

- Call `dfps_observability::init_environment()` once at startup to load the `platform.observability` env namespace. The function now returns `Result<(), dfps_configuration::EnvLoadError>` so API/CLI surfaces can log and continue instead of panicking when files are missing.
- Helpers (`log_pipeline_output`, `log_no_match`) internally call `load_env` and log a warning instead of panicking, so pipeline runs remain deterministic even when `.env` files are absent in non-strict mode.
- Standard knobs:
  - `DFPS_ENV` / `APP_ENV` select the profile (defaults to `dev`).
  - `DFPS_ENV_FILE` points to a specific `.env` file when needed for CI.
  - `DFPS_ENV_STRICT=1` forces an error if no env file is found for the namespace/profile.

## Pipeline metrics ownership

`PipelineMetrics` encapsulates bundle/mapping counters plus vector + analytics fields:

- Pipeline/domain layers call `log_pipeline_output` or `PipelineMetrics::record` to track bundle, staging, mapping, `auto_mapped`, `needs_review`, and `no_match` counts. Compliance-related counters (`license_blocked`, `compliance_mode`) are set by the CLI/API right after policy enforcement.
- Vector backends update `vector_queries`, `vector_hits`, `vector_fallbacks`, and capacity proxies via `apply_vector_usage` (fed with `VectorUsageSnapshot` + optional latency).
- Analytics-only fields (`analytics_requests`, `cohort_queries`, `cohort_results_total`, `avg_cohort_size`) are reserved for app/API layers; domain crates should leave them untouched to keep ownership clear.

## Vector usage & capacity helpers

- `VectorUsageSnapshot` carries per-run query/hit/fallback counts plus an optional `VectorCapacitySnapshot` (geom_rm, geom_dm, geom_rm_sqrt_dm, cap_alpha_sim). Backends emit this once per pipeline run.
- `apply_vector_usage(metrics, snapshot, latency_ms_p95)` merges those counters into `PipelineMetrics` and normalizes latency handling, so CLI/API surfaces can stick to one code path.
- `dfps_vector_store::CapacityProxies` implements `Into<VectorCapacitySnapshot>`, keeping the conversion localized to the vector-store crate.

## Metrics snapshots for API/CLI consumers

- `metrics_snapshot(&metrics)` returns a `MetricsSnapshot` struct containing the raw counters plus derived ratios (auto-mapped %, needs-review %, license-blocked %, vector hit/fallback rates).
- `metrics_snapshot_json(&metrics)` serializes that snapshot for `/metrics/summary` responses or CLI exporters—ensuring the same schema is used across app surfaces.

## Logging helpers

- `log_pipeline_output` records bundle/mapping counters, applies vector usage snapshots, and emits a structured info log on the `dfps_pipeline` target.
- `log_no_match` logs a warning for each `MappingResult` in the `NoMatch` state, helping CLI/API logs highlight gaps (`reason` defaults to `no_match` if none is provided).
