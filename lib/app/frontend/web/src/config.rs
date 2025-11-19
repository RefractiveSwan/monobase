use std::{env, fmt, num::ParseIntError, time::Duration};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub listen_addr: String,
    pub backend_base_url: String,
    pub client_timeout: Duration,
    pub docs_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let listen_addr =
            env::var("DFPS_FRONTEND_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8090".to_string());
        let backend_base_url =
            env::var("DFPS_API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
        let timeout = parse_timeout_seconds()?;

        Ok(Self {
            listen_addr,
            backend_base_url,
            client_timeout: Duration::from_secs(timeout),
            docs_url: env::var("DFPS_DOCS_URL").ok(),
        })
    }
}

fn parse_timeout_seconds() -> Result<u64, ConfigError> {
    match env::var("DFPS_API_CLIENT_TIMEOUT_SECS") {
        Ok(raw) => {
            let seconds = raw
                .parse::<u64>()
                .map_err(|err| ConfigError::InvalidTimeout(raw, err))?;
            Ok(seconds.max(1))
        }
        Err(env::VarError::NotPresent) => Ok(15),
        Err(env::VarError::NotUnicode(value)) => Err(ConfigError::InvalidUtf8(
            "DFPS_API_CLIENT_TIMEOUT_SECS",
            value,
        )),
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidTimeout(String, ParseIntError),
    InvalidUtf8(&'static str, std::ffi::OsString),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidTimeout(raw, _) => {
                write!(
                    f,
                    "invalid DFPS_API_CLIENT_TIMEOUT_SECS value '{raw}'; expected seconds"
                )
            }
            ConfigError::InvalidUtf8(key, _) => {
                write!(f, "environment variable {key} contains invalid UTF-8")
            }
        }
    }
}

impl std::error::Error for ConfigError {}
