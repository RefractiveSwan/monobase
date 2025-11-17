# Kanban - feature/mapping-benchmarking-platform (022)

> Epic: EVAL-022 – Mapping benchmarking platform  
> Branch: `feature/EVAL-022-mapping-benchmarking-platform`  
> Branch target version: `v0.1.0`  
> Status: **DOING**  
> Introduced in: `v0.1.0`  
> Last updated in: `v0.1.0`

**Theme:** Evaluation & benchmarking - large-scale gold sets, advanced metrics  
**Goal:** Evolve the existing mapping eval harness into a small benchmarking platform with larger gold datasets, richer metrics, and CI/Dashboard integration.

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / refactor / docs polish  
* **DONE** – Completed  

---

## TODO

---

## INPROGRESS
- _Empty_

---

## REVIEW

### EVAL-PLAT-01 – Eval crate & dataset handling

- [x] Introduce a dedicated eval crate `lib/domain/eval` (`dfps_eval`):
  - [x] Move or wrap the core types from epic 012:
    - `EvalCase { system, code, display, expected_ncit_id }`
    - `EvalResult`, `EvalSummary`.
  - [x] Add support for multiple datasets:
    - Named splits (e.g., `pet_ct_small`, `pet_ct_extended`, `mixed_modalities`).
    - Config-driven dataset root (`DFPS_EVAL_DATA_ROOT`).
- [x] Store gold-standard files under `lib/domain/fake_data/data/eval/*.ndjson`.

### EVAL-PLAT-02 – Advanced metrics

- [x] Extend `EvalSummary` to include:
  - [x] Precision/recall/F1 (overall).
  - [x] Metrics stratified by:
    - [x] Code system (CPT/SNOMED/LOINC/NCIt OBO).
    - [x] LicenseTier (licensed vs open).
    - MappingState (AutoMapped/NeedsReview/NoMatch) already present.
- [x] Advanced stats when the feature flag `eval-advanced` is set:
  - [x] Bootstrap confidence intervals for key metrics (`dfps_eval::bootstrap_metrics`).
  - [x] Calibration-style summaries (e.g., score buckets vs correctness).

### EVAL-PLAT-03 – CI & regression gates

- [x] Add a CLI entrypoint:
- `dfps_cli eval-mapping --dataset pet_ct_small --thresholds lib/domain/fake_data/data/meta/eval_thresholds.json`:
    - [x] Runs `run_eval` and writes a JSON summary.
    - [x] Exit with non-zero code if metrics drop below configured thresholds.
- [x] Wire CI job:
  - [x] On each merge to main, run `eval-mapping` against at least one gold set.
  - [x] Fail the pipeline if AutoMapped precision or overall accuracy regresses.
  
### EVAL-PLAT-04 – Dashboards & reporting

- [x] Emit machine-readable summaries (`eval_summary.json` + `eval_results.ndjson`) via `dfps_cli eval_mapping --out-dir <dir>`.
- [x] CLI artifacts: `dfps_cli eval_mapping --out-dir <dir>` writes `eval_summary.json` + `eval_results.ndjson` for CI/dashboards.
- [x] Simple Markdown report via `dfps_cli eval_mapping --report <path>` (temporary CLI-side generator until a richer HTMX/dfps_eval report lands).
- [x] Add a small HTMX/markdown report generator in `dfps_eval` and `dfps_web_frontend`:
  - [x] Renders tables of metrics and a short changelog comparing against a baseline.
- [x] Add endpoints in `dfps_api` to expose latest eval summaries to the web UI (`GET /api/eval/summary?dataset=...`).

### EVAL-PLAT-05 – Datasets & tests

- [x] Create nine distinct datasets (3× bronze/silver/gold):
  - Bronze: `bronze_pet_ct_small`, `bronze_pet_ct_unknowns`, `bronze_pet_ct_mixed`.
  - Silver: `silver_pet_ct_small`, `silver_pet_ct_extended`, `silver_pet_ct_obo`.
  - Gold: `gold_pet_ct_small`, `gold_pet_ct_extended`, `gold_pet_ct_comprehensive`.
  - Updated `lib/domain/fake_data/data/eval/README.md` with tier descriptions; datasets consume shared schema under `DFPS_EVAL_DATA_ROOT`.
- [x] Tests in `dfps_test_suite`:
  - [x] Verify that the eval harness correctly classifies matches/mismatches (`mapping_eval.rs` still exercises precision/recall + state counts).
  - [x] Add tiered dataset load test to ensure bronze/silver/gold splits stay readable.

### EVAL-PLAT-06 – Migrate/unwrap 012 harness into `dfps_eval`

* [x] Create new crate `lib/domain/eval` (`dfps_eval`).
  * [x] Move `EvalCase`, `EvalResult`, `EvalSummary`, `run_eval` out of `dfps_mapping::eval` into `dfps_eval`.
  * [x] Re-export from `dfps_mapping` temporarily to avoid churn (`pub use dfps_eval::*`), then remove once downstream crates are updated.
* [x] Update imports across `dfps_test_suite` to use `dfps_eval`.
* [x] Provide a small, streaming NDJSON reader in `dfps_eval::io` (avoids loading entire datasets into memory).
* [x] Deprecation note in `lib/domain/mapping/src/eval.rs` (with `#[deprecated]`) pointing to `dfps_eval`.

**Acceptance:** `dfps_test_suite::integration::mapping_eval` compiles against `dfps_eval` with no regressions; old module emits a deprecated warning only.

### EVAL-PLAT-07 – Dataset manifests, versioning & licensing

* [x] New directory: `lib/domain/fake_data/data/eval/` with datasets next to a manifest file `<dataset>.manifest.json`.
* [x] Manifest schema:
  * [x] `{ "name": "...", "version": "YYYYMMDD", "license": "…", "source": "…", "n_cases": N, "sha256": "<file hash>", "notes": "…" }`
* [x] Add a loader in `dfps_eval::datasets` that:
  * [x] Resolves data root via `DFPS_EVAL_DATA_ROOT` (falls back to `lib/domain/fake_data/data/eval`).
  * [x] Validates `sha256` on load; surfaces warnings on mismatches.
  * [x] Warns if `license` is missing/unknown.
* [x] Provide manifests for:
  * [x] `pet_ct_small.ndjson` (existing)
  * [x] `pet_ct_extended.ndjson` (new; includes OBO-backed NCIt IDs, unknown systems, tricky synonyms)

**Acceptance:** `dfps_cli eval-mapping --dataset pet_ct_small` reads via manifest and prints a warning if the checksum is stale.

### EVAL-PLAT-08 – Reproducibility & determinism guardrails

* [x] Ensure all scoring paths remain deterministic across platforms:
  * [x] Audit hash usage and iteration order in `dfps_mapping` (e.g., `HashSet` -> order-independent usage) and confirm determinism of `VectorRankerMock`.
  * [x] Add `--deterministic` flag to CLI that asserts stable results vs a prior `eval_results.json`.
* [x] Add a property test in `dfps_test_suite`:
  * [x] Given the same input NDJSON, `run_eval` output bytes are identical across two runs.

**Acceptance:** CI job fails if a second run (same commit) produces a different `eval_results.json`.

### EVAL-PLAT-09 – Top‑K, coverage & error analysis

* [x] Extend `EvalSummary` with:
  * [x] `top1_accuracy`, `top3_accuracy` (placeholder until engine exposes top‑k).
  * [x] `coverage` = predicted_cases / total_cases.
  * [x] Per-system confusion/coverage tables.
  * [x] Distribution of `MappingResult.reason` for `NoMatch` rows.
* [x] Add `--top-k N` to CLI to compute top‑k metrics (engine currently mirrors top‑1; warning emitted).
* [x] Optional (under `eval-advanced` feature):
  * [x] Bootstrap CIs for precision/recall/F1 using simple resampling.

**Acceptance:** `eval_results.json` contains `topk` and `coverage` fields; test asserts that `NoMatch` reasons include `missing_system_or_code` on the unknown-code fixture.

### EVAL-PLAT-10 – CLI thresholds & CI gate (first cut)

* [x] Crate: `lib/app/cli` (`dfps_cli`) with subcommand:
  * [x] `dfps_cli eval-mapping --input lib/domain/fake_data/data/eval/pet_ct_small.ndjson --thresholds lib/domain/fake_data/data/meta/eval_thresholds.json --out target/eval/pet_ct_small.json`
* [x] Thresholds schema (JSON):
  ```json
  {
    "min_precision": 0.95,
    "min_recall": 0.95,
    "min_top1": 0.95,
    "allow_no_match_reason": ["missing_system_or_code","unknown_code_system"]
  }
  ```
* [x] Exit non‑zero when metrics fall below thresholds.
* [x] GitHub Actions workflow `.github/workflows/eval.yml`:
  * [x] Runs CLI on PRs and merges to `main` over `pet_ct_small`.
  * [x] Uploads `eval_results.json` as an artifact.

**Acceptance:** CI fails if `AutoMapped` precision drops beneath threshold; artifact includes JSON summary.

### EVAL-PLAT-11 – API endpoints to expose eval results

* [x] In `dfps_api`:
  * [x] `GET /api/eval/datasets` → list manifests (name, version, n_cases).
  * [x] `POST /api/eval/run` with `{ "dataset": "pet_ct_small", "top_k": 3 }` → runs eval and returns `EvalSummary`.
  * [x] `GET /api/eval/latest` → last on‑box summary (cached).
* [x] Wire through `ApiState` a small, in‑memory cache (`Arc<Mutex<Option<(dataset, summary)>>>`).
* [x] Add unit tests mirroring `web_api.rs` style.

**Acceptance:** Integration test calls `/api/eval/run` and asserts JSON schema + key fields (precision, recall, state_counts) are present.

### EVAL-PLAT-12 – Minimal web UI for eval

* [x] In `dfps_web_frontend`:
  * [x] Add route `/eval` showing:
    * [x] Dataset picker (from `/api/eval/datasets`).
    * [x] Button “Run eval” → hits `/api/eval/run`.
    * [x] Cards for precision/recall/F1/top‑k/coverage.
    * [x] Table of top `NoMatch` reasons.
* [x] Reuse Tailwind components from the mapping workbench; render a small results fragment via HTMX.
* [ ] Snapshot test asserts presence of metric labels and values.

**Acceptance:** Visiting `/eval` renders the dataset list and displays metrics after a run; HTMX fragment updates without a full page reload.

### EVAL-PLAT-13 – Performance & scale

* [x] Add Criterion benchmarks under `lib/domain/eval/benches/`:
  * [x] `bench_eval_pet_ct_small`
  * [x] `bench_eval_pet_ct_extended`
* [x] Stream NDJSON in chunks to keep RSS < 256MB for 100k lines (document guideline).
* [x] Expose `--parallel` flag (opt‑in), chunking by lines and merging summaries.

**Acceptance:** Benchmarks run locally; RSS stays below the documented target on synthetic 100k rows (document how to generate).

### EVAL-PLAT-14 – Reporting artifacts

* [x] Add `dfps_eval::report` to emit:
  * [x] `eval_results.json` (machine-readable).
  * [x] `summary.md` (for humans; includes a simple table and deltas vs baseline).
* [x] Include a “baseline” file alongside each dataset (e.g., `pet_ct_small.baseline.json`).
* [x] CLI `--compare-to baseline.json` prints a short delta summary and sets exit code on regressions.

**Acceptance:** CI uploads both JSON and Markdown; a PR shows deltas vs baseline in the job log.

---

## DONE
- _Empty_

---

## Acceptance Criteria

- `dfps_eval` can run against multiple gold-standard datasets and emit rich metrics.
- A CLI/CI pipeline exists to guard mapping quality regressions.
- Metrics are available for inspection (JSON artifacts and/or basic dashboards).

## Out of Scope

- Massive-scale benchmarking infrastructure (distributed jobs, multi-GB datasets).
- Sophisticated statistical tooling beyond basic bootstrap CIs and summary tables.
