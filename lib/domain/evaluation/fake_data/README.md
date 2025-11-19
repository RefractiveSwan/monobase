# dfps_fake_data

Deterministic fake-data generators plus checked-in fixtures that mirror the DFPS
domain model (patients, encounters, ServiceRequests, mapping/eval corpora). See:

- `docs/system-design/base/directory-architecture.md`
- `docs/system-design/clinical/ncit/architecture.md`
- `docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md` (REFR-07)

## Modules

- `value`, `patient`, `encounter`, `order`, `scenarios`: thin helpers that create
  domain types from `dfps_core`, sharing ID/description logic with the rest of the
  workspace.
- `raw_fhir`: emits FHIR `Bundle`s + Patient/Encounter/ServiceRequest resources rooted
  in the same RNG helpers so CLI demos and tests stay reproducible.
- `fixtures`: loads evaluation + regression JSON/NDJSON under `data/` (see below).
- `rng`: exposes `with_global_rng`, `rng_from_seed`, and `SeedSequence` so CLIs/tests
  can share deterministic seeds instead of each module touching `rand::rng()`.

## Data layout

`lib/domain/evaluation/fake_data/data/`

- `eval/` – NDJSON corpora consumed by `dfps_eval::FileDatasetStore`.
- `meta/` – shared configs (e.g., eval thresholds).
- `regression/` – JSON fixtures referenced by the platform test suite.

Crates outside `domain/` should read `DFPS_FAKE_DATA_ROOT` or similar via their config
layer, then build a `fixtures::Registry::new_with_root(root)` (or a
`dfps_eval::FileDatasetStore`) instead of relying on this crate to read env vars.

## Determinism

- Default helpers use `rng::with_global_rng` so repeated runs emit the same IDs and
  orderings.
- Seed-specific helpers (`*_with_seed`) call `rng::rng_from_seed`, and consumers that
  need stable streams can leverage `rng::SeedSequence`.
