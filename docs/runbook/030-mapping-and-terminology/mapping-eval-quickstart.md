# Mapping Evaluation Quickstart

This runbook walks through running the mapping evaluation harness (Epic EVAL-012)
against the gold NDJSON fixtures.

## Prerequisites
- Rust toolchain (install via `data/scripts/install_rust_tooling.sh`).
- Gold dataset: `lib/domain/fake_data/data/eval/pet_ct_small.ndjson` (or your custom NDJSON with `EvalCase` rows). Override the root with `DFPS_EVAL_DATA_ROOT` if you keep datasets elsewhere. Each dataset ships with `<name>.manifest.json`; the CLI validates the SHA-256 listed there and warns if a checksum drifts.
- Tiered splits: bronze/silver/gold datasets (e.g., `bronze_pet_ct_small`, `silver_pet_ct_extended`, `gold_pet_ct_comprehensive`) live under `lib/domain/fake_data/data/eval/README.md`.

## Steps
1. Build/run the CLI using a named dataset
   ```bash
   cd code
   cargo run -p dfps_cli --bin eval_mapping -- \
     --dataset pet_ct_small \
     --dump-details
   ```
   (Use `--input <path>` instead if you want to point at a specific NDJSON file.)
2. Interpret output
   - Summary line: `{"kind":"eval_summary", ...}` includes `precision`, `recall`, counts.
   - Optional details: per-case `{"kind":"eval_result",...}` entries show whether each case was correct and include the `MappingResult` payload.
   - Calibration-style summaries land under `score_buckets`: each bucket represents a 0.1 score range and only includes MappingResults that produced an NCIt prediction, exposing how well the engine is calibrated per score band.
3. For custom gold sets, ensure each line matches:
   ```json
   {"system":"...","code":"...","display":"...","expected_ncit_id":"NCIT:Cxxxx"}
   ```
4. Optional: enforce thresholds in CI by supplying a JSON config:
   ```json
   {
     "min_precision": 0.95,
     "min_recall": 0.95,
     "min_f1": 0.95,
     "min_accuracy": 0.95,
     "min_auto_precision": 0.98,
     "min_coverage": 0.95
   }
   ```
   ```bash
  cargo run -p dfps_cli --bin eval_mapping -- \
    --dataset pet_ct_small \
    --thresholds lib/domain/fake_data/data/meta/eval_thresholds.json
  ```
  `min_accuracy` guards overall correctness (regardless of predictions) while `min_auto_precision` focuses on the AutoMapped band specifically.
   For determinism checks, provide a baseline fingerprint file:
   ```bash
   cargo run -p dfps_cli --bin eval_mapping -- \
     --dataset pet_ct_small \
     --deterministic target/eval/pet_ct_small.fingerprint
   ```
   First run writes the fingerprint; subsequent runs fail if the summary hash changes.
5. Persist machine-readable artifacts for dashboards/CI (plus optional Markdown report):
   ```bash
   cargo run -p dfps_cli --bin eval_mapping -- \
     --dataset gold_pet_ct_comprehensive \
     --out-dir target/eval \
     --report target/eval/report.md \
     --dump-details
   ```
   This writes `eval_summary.json` + `eval_results.ndjson` under `target/eval/gold_pet_ct_comprehensive/`.
6. Use `jq`/scripts (or the generated report) to gate CI metrics or share summaries.
7. Optional advanced stats: enable the `eval-advanced` feature to include bootstrap confidence intervals in the summary/report:
   ```bash
   cargo run -p dfps_cli --bin eval_mapping --features eval-advanced -- \
     --dataset pet_ct_small
   ```

## Dashboards & reporting
- `dfps_eval::report` now emits Markdown (for CLI artifacts) plus an HTML fragment consumed by the web frontend's HTMX panel.
- Run `dfps_cli eval_mapping --dataset <name> --report target/eval/report.md` to include the Markdown summary and baseline delta (if a `<dataset>.baseline.json` exists under `lib/domain/fake_data/data/eval/`).
- The web frontend automatically loads the `gold_pet_ct_small` baseline and exposes a dataset picker that swaps the HTMX fragment served from `/eval/report`.

## Requirements references
- `docs/system-design/clinical/ncit/requirements/ingestion-requirements.md` (MAP_ACCURACY) now points to `eval_mapping` as the verification method.
