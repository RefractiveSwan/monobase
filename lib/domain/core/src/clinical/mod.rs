//! Clinical aggregates (patient/encounter/order) expressed purely in domain terms.
//!
//! Clinical modules depend only on primitives; no FHIR/staging/mapping coupling.
//!
//! See:
//! - docs/system-design/base/directory-architecture.md
//! - docs/system-design/fhir/models/class-model.md
//! - docs/system-design/fhir/behavior/sequence-servicerequest.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-04--domain-core--staging
//!
//! Card: REFR-04.

pub mod encounter;
pub mod order;
pub mod patient;
