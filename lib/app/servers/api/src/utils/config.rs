use log::warn;
use refractive_swan_configuration::{EnvLoadError, EnvValueError};
use refractive_swan_mesh_node::{NodePlaneConfig, NodePlaneConfigError};
use thiserror::Error;

/// Combined configuration for the API server (network + data plane wiring).
#[derive(Debug)]
pub struct ApiConfig {
    pub server: ApiServerConfig,
    pub plane: NodePlaneConfig,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, ApiConfigError> {
        // Load the namespace first so any subsequent env lookups see overrides.
        load_api_env()?;
        let server = ApiServerConfig::from_env();
        let plane = NodePlaneConfig::from_env("app.web.api").map_err(ApiConfigError::Plane)?;
        Ok(Self { server, plane })
    }

    /// Load configuration from env or exit with a clear error message.
    pub fn from_env_or_exit() -> Self {
        match Self::from_env() {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("refractive_swan_api config error: {err}");
                std::process::exit(1);
            }
        }
    }
}

fn load_api_env() -> Result<(), ApiConfigError> {
    match refractive_swan_configuration::load_env("app.web.api") {
        Ok(_) => Ok(()),
        Err(EnvLoadError::FileMissing { .. }) => Ok(()),
        Err(err) => Err(ApiConfigError::Env(err)),
    }
}

/// Network-facing server configuration.
#[derive(Debug, Clone)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
}

impl ApiServerConfig {
    pub fn from_env() -> Self {
        let host = match refractive_swan_configuration::string_var("refractive_swan_API_HOST") {
            Ok(Some(value)) if !value.trim().is_empty() => value.trim().to_string(),
            Ok(_) => "127.0.0.1".into(),
            Err(err) => {
                warn!(target: "refractive_swan_api", "invalid refractive_swan_API_HOST: {err}; using default 127.0.0.1");
                "127.0.0.1".into()
            }
        };
        let port = match refractive_swan_configuration::port_var("refractive_swan_API_PORT") {
            Ok(Some(value)) => value,
            Ok(None) => 8080,
            Err(err) => {
                warn!(target: "refractive_swan_api", "invalid refractive_swan_API_PORT: {err}; using default 8080");
                8080
            }
        };
        Self { host, port }
    }
}

#[derive(Debug, Error)]
pub enum ApiConfigError {
    #[error("environment error: {0}")]
    Env(#[from] EnvLoadError),
    #[error("environment value error: {0}")]
    EnvValue(#[from] EnvValueError),
    #[error("node plane config error: {0}")]
    Plane(#[from] NodePlaneConfigError),
}
