//! Fake data generators for DFPS domain model.
//!
//! See:
//! - docs/system-design/base/directory-architecture.md
//! - docs/system-design/clinical/ncit/architecture.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md (REFR-07)
//!
//! This crate exposes helpers to synthesize coherent patients, encounters,
//! service requests, and composite scenarios for tests and local tooling. RNG
//! helpers live under [`rng`] so CLIs and tests can share deterministic seeds.

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

/// Simple placeholder so downstream crates can confirm the crate is wired.
pub fn ping() -> &'static str {
    "fake-data-ready"
}
