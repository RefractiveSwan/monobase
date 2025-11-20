use super::types::TerminologyMode;

/// Configures how terminology lookups source remote adapters.
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
