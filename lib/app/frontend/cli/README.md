# refractive_swan_cli

Thin orchestration layer over the domain + platform crates. Every binary loads the
`app.cli` env namespace, initializes logging through the shared `cli_core` module,
and emits NDJSON records with a `{ "kind": "...", "value": { ... } }` envelope so
downstream tooling can parse output consistently.

## Architecture

- `src/cli_core` – reusable helpers for env/logging, NDJSON streaming, compliance
  guards, vector config, and exit-code aware errors. Each bin calls `run_bin` so
  failures exit with a stable `ExitCode` (config, invalid input, compliance,
  external, etc.).
- `refractive_swan_pipeline::DefaultPipeline` implements the `PipelinePort` trait; CLI bins
  treat it as the inbound hexagonal port so orchestration stays in the domain
  layer while IO/NDJSON parsing stays in CLI adapters.
- Binaries stream input via `json_stream`, so large NDJSON files and stdin pipes
  are processed incrementally without buffering everything in memory.
- Shared compliance helpers mirror API/web behavior (e.g., `--fail-on-license-block`
  uses the same policy enforcement as `refractive_swan_api`). Vector helpers wrap
  `config_from_env` to keep `map_bundles`, `map_codes`, and
  `build_vector_index` in sync.

## Common flags & env

- `refractive_swan_COMPLIANCE_MODE` / `--fail-on-license-block` – CLI exits with the configured
  exit code if compliance policy blocks records (same semantics as API).
- `refractive_swan_VECTOR_*` – vector backend configuration shared with API + vector-store crate.
- `RUST_LOG` (or `--log-level` on `map_bundles`) – logging levels for the pipeline.
- `refractive_swan_EVAL_DATA_ROOT` – dataset root for `eval_mapping` (matches API/eval endpoints).
- `refractive_swan_FHIR_VALIDATOR_*` – external validation parameters for `validate_fhir`.
- `refractive_swan_WAREHOUSE_*` – warehouse/sqlite config for `load_datamart` (documented in
  `refractive_swan_datamart`).

## Binaries → domain flows

| Binary | Flow | Notes |
| --- | --- | --- |
| `map_bundles` | `refractive_swan_ingestion::validation` → `refractive_swan_pipeline::DefaultPipeline::map_bundle` (via `PipelinePort`) → `refractive_swan_observability::PipelineMetrics` | Streams Bundles/NDJSON, emits per-row artifacts plus `pipeline_output`/`metrics_summary` records; shared compliance + vector helpers. |
| `map_codes` | `refractive_swan_mapping::{map_staging_codes_with_vector_and_policy,map_staging_codes_with_summary_and_policy}` | Shares vector config with `build_vector_index`; explanation rows use the same contract as API. |
| `eval_mapping` | `refractive_swan_eval::run_eval_streaming_with_mapper` (lexical mapping) | Dataset loading/reporting matches API eval endpoints; writes `eval_summary`/`eval_result` NDJSON for downstream dashboards. |
| `validate_fhir` | `refractive_swan_ingestion::validation::{validate_bundle_with_external_profile}` | Modes (`lenient`, `strict`, `external_preferred`, `external_strict`) align with ingestion docs and API options; emits `validation_issue` + `validation_summary`. |
| `load_datamart` | `refractive_swan_pipeline` (optional) → `refractive_swan_datamart::load_from_pipeline_output` | Handles `PipelineOutput` or raw Bundles (mapped on the fly) and shares export-policy/compliance semantics with API/datamart loaders. |
| `build_vector_index` | `refractive_swan_mapping::load_ncit_concepts` → `refractive_swan_vector_store::{Mock,Qdrant,PgVector}` | Structured CLI errors replace panics; embedding metadata/version logging matches vector docs. |

## Usage examples

### `map_bundles`

Reads FHIR Bundles (JSON/array/NDJSON) from a file or stdin, validates them, and
streams `pipeline_output`, `staging_flat`, `staging_code`, `mapping_result`,
`dim_concept`, `validation_issue`, and `metrics_summary` records. CLI adapters
call the `PipelinePort` trait so domain orchestration remains testable and
transport-agnostic.

```bash
cd code
cargo run -p refractive_swan_cli --bin map_bundles -- ./bundles.ndjson \
  --log-level debug \
  --fail-on-license-block
```

### `map_codes`

Reads `StgSrCodeExploded` rows and outputs mapping results + optional explanation
records. Uses the same compliance + vector config helpers as `map_bundles` /
`build_vector_index`.

```bash
cargo run -p refractive_swan_cli --bin map_codes -- \
  --explain --explain-top 5 < codes.ndjson
```

### `eval_mapping`

Evaluates the mapping pipeline against gold NDJSON or a named dataset under
`refractive_swan_EVAL_DATA_ROOT`. Emits `eval_summary` (with manifest + metrics) and optional
`eval_result` rows.

```bash
cargo run -p refractive_swan_cli --bin eval_mapping -- --dataset pet_ct_small --dump-details

# Gate CI with thresholds
cargo run -p refractive_swan_cli --bin eval_mapping -- \
  --dataset pet_ct_small \
  --thresholds lib/domain/meta/evaluation/data/meta/eval_thresholds.json

# Persist machine-readable artifacts and an HTML report
cargo run -p refractive_swan_cli --bin eval_mapping -- \
  --dataset gold_pet_ct_comprehensive \
  --out-dir target/eval \
  --report target/eval/report.html \
  --dump-details
```

### `validate_fhir`

Validates Bundles using ingestion rules and optional external `$validate` calls.
Outputs `validation_issue` + `validation_summary` rows consumable by CI dashboards.

```bash
cargo run -p refractive_swan_cli --bin validate_fhir -- --mode external_preferred ./bundle.json
```

### `load_datamart`

Loads `PipelineOutput` or Bundle NDJSON into the SQLite warehouse schema while
reusing `refractive_swan_datamart` export-policy controls.

```bash
cargo run -p refractive_swan_cli --bin load_datamart -- \
  --input target/pipeline_output.ndjson \
  --input-kind pipeline
```

### `build_vector_index`

Builds (or rebuilds) the NCIt vector index for the configured backend.

```bash
cargo run -p refractive_swan_cli --bin build_vector_index -- \
  --limit 5000 \
  --embedding-version deterministic-hash-v1
```
