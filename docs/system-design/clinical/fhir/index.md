# Basic FHIR System Design

## Contents

- [Overview](./overview.md)
- Architecture
  - [System architecture](./architecture/system-architecture.md)
- Models
  - [FHIR class model](./models/class-model.md)
  - [Ingestion ER model](./models/data-model-er.md)
- Behavior
  - [ServiceRequest sequence](./behavior/sequence-servicerequest.md)
  - [ServiceRequest state lifecycle](./behavior/state-servicerequest.md)
- Requirements
  - [Ingestion requirements](./requirements/ingestion-requirements.md)
- Experience
  - [PET/CT user journey](./experience/user-journey-pet-ct.md)
- Concepts
  - [Pipeline mindmap](./concepts/mindmap-pipeline.md)

## Ingestion MVP

The Rust modules `dfps_core::fhir` and `dfps_core::staging`, plus the
`dfps_ingestion` crate, implement the ServiceRequest ingestion flow described in
the [system architecture](./architecture/system-architecture.md),
[ingestion ER model](./models/data-model-er.md), and
[ServiceRequest sequence](./behavior/sequence-servicerequest.md) documents. This
MVP powers the synthetic bundle generators and end-to-end ingestion tests.

## Quickstart

### Code snippet

```rust
use dfps_ingestion::{
    bundle_to_staging_with_validation, ExternalValidationContext,
    validation::{ValidationMode, validate_bundle},
};
use dfps_pipeline::bundle_to_mapped_sr;
use serde_json::from_str;

let bundle: dfps_core::fhir::Bundle =
    from_str(include_str!("../../lib/domain/evaluation/fake_data/data/regression/fhir_bundle_sr.json"))?;

let validated = bundle_to_staging_with_validation(
    &bundle,
    ValidationMode::Lenient,
    ExternalValidationContext::default(),
)?;
assert!(!validated.report.has_errors());
let (flats, exploded) = validated.value;
let mapped = bundle_to_mapped_sr(&bundle)?;

assert_eq!(flats.len(), mapped.flats.len());
assert_eq!(exploded.len(), mapped.exploded_codes.len());
```

### Validation quickstart

```rust
use dfps_ingestion::validation::{validate_bundle, validate_sr, ValidationMode};

let bundle: dfps_core::fhir::Bundle =
    serde_json::from_str(include_str!("../../lib/domain/evaluation/fake_data/data/regression/fhir_bundle_sr.json"))?;

// Validate the whole bundle before ingestion.
let report = validate_bundle(&bundle);
assert!(!report.has_errors());

// Validate an individual ServiceRequest.
let sr = bundle
    .iter_servicerequests()
    .next()
    .expect("bundle contains ServiceRequest")
    .expect("service request decodes");
let issues = validate_sr(&sr);
assert!(issues.is_empty());

// Strict mode will block ingestion when issues are present.
let lenient = dfps_ingestion::bundle_to_staging_with_validation(
    &bundle,
    ValidationMode::Lenient,
    dfps_ingestion::ExternalValidationContext::default(),
)?;
assert!(!lenient.report.has_errors());
```

### CLI helpers

- Generate sample NDJSON Bundles:

  ```bash
  cargo run -p dfps_eval --bin generate_fhir_bundle -- --count 5 --seed 42 > bundles.ndjson
  ```

- Run the full ingestion + mapping pipeline:

  ```bash
  cargo run -p dfps_cli --bin map_bundles bundles.ndjson > pipeline_output.ndjson
  ```

- Show CLI help:

  ```bash
  cargo run -p dfps_cli --bin map_bundles -- --help
  cargo run -p dfps_eval --bin generate_fhir_bundle -- --help
  ```

### Observability & logging

- Enable structured logs + metrics summary:

  ```bash
  RUST_LOG=dfps_pipeline=info,dfps_mapping=warn \
    cargo run -p dfps_cli --bin map_bundles -- --log-level debug bundles.ndjson
  ```

  The CLI prints NDJSON outputs plus a final `metrics_summary` line with counts
  per mapping state.

- Inspect why a specific code mapped the way it did:

  ```bash
  RUST_LOG=dfps_mapping=warn \
    cargo run -p dfps_mapping --bin map_codes -- --explain staging_codes.ndjson
  ```

  Each `mapping_result` is followed by `{"kind":"explanation","value":{...}}`
  rows showing the top-N ranked candidates.
