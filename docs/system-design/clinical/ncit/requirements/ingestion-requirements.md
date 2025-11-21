# Requirements: NCIm / NCIt mapping layer

```mermaid
requirementDiagram
  requirement MAP_ACCURACY {
    id: N1
    text: "High-confidence auto-maps SHALL achieve > 90% precision."
    risk: High
    verifymethod: Test (refractive_swan_cli eval_mapping)
  }

  requirement MAP_TRACE {
    id: N2
    text: "Every mapping MUST store provenance (source, version)."
    risk: Medium
    verifymethod: Analysis
  }

  element RerankPipeline {
    type: "Programming module"
  }

  element MappingDB {
    type: "DB schema"
  }

  element EvalHarness {
    type: "CLI/Test Harness"
  }

  RerankPipeline - satisfies -> MAP_ACCURACY
  EvalHarness - verifies -> MAP_ACCURACY
  MappingDB - verifies -> MAP_TRACE
```

## Verification notes
- MAP_ACCURACY → `refractive_swan_cli eval_mapping --dataset pet_ct_small --thresholds lib/domain/meta/evaluation/data/meta/eval_thresholds.json` (datasets under `lib/domain/meta/evaluation/data/eval/`).
- MAP_TRACE → Schema/tests ensure each `MappingResult` stores provenance (`source_version`, `strategy`, `reason`).
