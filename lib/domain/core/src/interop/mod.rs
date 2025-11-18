//! Boundary shapes for interop with external/landing formats.
//!
//! FHIR representations and staging rows live here; no clinical/semantics deps.
//!
//! See:
//! - docs/system-design/fhir/architecture/system-architecture.md
//! - docs/system-design/fhir/models/class-model.md
//! - docs/system-design/fhir/behavior/sequence-servicerequest.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-04--domain-core--staging
//!
//! Card: REFR-04.

pub mod fhir;
pub mod staging;
