use refractive_swan_configuration::{EnvLoadError, EnvValueError};
use refractive_swan_mesh_node::{NodePlaneConfig, NodePlaneConfigError};
use thiserror::Error;

use crate::server::ApiServerConfig;

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
}

fn load_api_env() -> Result<(), ApiConfigError> {
    match refractive_swan_configuration::load_env("app.web.api") {
        Ok(_) => Ok(()),
        Err(EnvLoadError::FileMissing { .. }) => Ok(()),
        Err(err) => Err(ApiConfigError::Env(err)),
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
