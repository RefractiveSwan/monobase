# Crate: lib/domain/core - `refractive_swan_core`

**Path:** `code/lib/domain/core`  
**Purpose:** canonical domain/FHIR/staging/mapping/value types with `serde` support.  
**Feature flags:** `dummy` (enables `fake` derives for many types).

## Responsibilities
- Define the **domain model** (entities, value objects) and **FHIR-minimal** structs.
- Provide **staging rows** used by ingestion and mapping.
- Provide **mapping domain types** (`CodeElement`, `MappingResult`, etc.).
- Keep all public types serializable + testable (JSON round‑trip, doc tests).

## Modules & key types
- `value/` - `PatientId`, `EncounterId`, `ServiceRequestId` newtypes.
- `patient/` - `Patient` aggregate (minimal, expandable).
- `encounter/` - `Encounter` entity linking patient to context.
- `order/` - `ServiceRequest` aggregate + `ServiceRequestStatus/Intent` enums.
- `fhir/` - minimal FHIR R4/R5 structs (`Bundle`, `ServiceRequest`, `Reference`, ...) + `Bundle::iter_servicerequests()`.
- `staging/` - `StgServiceRequestFlat`, `StgSrCodeExploded` for landing tables.
- `mapping/` - `CodeElement`, `MappingCandidate`, `MappingResult`, `MappingState`, `MappingThresholds`, `MappingSourceVersion`, `NCItConcept`, `DimNCITConcept`.

## Cross‑links
- FHIR flows & requirements: `docs/system-design/fhir/**`
- NCIt flows & states: `docs/system-design/ncit/**`
- Terminology semantics: `docs/reference-terminology/semantic-relationships.yaml`

## Tests
- Keep unit tests co‑located (e.g., `fhir::Bundle` iteration test, `mapping` ID stability).
- Prefer deterministic seeds when using `#[cfg(feature = "dummy")]` generators.
