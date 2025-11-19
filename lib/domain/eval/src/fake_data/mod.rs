//! Deterministic fake-data generators plus checked-in fixtures that mirror the
//! DFPS domain model (patients, encounters, ServiceRequests, eval corpora).
//!
//! Merged from the standalone `dfps_fake_data` crate so evaluation + fixtures
//! now live under one home. See:
//! - docs/system-design/base/directory-architecture.md
//! - docs/system-design/clinical/ncit/architecture.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md (REFR-07)

pub mod encounter;
pub mod fixtures;
pub mod order;
pub mod patient;
pub mod raw_fhir;
pub mod rng;
pub mod scenarios;
pub mod value;

pub use encounter::*;
pub use order::*;
pub use patient::*;
pub use raw_fhir::*;
pub use scenarios::*;
pub use value::*;

/// Simple placeholder so downstream crates can confirm the module is wired.
pub fn ping() -> &'static str {
    "fake-data-ready"
}
