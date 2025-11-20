use serde::{Deserialize, Serialize};

#[cfg(feature = "dummy")]
use fake::Dummy;

fn ensure_non_empty(value: impl Into<String>, name: &str) -> String {
    let value = value.into();
    assert!(
        !value.trim().is_empty(),
        "{name} must not be empty or whitespace"
    );
    value
}

/// Strongly-typed identifier for a patient.
/// Wraps a string, but gives the type system something to grab.
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatientId(pub String);

/// Strongly-typed identifier for an encounter.
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EncounterId(pub String);

/// Strongly-typed identifier for a service request (order).
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceRequestId(pub String);

impl PatientId {
    #[track_caller]
    pub fn new(id: impl Into<String>) -> Self {
        Self(ensure_non_empty(id, "PatientId"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EncounterId {
    #[track_caller]
    pub fn new(id: impl Into<String>) -> Self {
        Self(ensure_non_empty(id, "EncounterId"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ServiceRequestId {
    #[track_caller]
    pub fn new(id: impl Into<String>) -> Self {
        Self(ensure_non_empty(id, "ServiceRequestId"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
