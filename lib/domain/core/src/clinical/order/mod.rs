//! Order / ServiceRequest aggregate mirroring the flows in the FHIR design docs.
//!
//! This module maps directly to the ServiceRequest lifecycle shown in
//! `docs/system-design/fhir/behavior/state-servicerequest.md` and feeds the
//! ingestion/mapping pipelines documented in `docs/system-design/fhir/index.md`
//! and `docs/system-design/ncit/architecture/system-architecture.md`.
//!
//! Card: REFR-04.

pub mod intent;
pub mod ops;
pub mod status;
pub mod types;

#[cfg(test)]
mod tests;

pub use intent::ServiceRequestIntent;
pub use status::ServiceRequestStatus;
pub use types::ServiceRequest;
