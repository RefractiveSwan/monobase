# refractive_swan_eval

Domain-facing evaluation harness for the NCIt mapping pipeline. The crate owns the
`EvalCase`/`EvalSummary` types, dataset manifests, baseline snapshots, and the
streaming runners consumed by the CLI, API, and frontend. See:

- `docs/system-design/clinical/ncit/architecture.md`
- `docs/runbook/030-mapping-and-terminology/mapping-eval-quickstart.md`
- `docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md` (REFR-07)

## Responsibilities

- Parse deterministic NDJSON datasets made of `EvalCase` rows and compute evaluation
  summaries (`EvalSummary`, `EvalResult`, calibration buckets, confusion matrices).
- Surface streaming helpers (`run_eval_streaming_with_mapper`) so apps can inject
  their own IO or mapping implementations without forcing this crate to own env/IO.
- Keep dataset manifests/baselines coupled to the data directory while surfacing a
  `FileDatasetStore` seam so platform layers can swap in alternative sources later
  (HTTP buckets, DB catalogues, etc.).
- Expose deterministic fake-data generators/fixtures under `fake_data::*` so tests,
  CLIs, and demos share the same RNG helpers and registry lookups.

## Dataset layout and config

- `DEFAULT_DATA_ROOT` points at the checked-in fixtures under
  `lib/domain/meta/eval/data/eval`. Apps should read `refractive_swan_EVAL_DATA_ROOT`
  via `refractive_swan_eval::config::EvalDatasetConfig::from_env()` to override the location;
  the helper resolves relative paths against the workspace and hands back a
  `FileDatasetStore` ready for use in CLIs/web/API layers.
- `FileDatasetStore` exposes helpers for manifests, NDJSON readers, and baseline
  snapshots. Convenience wrappers (`load_dataset*`, `list_manifests`) still exist for
  simple tests, but surfaces that need configurability should hold a store instance.
- Baseline Markdown/HTML reports are rendered via `refractive_swan_eval::report`, which now
  accepts explicit roots so platform surfaces can compare against their own snapshots.

## Runners and determinism

- `run_eval_with_mapper` accepts an in-memory slice of `EvalCase` plus an injected
  mapper closure (`Vec<StgSrCodeExploded> -> Vec<MappingResult>`).
- `run_eval_streaming_with_mapper` wraps a buffered reader, chunking NDJSON rows so
  large datasets stay memory-friendly and deterministic.
- The CLI (`refractive_swan_cli eval_mapping`), API (`/api/eval/*`), and frontend report panel
  all share the same entrypoints so regression fingerprints stay stable.

## Extending

- Add new dataset tiers by checking NDJSON + manifest (+ optional baseline) into
  `lib/domain/meta/eval/data/eval`.
- Keep manifests up to date when row counts or checksums change; the CLI warns when
  manifest metadata drifts from the on-disk NDJSON.
- When adding new summary fields, extend both `EvalSummary` and
  `refractive_swan_eval::report` so CLI/API artifacts remain in sync.
