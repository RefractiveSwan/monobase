# refractive_swan_test_suite – Shared fixtures & harness

Utilities, fixtures, and integration/e2e tests that keep the refractive_swan workspace honest. Apps and domain crates depend on this crate **only** for test code.

- `src/assertions.rs` – custom `assert_*` helpers surfaced as public functions so other crates can reuse them without copy/paste.
- `src/fixtures.rs` – deterministic fake-data builders (`service_request_*`, `eval_*`) backed by `refractive_swan_eval::fake_data`.
- `src/regression.rs` – helpers that load JSON/NDJSON fixtures from `lib/domain/meta/evaluation/data/**` via the shared `Registry`.
- `tests/` – three entry points wired from `cargo test -p refractive_swan_test_suite`:
  - `tests/unit` – property/unit coverage for shared helpers.
  - `tests/integration` – exercises pipeline/eval/datamart surfaces via public APIs (no internal modules).
  - `tests/e2e` – longer-running smoke flows, including the new `smoke_index` that touches ingestion → mapping → datamart → eval → vector layers.

## Environment helpers

- Call `refractive_swan_test_suite::init_environment()` at the start of each test (or use `ping()`) to load the `platform.test_suite` namespace via `refractive_swan_configuration`. The function now returns a `Result` so CI/tests can bubble meaningful errors.
- Use `refractive_swan_test_suite::ensure_eval_data_root()` to resolve the dataset root on disk (defaults to `lib/domain/meta/evaluation/data/eval`). Pass the returned path to CLIs/tests via `Command::env` or `scoped_env_var` when you need to set `refractive_swan_EVAL_DATA_ROOT`.
- Use `refractive_swan_test_suite::scoped_env_var(key, value)` instead of `unsafe { set_var }` inside tests. The guard restores the previous value on drop and keeps env mutations localized.

## Fixture ownership

- Regression and eval datasets live under `lib/domain/meta/evaluation/data/**`. When adding a new dataset:
  1. Check in the NDJSON under `data/eval/`.
  2. Create `<dataset>.manifest.json` with version/id/row-count plus SHA-256 checksum (see existing manifests).
  3. Update `lib/domain/meta/evaluation/data/README.md` with a short description and license note.
  4. Add a helper in `refractive_swan_test_suite::fixtures` if the dataset is used widely.
- Keep regression JSON fixtures focused (one scenario per file) so pipeline/eval tests remain deterministic.

## CI guidance

- `cargo test -p refractive_swan_test_suite` exercises the default mocks (in-memory sqlite/vector). For full backend coverage (e.g., pgvector), run `cargo test -p refractive_swan_test_suite --features backend-pgvector`.
- The crate intentionally stays test-only: production binaries should not depend on `refractive_swan_test_suite`.
- Integration/e2e tests only call public surfaces: `refractive_swan_pipeline::bundle_to_mapped_sr`, `refractive_swan_datamart::load_from_pipeline_output`, CLI entrypoints, `refractive_swan_eval::run_eval_with_mapper`, `refractive_swan_vector_store::MockVectorStore`, etc. If a new internal dependency creeps in, move it behind a public API before reusing it here.

## Testing conventions

- **File naming** – use `*_flow.rs` for e2e flows, `*_spec.rs`/`*_tests.rs` for integration/unit specs, and `*_helpers.rs` for pure helper tests. Each file starts with a `//!` doc referencing the epic or system-design doc it covers.
- **Modules** – `tests/unit`, `tests/integration/<area>`, `tests/e2e` mirror the workspace concerns (ingestion, mapping, datamart, api, vector, regression). Check `tests/integration/mod.rs` for the full map.
- **Running subsets** – examples:
  - `cargo test -p refractive_swan_test_suite --test unit_tests mapping_properties` (unit layer).
  - `cargo test -p refractive_swan_test_suite --test integration_tests mapping::mapping_eval` (mapping integrations).
  - `cargo test -p refractive_swan_test_suite --test e2e_tests smoke_index` (full-stack smoke).
- **Environment** – always call `refractive_swan_test_suite::init_environment()` at the start of tests and wrap env overrides in `ScopedEnvVar`.
