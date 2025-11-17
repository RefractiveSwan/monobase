# Kanban - feature/app/cli-mvp

### Columns
* **TODO** – Not started yet
* **INPROGRESS** – In progress
* **REVIEW** – Needs code review / refactor / docs polish
* **DONE** – Completed

---

## TODO

### APP-01 – Scaffold CLI app crate (`dfps_cli`)
- [ ] Create `code/lib/app/frontend/cli/dfps_cli` crate with `src/main.rs`.
- [ ] Add the crate to the root `[workspace].members` under the `app` bucket.
- [ ] Expose a `run()` function that `main()` delegates to so tests can exercise CLI logic without spawning a separate process.

### APP-02 – `map-bundles` subcommand (Bundle -> NCIt pipeline)
- [ ] Implement a `map-bundles` subcommand that:
  - [ ] Reads NDJSON FHIR `Bundle`s from stdin or an `--input` path.
  - [ ] Calls `dfps_pipeline::bundle_to_mapped_sr` for each bundle.
  - [ ] Writes NDJSON rows for `StgServiceRequestFlat`, `MappingResult`, and `DimNCITConcept`.
- [ ] Wire in `dfps_observability::PipelineMetrics` so each run emits a final metrics summary line.
- [ ] Support flags such as:
  - [ ] `--explain` – emit explanation rows alongside mapping results.
  - [ ] `--no-metrics` – suppress metrics output.
  - [ ] `--pretty` – pretty-print JSON for debugging.

### APP-03 – `generate-fhir-bundles` subcommand (fake data)
- [ ] Implement a `generate-fhir-bundles` subcommand that wraps `dfps_fake_data::raw_fhir` helpers.
- [ ] Support `--count` and `--seed` options mirroring the existing `generate_fhir_bundle` binary semantics.
- [ ] Ensure the output shape matches what `bundle_to_mapped_sr` expects so it can be piped directly into `map-bundles`.

### APP-04 – CLI ergonomics & logging (OBS-03 follow-up)
- [ ] Initialize logging via `env_logger` and respect `RUST_LOG` for `dfps_pipeline` and `dfps_mapping`.
- [ ] Make `--help` output describe:
  - [ ] The Bundle -> staging -> NCIt flow at a high level.
  - [ ] Required/optional flags and relevant environment variables.
- [ ] Add a short README section (or extend existing docs) describing common CLI invocations and sample pipelines.
- [ ] Close out `OBS-03` from `feature-base-skeleton` by ensuring CLIs are discoverable and documented.

### APP-05 – Docs & directory-architecture alignment
- [ ] Update `docs/system-design/base/directory-architecture.md` to:
  - [ ] Use the correct path (`base/directory-architecture.md`) in the header.
  - [ ] Explicitly list `code/lib/app/frontend/cli/dfps_cli` under the `app/` bucket with responsibilities.
- [ ] Fix Rust doc comments that still reference `docs/system-design/fhir/*` or `docs/system-design/ncit/*` so they point to `docs/system-design/clinical/fhir/*` and `docs/system-design/clinical/ncit/*`.
- [ ] Update `docs/system-design/clinical/fhir/index.md` Quickstart CLI snippets to use `dfps_cli` subcommands instead of ad-hoc binaries.

### APP-06 – Tests & CI for CLI surfaces
- [ ] Add integration tests in `dfps_test_suite` that exercise the CLI `run()` entrypoint with:
  - [ ] The baseline FHIR bundle fixture -> assert NDJSON counts and expected NCIt IDs.
  - [ ] An “unknown code” bundle -> assert `NoMatch` mapping state and corresponding metrics increments.
- [ ] Optionally add a small CLI smoke test in CI (e.g., run `dfps_cli generate-fhir-bundles | dfps_cli map-bundles` on 1–2 bundles and assert exit code 0).

---

## INPROGRESS
- _Empty_

---

## REVIEW
- _Empty_

---

## DONE
- _Empty_