# dfps_cli

## `map_bundles`
Reads FHIR Bundles (JSON/array/NDJSON) from a file or stdin and emits
`staging_flat`, `staging_code`, `mapping_result`, and `dim_concept` rows (NDJSON).

```bash
cd code
cargo run -p dfps_cli --bin map_bundles -- <input.ndjson>
````

## `map_codes`

Reads `StgSrCodeExploded` rows and outputs mapping results. Use `--explain` to emit
ranked candidate explanations.

```bash
cd code
cargo run -p dfps_cli --bin map_codes -- --explain --explain-top 5 <codes.ndjson>
```

## `eval_mapping`

Evaluates the NCIt mapping pipeline against a gold-standard NDJSON file made of
`EvalCase` rows. Emits a JSON summary plus optional per-case details. You can point
to a named dataset under `DFPS_EVAL_DATA_ROOT` via `--dataset` or provide a direct
NDJSON path via `--input`.

```bash
cd code
cargo run -p dfps_cli --bin eval_mapping -- --dataset pet_ct_small --dump-details

# Gate CI with thresholds
cargo run -p dfps_cli --bin eval_mapping -- \
  --dataset pet_ct_small \
  --thresholds lib/domain/evaluation/fake_data/data/meta/eval_thresholds.json

# Persist machine-readable artifacts (summary/results) and markdown report
cargo run -p dfps_cli --bin eval_mapping -- \
  --dataset gold_pet_ct_comprehensive \
  --out-dir target/eval \
  --report target/eval/report.md \
  --dump-details
```
