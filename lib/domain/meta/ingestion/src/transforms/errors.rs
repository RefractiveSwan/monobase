use serde_json::Error as SerdeError;

use crate::validation::types::ValidationIssue;

/// Errors surfaced while normalizing raw FHIR payloads.
#[derive(Debug)]
pub enum IngestionError {
    MissingField(&'static str),
    InvalidReference(&'static str),
    InvalidResourceType {
        expected: &'static str,
        found: String,
    },
    InvalidStatus(String),
    InvalidIntent(String),
    Decode(SerdeError),
    ValidationFailed(Vec<ValidationIssue>),
}

impl std::fmt::Display for IngestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "missing required field: {field}"),
            Self::InvalidReference(field) => write!(f, "invalid reference format for {field}"),
            Self::InvalidResourceType { expected, found } => {
                write!(f, "invalid resourceType '{found}', expected '{expected}'")
            }
            Self::InvalidStatus(value) => write!(f, "invalid status value '{value}'"),
            Self::InvalidIntent(value) => write!(f, "invalid intent value '{value}'"),
            Self::Decode(err) => write!(f, "failed to decode resource: {err}"),
            Self::ValidationFailed(issues) => {
                write!(f, "validation failed with {} issue(s)", issues.len())
            }
        }
    }
}

impl std::error::Error for IngestionError {}

impl From<SerdeError> for IngestionError {
    fn from(value: SerdeError) -> Self {
        Self::Decode(value)
    }
}
