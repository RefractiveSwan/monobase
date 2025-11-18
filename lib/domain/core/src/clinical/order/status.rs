use serde::{Deserialize, Serialize};

#[cfg(feature = "dummy")]
use fake::Dummy;

/// Status of a service request (order).
///
/// Modeled loosely on FHIR `request-status`.
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRequestStatus {
    Draft,
    Active,
    OnHold,
    Completed,
    Cancelled,
    Revoked,
    EnteredInError,
}
