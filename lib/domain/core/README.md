# dfps_core – Domain core, staging, and mapping types

`dfps_core` keeps the pure domain model for DFPS: ServiceRequest/value objects, staging rows, mapping results, and minimal FHIR shapes. It stays free of IO and environment access so app/pipeline layers can inject policy and configuration. Modules are grouped into four super-domains to keep dependencies explicit: `primitives/`, `clinical/`, `interop/`, and `semantics/`.

## System-design links
- docs/system-design/base/directory-architecture.md
- docs/system-design/fhir/models/class-model.md
- docs/system-design/ncit/models/class-model.md
- docs/system-design/fhir/behavior/sequence-servicerequest.md
- docs/system-design/ncit/behavior/sequence-servicerequest.md
- docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-04--domain-core--staging

## Super-domain layout
- `primitives/value/` – typed identifiers (`PatientId`, `EncounterId`, `ServiceRequestId`) with no upward dependencies.
- `clinical/{patient,encounter,order}/` – aggregates and lifecycle enums that depend only on `primitives`.
- `interop/fhir/` – minimal FHIR R4 shapes needed for ingestion; `interop/staging/` – flattened landing rows (`StgServiceRequestFlat`, `StgSrCodeExploded`) used by ingestion/pipeline/datamart.
- `semantics/mapping/` – `CodeElement`, `MappingResult`, thresholds/source versions, and NCIt concept views. Cross-domain bridges (e.g., `From<StgSrCodeExploded> for CodeElement`) live under `semantics/mapping/bridge/`.
- `prelude.rs` – convenience re-exports so consumers can keep using `dfps_core::{patient, order, staging, mapping, value, fhir}` paths.
- `semantics/mapping/element.rs` exposes `CodeElement::id_for(sr_id, system, code, display)` so ingestion,
  mapping, and datamart generate the exact same `code_element_id` string (`<sr_id>::<system>::<code>`).
- `semantics/mapping/result.rs` provides the canonical `MappingResult::auto_mapped / needs_review / no_match`
  builders so downstream crates do not hand-roll reason/state handling.

## Helpers and invariants
- `CodeElement::id_for` (semantics/mapping) standardizes the `code_element_id` format across ingestion, mapping, and datamart.
- `MappingResult::auto_mapped/needs_review/no_match` constructors align state/reason handling for downstream surfaces.
- Staging structs are shared across `dfps_ingestion::transforms::sr_to_staging`, `dfps_pipeline::bundle_to_mapped_sr`, and the datamart loader; fields mirror the staging tables in the FHIR/NCIt ERDs.
- `MappingResult`, `MappingThresholds`, and `MappingSourceVersion` are the canonical mapping structs; other crates (mapping/pipeline/datamart/api/frontend) import these rather than re-declaring variants.
- Keep this crate dependency-light (`serde`, `fake`/`Dummy` under the `dummy` feature) and avoid any env or filesystem access.
- Unit tests live alongside modules (FQID iteration in `fhir`, mapping helpers, staging serde round-trips) to keep the canonical ServiceRequest journey covered deterministically.
