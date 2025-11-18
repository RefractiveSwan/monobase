# Data Directory Layout

- `environment/` – env templates and examples.
- `clinical/` – clinical/semantic assets used by the stack:
  - `fhir/profiles/` – minimal FHIR StructureDefinition JSON snapshots.
  - `ontologies/` – OBO slices (e.g., NCIt, MONDO).
  - `warehouse/sql/migrations/` – warehouse DDL/migrations.
- `ops/` – operational tooling for builds and automation:
  - `makefiles/` – shared cargo/mk orchestration.
  - `scripts/` – helper scripts (codex sync, toolchain install).
  - `prompts/` – agent/LLM prompt references.
