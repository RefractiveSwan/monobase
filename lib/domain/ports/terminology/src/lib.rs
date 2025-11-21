//! Terminology lookup port shared across domain crates.
//! Implementations live in `dfps_terminology` and platform adapters.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminologyMode {
    MockOnly,
    HttpFallback,
    HttpOnly,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminologyClientConfig {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub timeout_secs: Option<u64>,
    pub mode: TerminologyMode,
}

impl Default for TerminologyClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            api_key: None,
            timeout_secs: None,
            mode: TerminologyMode::MockOnly,
        }
    }
}

pub trait TerminologyClient: Send + Sync {
    fn lookup_cui(&self, system: &str, code: &str) -> TerminologyResult<Option<CuiRecord>>;

    fn lookup_ncit(&self, cui_or_code: &str) -> TerminologyResult<Option<NcitRecord>>;

    fn search_by_text(&self, _text: &str) -> TerminologyResult<Vec<NcitRecord>> {
        Ok(Vec::new())
    }
}
