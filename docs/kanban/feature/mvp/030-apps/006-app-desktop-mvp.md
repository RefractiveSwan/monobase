# Kanban - feature/app/desktop-mvp

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / refactor / docs polish  
* **DONE** – Completed  

---

## TODO

### DESK-01 – Scaffold desktop shell crate (`dfps_desktop`)
- [ ] Create `code/lib/app/frontend/desktop/dfps_desktop` with `Cargo.toml` + `src/main.rs`.
- [ ] Add the crate to `[workspace].members` under the `app/` section.
- [ ] Expose a `run()` function from `lib` that `main()` calls so tests can exercise the desktop shell without spawning a process.

### DESK-02 – Wire desktop shell to pipeline
- [ ] Add dependency on `dfps_pipeline`, `dfps_fake_data`, and `dfps_observability`.
- [ ] Implement a simple “Load & map sample bundle” flow:
  - [ ] Option A: load the baseline bundle fixture from `dfps_test_suite::regression::baseline_fhir_bundle`.
  - [ ] Option B: open a local JSON/NDJSON file chosen by the user.
- [ ] Call `bundle_to_mapped_sr` and surface:
  - [ ] Count of flats / exploded codes.
  - [ ] Mapping state distribution (AutoMapped / NeedsReview / NoMatch).

### DESK-03 – Minimal UI frame
- [ ] Introduce a minimal desktop UI harness (e.g., simple multi-pane or tabbed layout; concrete GUI framework can remain abstracted behind a small “view” module).
- [ ] Show:
  - [ ] Left: list of loaded bundles / runs.
  - [ ] Right: summary table of NCIt IDs + mapping states.
- [ ] Add an “Export results” action that writes mappings + dim concepts to a JSON/NDJSON file.

### DESK-04 – Observability & debug tools
- [ ] Reuse `dfps_observability::PipelineMetrics` to show per-run metrics in the UI.
- [ ] Surface “NoMatch” codes in a dedicated panel for debugging triage.
- [ ] Ensure logging respects `RUST_LOG` and prints useful debug messages to a console/log pane.

### DESK-05 – Docs & architecture alignment
- [ ] Update `docs/system-design/base/directory-architecture.md` to list `dfps_desktop` explicitly under `lib/app/frontend/desktop`.
- [ ] Add a short “Desktop shell MVP” section (either here or a new doc) explaining how the desktop app wraps the existing FHIR -> NCIt pipeline.
- [ ] Ensure doc comments in `dfps_desktop` link to `docs/system-design/clinical/fhir/*` and `docs/system-design/clinical/ncit/*` where appropriate.

---

## INPROGRESS
- _Empty_

---

## REVIEW
- _Empty_

---

## DONE
- _Empty_
