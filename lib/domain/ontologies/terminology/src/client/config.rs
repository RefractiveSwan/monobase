use super::types::TerminologyMode;

/// Configures how terminology lookups source remote adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminologyClientConfig {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub timeout_secs: Option<u64>,
    pub mode: TerminologyMode,
}

impl TerminologyClientConfig {
    /// Build a config from environment variables.
    ///
    /// - `DFPS_TERMINOLOGY_BASE_URL`
    /// - `DFPS_TERMINOLOGY_API_KEY`
    /// - `DFPS_TERMINOLOGY_TIMEOUT_SECS`
    /// - `DFPS_TERMINOLOGY_MODE` (`mock_only` | `http_fallback` | `http_only`)
    ///
    /// Platform adapters should prefer loading env/config via `dfps_configuration`
    /// (or equivalent) and then constructing this struct explicitly; this helper
    /// exists for legacy CLIs/tests.
    pub fn from_env() -> Self {
        let base_url = std::env::var("DFPS_TERMINOLOGY_BASE_URL").ok();
        let api_key = std::env::var("DFPS_TERMINOLOGY_API_KEY").ok();
        let timeout_secs = std::env::var("DFPS_TERMINOLOGY_TIMEOUT_SECS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok());
        let mode = std::env::var("DFPS_TERMINOLOGY_MODE")
            .ok()
            .and_then(|raw| TerminologyMode::from_env_value(&raw))
            .unwrap_or(TerminologyMode::MockOnly);
        Self {
            base_url,
            api_key,
            timeout_secs,
            mode,
        }
    }
}
