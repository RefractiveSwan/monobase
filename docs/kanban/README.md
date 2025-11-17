# Kanban Directory Layout

This folder organizes epics by concern to keep navigation predictable. All Kanban files remain under `docs/kanban/`.

- `feature/mvp/000-meta` – base skeleton and shared MVP scaffolding.
- `feature/mvp/010-fhir-pipeline` – FHIR ingestion, validation, and warehouse epics.
- `feature/mvp/020-ontology-mapping` – terminology, NCIt mapping, vectors, benchmarking, and compliance epics.
- `feature/mvp/030-apps` – CLI, web, and desktop MVPs.
- `feature/mvp/040-infra-and-docs` – observability, docs/makefiles, analytics dashboards, docs hosting/search.
- `feature/manifold-clinical_ontology` – manifold research epics (unchanged).
- `backlog/feature/mvp/apps` – backlog app epics (mirrors the apps grouping for symmetry).

Supporting files stay at the top level (`ROADMAP.md`, `_epic_versions.yaml`, `AGENTS.override.md`).
