use serde::{Deserialize, Serialize};

#[cfg(feature = "dummy")]
use fake::Dummy;

/// Intent of the service request.
///
/// Modeled loosely on FHIR `request-intent`.
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRequestIntent {
    Proposal,
    Plan,
    #[default]
    Order,
    OriginalOrder,
    ReflexOrder,
    FillerOrder,
}
