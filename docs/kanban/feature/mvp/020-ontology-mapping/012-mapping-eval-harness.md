# Kanban - feature/mapping-eval-harness (012)

> Epic: EVAL-012 – Mapping eval harness  
> Branch: `feature/EVAL-012-mapping-eval-harness`  
> Branch target version: `Unreleased`  
> Status: **INPROGRESS**  
> Introduced in: `Unreleased`  
> Last updated in: `Unreleased`

**Goal:** Build a lightweight evaluation harness that measures NCIt mapping quality (precision/recall, state distributions) against gold-standard code -> NCIt labels.

### Columns
* **TODO** - Not started yet
* **INPROGRESS** - In progress
* **REVIEW** - Needs code review / refactor / docs polish
* **DONE** - Completed

---

## TODO
- _Empty_

---

## INPROGRESS

### EVAL-00 - General Tasks
- [ ] Confirm EvalSummary is stable enough to be consumed by CI or dashboards.
- [ ] Sanity check results on the current gold sample for regressions.

### EVAL-03 - Test harness integration
- [ ] Optionally add a property test ensuring:
  - [ ] label-mismatched golds never report as correct.

---

## REVIEW

### EVAL-01 - Gold standard format
- [x] Define a simple gold dataset schema (e.g., NDJSON or JSONL) under `lib/domain/fake_data/data/eval/`:
  - [x] `{"system": "...", "code": "...", "display": "...", "expected_ncit_id": "NCIT:Cxxxx"}`
- [x] Include a small PET/CT-focused sample:
  - [x] Codes for CPT, SNOMED, LOINC used in existing regression fixtures.

### EVAL-02 - Evaluation core API
- [x] New module or crate (e.g., `lib/platform/eval`):
  - [x] `EvalCase` struct mirroring the fixture shape.
  - [x] `EvalResult` / `EvalSummary` with:
    - [x] counts of correct / incorrect mappings,
    - [x] precision/recall,
    - [x] confusion by `MappingState` (`AutoMapped`, `NeedsReview`, `NoMatch`).
- [x] Provide a function:
  - [x] `run_eval(cases: &[EvalCase]) -> EvalSummary` that:
    - [x] runs each case through `map_staging_codes` (or equivalent),
    - [x] compares `expected_ncit_id` to the top `MappingResult`.
- [x] Implementation lives at `lib/domain/eval` (`dfps_eval::run_eval_with_mapper`), with a deprecated shim exposed through `dfps_mapping::eval`.

### EVAL-03 - Test harness integration
- [x] Add evaluation tests in `dfps_test_suite`:
  - [x] Construct a small suite of EvalCase rows from fixtures.
  - [x] Assert:
    - [x] AutoMapped precision meets a minimal bar for the tiny sample.
    - [x] NoMatch cases are correctly flagged when NCIt has no entry for the code.
- [x] Coverage implemented in `lib/platform/test_suite/tests/integration/mapping_eval.rs`.

### EVAL-04 - CLI wrapper
- [x] Introduce a small CLI binary, e.g.:
  - [x] Integrate a new `dfps_cli` subcommand `eval-mapping`.
- [x] CLI behavior:
  - [x] Accepts an NDJSON gold file path (`--input`).
  - [x] Prints summary metrics (precision, recall, counts by MappingState).
  - [x] Optional `--dump-details` flag to emit per-code results.
- [x] `lib/app/cli/src/bin/eval_mapping.rs` streams JSON summary rows that CI/scripts can consume.

### EVAL-05 - Docs & requirements link
- [x] Add `docs/runbook/mapping-eval-quickstart.md` describing:
  - [x] how to run the CLI over the gold file,
  - [x] how to interpret metrics.
- [x] Update `docs/system-design/clinical/ncit/requirements/ingestion-requirements.md` (e.g., requirement `MAP_ACCURACY`):
  - [x] reference the eval harness as the primary verification method.
- [x] Runbook also mirrored into `docs/book/src/runbook/mapping-eval-quickstart.md`.

---

## DONE
- _Empty_

---

## Acceptance Criteria
- A gold-standard fixture exists and is versioned.
- `run_eval` produces:
  - precision/recall numbers,
  - per-state confusion stats for `MappingState`.
- A CLI is available to run evals from the command line, and the runbook documents how.

## Out of Scope
- Large-scale benchmarking infrastructure or external datasets.
- Advanced statistical tests (bootstrap CIs, calibration plots, etc.) beyond basic metrics.
