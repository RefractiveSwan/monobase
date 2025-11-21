# Class view: mapping engine & NCIt concepts

```mermaid
classDiagram
  class CodeElement {
    +id : string
    +system : string
    +code : string
    +display : string
  }

  class MappingCandidate {
    +target_system : string
    +target_code : string
    +cui : string
    +score : float
  }

  class MappingResult {
    +code_element_id : string
    +cui : string
    +ncit_id : string?
    +score : float
    +strategy : string
    +state : MappingState
    +reason : string?
    +license_tier : string?
    +source_kind : string?
  }

  class MappingEngine {
    +map(code : CodeElement) : MappingResult
  }

  class NCItConcept {
    +ncit_id : string
    +preferred_name : string
    +synonyms[] : string
  }

  class DimNCITConcept {
    +ncit_id : string
    +preferred_name : string
    +semantic_group : string
  }

  MappingEngine --> CodeElement : consumes
  MappingEngine --> MappingResult : produces
  MappingResult --> NCItConcept : resolved_to
  NCItConcept --> DimNCITConcept : materialized_as
```

Implementation notes:

- Types are implemented in `refractive_swan_core::mapping`.
- Mapping behavior lives in `refractive_swan_mapping::MappingEngine` with state threshold
  logic (MAP-07) and explainability helpers (MAP-11). Threshold defaults live in
  `MappingThresholds`, surfaced on every `MappingResult`.
- The end-to-end façade `refractive_swan_pipeline::bundle_to_mapped_sr` produces the
  `MappingResult`/`DimNCITConcept` pairs used by the warehouse layer.
- When `state == NoMatch`, `reason` captures whether the engine fell below
  thresholds or lacked required identifiers. Known systems now attach license metadata for downstream policy hooks via the `license_tier` / `source_kind` fields exposed on `MappingResult`.
