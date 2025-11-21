//! Core domain model for refractive_swan (depth_forward_ontology_clinical_model).
//! Keeps staging/order/mapping value objects together with `serde` support.
//!
//! Super-domains: primitives, clinical, interop, semantics.
//!
//! See:
//! - docs/system-design/base/directory-architecture.md
//! - docs/system-design/fhir/models/class-model.md
//! - docs/system-design/ncit/models/class-model.md
//! - docs/system-design/fhir/behavior/sequence-servicerequest.md
//! - docs/system-design/ncit/behavior/sequence-servicerequest.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-04--domain-core--staging
//!
//! Card: REFR-04 (Domain core & staging). This crate stays pure domain—
//! no IO or environment access. Apps/pipeline inject policies and configs.

pub mod clinical;
pub mod interop;
pub mod prelude;
pub mod primitives;
pub mod semantics;

// Back-compat module aliases (so refractive_swan_core::<module> keeps working).
pub use clinical::encounter;
pub use clinical::order;
pub use clinical::patient;
pub use interop::fhir;
pub use interop::staging;
pub use primitives::value;
pub use semantics::mapping;
