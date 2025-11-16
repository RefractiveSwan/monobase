# Kanban — Learning and Teaching Docs
**Summary:** Track learning modules and guides (quickstarts, deep dives, contributor guides) for engineers, clinicians, and data scientists using FOSS-only tooling (Markdown, Mermaid, Rust notebooks where applicable).

## Kanban
### DOC-01 — Contributor quickstart
- [ ] Objectives/prereqs set; runnable examples included.
- [ ] Review checklist defined; cross-link to repo README and env quickstart.

### DOC-02 — FHIR basics module
- [ ] Explain FHIR concepts relevant to ServiceRequest; include small JSON examples.
- [ ] Link to `../../system-design/clinical/fhir/overview.md`.

### DOC-03 — NCIt/OBO basics module
- [ ] Overview of NCIt/OBO terms and mapping surface; glossary seeded.
- [ ] Link to `../../system-design/clinical/ncit/architecture/system-architecture.md`.

### DOC-04 — Vector search 101
- [ ] Describe vector stores and ANN; include simple CLI demo.
- [ ] Link to `../../system-design/clinical/ncit/concepts/vector-layer.md` and `../../runbook/vector-store-quickstart.md`.

### DOC-05 — Manifold geometry 101
- [ ] Introduce `R_M`, `D_M`, centroid/axis correlations; small synthetic example.
- [ ] Link to geometry Kanban `../research/math-proofs-and-geometry-docs.md`.

### DOC-06 — Deep dives (mapping pipeline)
- [ ] Walkthrough from staging codes to `MappingEngine`; include metrics/observability notes.
- [ ] Cross-link to FHIR and NCIt sequence docs.

### DOC-07 — Teaching modules review pipeline
- [ ] Define review rubric (accuracy, runnable examples, clarity).
- [ ] Track reviewer assignments and sign-off.

### DOC-08 — Tooling & formats
- [ ] Standardize Mermaid usage, Markdown templates, Rust notebook guidance (FOSS only).
- [ ] Provide sample snippet library for reuse.

## Acceptance Criteria
- Each module carries objectives, prerequisites, runnable examples, and a review checklist.
- Cross-links to system-design docs and vector-layer/runbook pages are present.
- Examples runnable with FOSS tooling; no closed SaaS dependencies.
