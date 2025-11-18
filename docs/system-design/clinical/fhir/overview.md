# FHIR slice overview

```mermaid
graph LR
  Source["Source clinical systems"]
  FHIR_API["FHIR API & resources"]
  Landing["Raw ingestion & staging"]
  FHIR_Term["FHIR terminology layer"]
  NCIT_UMLS["Terminology mapping layer"]
  OBO["OBO / ontology layer"]
  DW["Warehouse & analytics"]

  Source --> FHIR_API
  FHIR_API --> Landing
  Landing --> FHIR_Term
  FHIR_Term --> NCIT_UMLS
  NCIT_UMLS --> OBO
  NCIT_UMLS --> DW
```

---

DFPS treats the "FHIR terminology layer" node above as a license-aware registry: `dfps_terminology::bridge::EnrichedCode` normalises system URLs, labels each code as licensed/open/OBO, and hands that metadata to `dfps_mapping`. CLI flows such as `map_codes` use `map_staging_codes_with_summary` to print counts by `CodeKind` so humans immediately see how many codes were missing identifiers or referenced licensed systems.

## Profiles & conformance

DFPS embeds thin StructureDefinition snapshots (`Patient`, `Encounter`, `ServiceRequest`) under `dfps_ingestion::profiles`. Ingestion callers can enable the `profile_validation` feature to enforce required elements and cardinalities (e.g., `ServiceRequest.status`, `ServiceRequest.subject`, `ServiceRequest.intent`, `ServiceRequest.id`) alongside hand-written checks and optional external `$validate` calls injected via an `ExternalValidator`. See `docs/runbook/fhir-profiles-quickstart.md` for updating embedded profiles.

**Related diagrams**

* [System architecture](./architecture/system-architecture.md)
* [FHIR class model](./models/class-model.md)
* [Ingestion ER model](./models/data-model-er.md)
* [ServiceRequest sequence](./behavior/sequence-servicerequest.md)
* [Ingestion requirements](./requirements/ingestion-requirements.md)
* [Terminology layer](./concepts/terminology-layer.md)

