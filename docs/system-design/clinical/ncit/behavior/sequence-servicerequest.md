# Sequence: code -> NCIt mapping -> warehouse

```mermaid
sequenceDiagram
  autonumber
  participant Stg as "stg_sr_code_exploded"
  participant Engine as "Mapping Engine"
  participant Vec as "Vector index"
  participant UMLS as "UMLS / NCIm"
  participant NCIt as "NCIt"
  participant DW as "Warehouse"

  Stg->>Engine: Batch (system, code, display)
  Engine->>Vec: Encode & search
  Vec-->>Engine: Top-k candidates

  Engine->>UMLS: Resolve to CUI
  UMLS-->>Engine: Best CUI

  Engine->>NCIt: Lookup NCIt concept
  NCIt-->>Engine: NCIT:Cxxxx

  Engine->>DW: Upsert dim_ncit_concept + link fact_sr
```

## State transitions & modules

- `dfps_mapping::MappingEngine` executes the Engine lifeline and assigns
  `MappingState` (`AutoMapped`, `NeedsReview`, `NoMatch`) based on the thresholds
  documented in the architecture page.
- `dfps_pipeline::bundle_to_mapped_sr` provides the orchestration glue between
  staging (`dfps_ingestion`) and mapping, ensuring the sequence stays intact for
  each Bundle.
- Explainability helpers (MAP-11) expose the candidate list generated during the
  Vec/UMLS steps so reviewers can audit why a given state was produced.
- `lib/app/frontend/cli` supplies:
  - `map_bundles` - streams Bundles through ingestion + mapping, emitting staging/mapping rows plus metrics.
  - `map_codes --explain` - prints the final `MappingResult` and JSON explanations (top-N candidates per code).
- Observability metrics (`PipelineMetrics`) ensure every run reports AutoMapped / NeedsReview / NoMatch counts.
