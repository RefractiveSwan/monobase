//! Fake data generators for DFPS domain model.
//!
//! This crate exposes helpers to synthesize coherent patients, encounters,
//! service requests, and composite scenarios for tests and local tooling.

pub mod encounter;
pub mod fixtures;
pub mod order;
pub mod patient;
pub mod raw_fhir;
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
