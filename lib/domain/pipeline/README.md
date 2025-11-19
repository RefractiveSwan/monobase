# dfps_pipeline

Thin orchestration crate that composes ingestion and mapping into a single
Bundle→staging→NCIt flow. See:

- `docs/system-design/fhir/index.md`
- `docs/system-design/clinical/ncit/behavior/sequence-servicerequest.md`
- `docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md` (REFR-09)

## Responsibilities

- Provide a single entrypoint (`bundle_to_mapped_sr_with_vector_context`) that
  runs `dfps_ingestion::bundle_to_staging` followed by lexical/vector mapping.
- Keep `PipelineOutput` stable so app surfaces (CLI/API/web/datamart) can reuse
  the same DTOs without translation layers.
- Remain environment-free; callers inject any vector-store handles/config via
  `VectorPipelineContext` so transport-specific details stay in app/platform
  crates.

## Vector integration

- `VectorPipelineContext` bundles an `Arc<dyn VectorStore>`, the corresponding
  `VectorStoreConfig`, and tunables (e.g., `top_k`). Apps/bootstrap code build
  this context (reading env/config as needed) and pass `Some(&context)` when
  invoking the pipeline.
- When no context is provided, the pipeline automatically falls back to the
  lexical mapping path.

## Tests

- Pure lexical run (default): exercises ingestion→mapping and asserts
  `vector_usage` is `None`.
- Vector-enabled run (mock backend): asserts usage snapshots are recorded and
  the mapping path succeeds.
- Failure modes: invalid bundles bubble ingestion errors, and unhealthy vector
  contexts fall back to lexical behavior without panicking.
