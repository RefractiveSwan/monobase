# Crate: lib/app/frontend/cli — `refractive_swan_cli`

**Purpose**  
Small CLIs for local ingestion + mapping workflows.

**Env & logging**
- Loads `app.cli` via `refractive_swan_configuration`.
- `env_logger` with `--log-level` on `map_bundles`.

**Bins**
- **`map_bundles`** — read Bundle(s) (object/array/NDJSON) from file/stdin → emit rows.
  - Output (NDJSON to stdout; each line wraps the record):
    - `{"kind":"validation_issue", ...}`
    - `{"kind":"staging_flat", ...}`
    - `{"kind":"staging_code", ...}`
    - `{"kind":"mapping_result", ...}`
    - `{"kind":"dim_concept", ...}` (deduped by `ncit_id`)
    - `{"kind":"metrics_summary", ...}` (final)
  - Logs pipeline summaries and `NoMatch` reasons via `refractive_swan_observability`.
  - Example:
    ```bash
    cd code
    cargo run -p refractive_swan_cli --bin map_bundles -- ./bundle.ndjson
    ```
- **`map_codes`** — map `StgSrCodeExploded` rows.
  - Flags: `--explain` (emit candidate explanations), `--explain-top N` (default 5).
  - Stdout: one `MappingResult` JSON per line (+ optional `{"kind":"explanation",...}`).
  - Stderr: summary (`total`, `by_code_kind`, `by_license_tier`).
  - Example:
    ```bash
    cd code
    cargo run -p refractive_swan_cli --bin map_codes -- --explain --explain-top 5 ./codes.ndjson
    ```
