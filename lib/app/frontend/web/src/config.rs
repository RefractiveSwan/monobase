use dfps_configuration::{self as config, EnvValueError};
use std::{fmt, time::Duration};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub listen_addr: String,
    pub backend_base_url: String,
    pub client_timeout: Duration,
    pub docs_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let listen_addr = config::string_var("DFPS_FRONTEND_LISTEN_ADDR")
            .map_err(ConfigError::Env)?
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "127.0.0.1:8090".into());
        let backend_base_url = config::string_var("DFPS_API_BASE_URL")
            .map_err(ConfigError::Env)?
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "http://127.0.0.1:8080".into());
        let timeout_secs = config::u64_var("DFPS_API_CLIENT_TIMEOUT_SECS")
            .map_err(ConfigError::Env)?
            .unwrap_or(15)
            .max(1);
        let docs_url = config::string_var("DFPS_DOCS_URL")
            .map_err(ConfigError::Env)?
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            listen_addr,
            backend_base_url,
            client_timeout: Duration::from_secs(timeout_secs),
            docs_url,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Env(EnvValueError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Env(err) => write!(f, "frontend env error: {err}"),
        }
    }
}

impl std::error::Error for ConfigError {}
