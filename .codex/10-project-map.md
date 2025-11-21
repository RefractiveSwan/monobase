# Project Map (what lives where)

_All paths relative to `code/`._

## Kanban
- `docs/kanban/**/*.md` (mirrored into `docs/book/src/kanban/**` via `cargo make docs-sync`)

## Reference terminology
- `docs/reference-terminology/semantic-relationships.yaml`  
  Keep semantic relationships (synonym, hypernym, mapping states) consistent with code & docs.

## System-design docs

### Base / workspace layout
- `docs/system-design/base/directory-architecture.md` � **Read this first** to pick crate homes.  
  Buckets: `app/`, `domain/`, `platform/`.

### FHIR system (selected)
- Architecture: `docs/system-design/fhir/architecture/system-architecture.md`
- Behavior: `docs/system-design/fhir/behavior/sequence-servicerequest.md`, `state-servicerequest.md`
- Models: `docs/system-design/fhir/models/class-model.md`, `data-model-er.md`
- Overview: `docs/system-design/fhir/overview.md` / `index.md`

### NCIt mapping system (selected)
- Architecture: `docs/system-design/ncit/architecture/system-architecture.md`, `architecture.md`
- Behavior: `docs/system-design/ncit/behavior/sequence-servicerequest.md`, `state-servicerequest.md`
- Models: `docs/system-design/ncit/models/class-model.md`, `data-model-er.md`
- Overview: `docs/system-design/ncit/index.md`

## Workspace tooling
- `Makefile.toml` + `data/makefiles/` � standardized cargo-make tasks
- `docs/book/` � mdBook sources and built HTML
- `docs/runbook/` � runbooks (synced into the mdBook)
- `data/environment/` � `.env.*.example` templates (loader: `refractive_swan_configuration`)

## Binary entrypoint
- `src/main.rs` � if used; may compose `lib/pipeline` etc.
