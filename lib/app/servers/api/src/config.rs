use dfps_compliance::{ComplianceConfig, ComplianceError, Policy};
use dfps_configuration::{EnvLoadError, EnvValueError};
use dfps_datamart::WarehouseConfig;
use dfps_eval::{
    FileDatasetStore,
    config::{EvalConfigError as EvalDatasetError, EvalDatasetConfig},
};
use dfps_vector_store::{VectorStoreConfig, VectorStoreConfigError, config_from_env};
use thiserror::Error;

use crate::server::ApiServerConfig;

/// Combined configuration for the API server (network + data plane wiring).
#[derive(Debug)]
pub struct ApiConfig {
    pub server: ApiServerConfig,
    pub plane: DataPlaneConfig,
}

#[derive(Debug)]
pub struct DataPlaneConfig {
    pub policy: Policy,
    pub dataset_store: FileDatasetStore,
    pub datamart: Option<WarehouseConfig>,
    pub vector: Option<VectorStoreConfig>,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, ApiConfigError> {
        // Load the namespace first so any subsequent env lookups see overrides.
        load_api_env()?;
        let server = ApiServerConfig::from_env();
        let plane = DataPlaneConfig::from_env()?;
        Ok(Self { server, plane })
    }
}

impl DataPlaneConfig {
    pub fn from_env() -> Result<Self, ApiConfigError> {
        load_api_env()?;
        let compliance = ComplianceConfig::from_env().map_err(ApiConfigError::Compliance)?;
        let policy = compliance
            .load_policy()
            .map_err(ApiConfigError::Compliance)?;

        let dataset_store = EvalDatasetConfig::from_env()
            .map_err(ApiConfigError::Eval)?
            .dataset_store();

        let vector = match config_from_env() {
            Ok(config) if config.enabled => Some(config),
            Ok(_) => None,
            Err(err) => return Err(ApiConfigError::Vector(err)),
        };

        let datamart = match WarehouseConfig::from_env() {
            Ok(cfg) => Some(cfg),
            Err(dfps_datamart::sql::WarehouseConfigError::MissingUrl) => None,
            Err(err) => return Err(ApiConfigError::Datamart(err)),
        };

        Ok(Self {
            policy,
            dataset_store,
            datamart,
            vector,
        })
    }

    pub fn dataset_store(&self) -> FileDatasetStore {
        self.dataset_store.clone()
    }
}

fn load_api_env() -> Result<(), ApiConfigError> {
    match dfps_configuration::load_env("app.web.api") {
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
    #[error("compliance configuration error: {0}")]
    Compliance(#[from] ComplianceError),
    #[error("evaluation dataset config error: {0}")]
    Eval(#[from] EvalDatasetError),
    #[error("vector store configuration error: {0}")]
    Vector(#[from] VectorStoreConfigError),
    #[error("warehouse configuration error: {0}")]
    Datamart(#[from] dfps_datamart::sql::WarehouseConfigError),
}
