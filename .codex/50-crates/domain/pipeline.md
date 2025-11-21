# Crate: lib/domain/pipeline — `refractive_swan_pipeline`

**Path:** `code/lib/domain/pipeline`  
**Depends on:** `refractive_swan_ingestion`, `refractive_swan_mapping`, `refractive_swan_core`, `refractive_swan_observability` (logging), `serde(_json)`, `thiserror`, `log`, `env_logger`.

## Responsibilities
- Provide a **single façade** from FHIR `Bundle` → staging → mapping → NCIt dims.
- Keep orchestration thin; **no business logic** beyond composition and error plumbing.

## Public API
- `bundle_to_mapped_sr(bundle: &Bundle) -> Result<PipelineOutput, PipelineError>`
  - Output: `{ flats, exploded_codes, mapping_results, dim_concepts }`
  - Error: `PipelineError::Ingestion(refractive_swan_ingestion::IngestionError)`

## Cross‑links
- FHIR quickstart & NCIt sequence: `docs/system-design/fhir/index.md`, `docs/system-design/ncit/behavior/sequence-servicerequest.md`

## Tests
- Add e2e tests as surfaces grow; today, lean on ingestion + mapping unit tests.
