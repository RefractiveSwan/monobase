//! Commonly used dfps_core types for ergonomic imports.
//!
//! See:
//! - docs/system-design/base/directory-architecture.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-04--domain-core--staging
//!
//! Card: REFR-04.

pub use crate::clinical::encounter::types::Encounter;
pub use crate::clinical::order::{
    intent::ServiceRequestIntent, status::ServiceRequestStatus, types::ServiceRequest,
};
pub use crate::clinical::patient::types::Patient;
pub use crate::interop::fhir::{Bundle, BundleEntry, CodeableConcept, Coding, Reference};
pub use crate::interop::staging::rows::{StgServiceRequestFlat, StgSrCodeExploded};
pub use crate::primitives::value::{EncounterId, PatientId, ServiceRequestId};
pub use crate::semantics::mapping::{
    CodeElement, DimNCITConcept, MappingResult, MappingSourceVersion, MappingState,
    MappingStrategy, MappingThresholds, NCItConcept,
};
