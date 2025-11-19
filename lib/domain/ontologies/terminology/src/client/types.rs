use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminologyMode {
    MockOnly,
    HttpFallback,
    HttpOnly,
}

impl TerminologyMode {
    pub(crate) fn from_env_value(raw: &str) -> Option<Self> {
        match raw {
            "mock_only" => Some(Self::MockOnly),
            "http_fallback" => Some(Self::HttpFallback),
            "http_only" => Some(Self::HttpOnly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CuiRecord {
    pub cui: String,
    pub preferred_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct NcitRecord {
    pub ncit_id: String,
    pub preferred_name: String,
    pub synonyms: Vec<String>,
}

/// Errors that can occur during terminology lookups.
#[derive(Debug, Error, Clone)]
pub enum TerminologyClientError {
    #[error("terminology lookup forbidden")]
    Forbidden,
    #[error("terminology lookup timed out")]
    Timeout,
    #[error("terminology service unavailable")]
    Unavailable,
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("network error: {0}")]
    Network(String),
}

pub type TerminologyResult<T> = Result<T, TerminologyClientError>;

/// Abstraction for UMLS/NCIt external services.
pub trait TerminologyClient: Send + Sync {
    fn lookup_cui(&self, system: &str, code: &str) -> TerminologyResult<Option<CuiRecord>>;

    fn lookup_ncit(&self, cui_or_code: &str) -> TerminologyResult<Option<NcitRecord>>;

    fn search_by_text(&self, _text: &str) -> TerminologyResult<Vec<NcitRecord>> {
        Ok(Vec::new())
    }
}
