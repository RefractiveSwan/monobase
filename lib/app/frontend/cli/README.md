# dfps_cli

Thin orchestration layer over the domain + platform crates. Every binary loads the
`app.cli` env namespace, initializes logging through the shared `cli_core` module,
and emits NDJSON records with a `{ "kind": "...", "value": { ... } }` envelope so
downstream tooling can parse output consistently.

## Architecture

- `src/cli_core` – reusable helpers for env/logging, NDJSON streaming, compliance
  guards, vector config, and exit-code aware errors. Each bin calls `run_bin` so
  failures exit with a stable `ExitCode` (config, invalid input, compliance,
  external, etc.).
- `dfps_pipeline::DefaultPipeline` implements the `PipelinePort` trait; CLI bins
  treat it as the inbound hexagonal port so orchestration stays in the domain
  layer while IO/NDJSON parsing stays in CLI adapters.
- Binaries stream input via `json_stream`, so large NDJSON files and stdin pipes
  are processed incrementally without buffering everything in memory.
- Shared compliance helpers mirror API/web behavior (e.g., `--fail-on-license-block`
  uses the same policy enforcement as `dfps_api`). Vector helpers wrap
  `config_from_env` to keep `map_bundles`, `map_codes`, and
  `build_vector_index` in sync.

## Common flags & env

- `DFPS_COMPLIANCE_MODE` / `--fail-on-license-block` – CLI exits with the configured
  exit code if compliance policy blocks records (same semantics as API).
- `DFPS_VECTOR_*` – vector backend configuration shared with API + vector-store crate.
- `RUST_LOG` (or `--log-level` on `map_bundles`) – logging levels for the pipeline.
- `DFPS_EVAL_DATA_ROOT` – dataset root for `eval_mapping` (matches API/eval endpoints).
- `DFPS_FHIR_VALIDATOR_*` – external validation parameters for `validate_fhir`.
- `DFPS_WAREHOUSE_*` – warehouse/sqlite config for `load_datamart` (documented in
  `dfps_datamart`).

## Binaries → domain flows

| Binary | Flow | Notes |
| --- | --- | --- |
| `map_bundles` | `dfps_ingestion::validation` → `dfps_pipeline::DefaultPipeline::map_bundle` (via `PipelinePort`) → `dfps_observability::PipelineMetrics` | Streams Bundles/NDJSON, emits per-row artifacts plus `pipeline_output`/`metrics_summary` records; shared compliance + vector helpers. |
| `map_codes` | `dfps_mapping::{map_staging_codes_with_vector_and_policy,map_staging_codes_with_summary_and_policy}` | Shares vector config with `build_vector_index`; explanation rows use the same contract as API. |
| `eval_mapping` | `dfps_eval::run_eval_streaming_with_mapper` (lexical mapping) | Dataset loading/reporting matches API eval endpoints; writes `eval_summary`/`eval_result` NDJSON for downstream dashboards. |
| `validate_fhir` | `dfps_ingestion::validation::{validate_bundle_with_external_profile}` | Modes (`lenient`, `strict`, `external_preferred`, `external_strict`) align with ingestion docs and API options; emits `validation_issue` + `validation_summary`. |
| `load_datamart` | `dfps_pipeline` (optional) → `dfps_datamart::load_from_pipeline_output` | Handles `PipelineOutput` or raw Bundles (mapped on the fly) and shares export-policy/compliance semantics with API/datamart loaders. |
| `build_vector_index` | `dfps_mapping::load_ncit_concepts` → `dfps_vector_store::{Mock,Qdrant,PgVector}` | Structured CLI errors replace panics; embedding metadata/version logging matches vector docs. |

## Usage examples

### `map_bundles`

Reads FHIR Bundles (JSON/array/NDJSON) from a file or stdin, validates them, and
streams `pipeline_output`, `staging_flat`, `staging_code`, `mapping_result`,
`dim_concept`, `validation_issue`, and `metrics_summary` records. CLI adapters
call the `PipelinePort` trait so domain orchestration remains testable and
transport-agnostic.

```bash
cd code
cargo run -p dfps_cli --bin map_bundles -- ./bundles.ndjson \
  --log-level debug \
  --fail-on-license-block
```

### `map_codes`

Reads `StgSrCodeExploded` rows and outputs mapping results + optional explanation
records. Uses the same compliance + vector config helpers as `map_bundles` /
`build_vector_index`.

```bash
cargo run -p dfps_cli --bin map_codes -- \
  --explain --explain-top 5 < codes.ndjson
```

### `eval_mapping`

Evaluates the mapping pipeline against gold NDJSON or a named dataset under
`DFPS_EVAL_DATA_ROOT`. Emits `eval_summary` (with manifest + metrics) and optional
`eval_result` rows.

```bash
cargo run -p dfps_cli --bin eval_mapping -- --dataset pet_ct_small --dump-details

# Gate CI with thresholds
cargo run -p dfps_cli --bin eval_mapping -- \
  --dataset pet_ct_small \
  --thresholds lib/domain/evaluation/fake_data/data/meta/eval_thresholds.json

# Persist machine-readable artifacts and an HTML report
cargo run -p dfps_cli --bin eval_mapping -- \
  --dataset gold_pet_ct_comprehensive \
  --out-dir target/eval \
  --report target/eval/report.html \
  --dump-details
```

### `validate_fhir`

Validates Bundles using ingestion rules and optional external `$validate` calls.
Outputs `validation_issue` + `validation_summary` rows consumable by CI dashboards.

```bash
cargo run -p dfps_cli --bin validate_fhir -- --mode external_preferred ./bundle.json
```

### `load_datamart`

Loads `PipelineOutput` or Bundle NDJSON into the SQLite warehouse schema while
reusing `dfps_datamart` export-policy controls.

```bash
cargo run -p dfps_cli --bin load_datamart -- \
  --input target/pipeline_output.ndjson \
  --input-kind pipeline
```

### `build_vector_index`

Builds (or rebuilds) the NCIt vector index for the configured backend.

```bash
cargo run -p dfps_cli --bin build_vector_index -- \
  --limit 5000 \
  --embedding-version deterministic-hash-v1
```
