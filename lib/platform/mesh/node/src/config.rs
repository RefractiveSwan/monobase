use refractive_swan_compliance::{ComplianceConfig, ComplianceError, Policy};
use refractive_swan_configuration::EnvLoadError;
use refractive_swan_datamart::sql::{WarehouseConfig, WarehouseConfigError};
use refractive_swan_eval::{
    FileDatasetStore,
    config::{EvalConfigError, EvalDatasetConfig},
};
use refractive_swan_vector_store::{VectorStoreConfig, VectorStoreConfigError, config_from_env};
use thiserror::Error;

/// Configuration inputs required to build a [`NodeDataPlane`].
#[derive(Debug, Clone)]
pub struct NodePlaneConfig {
    pub(crate) policy: Policy,
    pub(crate) dataset_store: FileDatasetStore,
    pub(crate) datamart: Option<WarehouseConfig>,
    pub(crate) vector: Option<VectorStoreConfig>,
}

impl NodePlaneConfig {
    /// Load configuration from the provided env namespace.
    pub fn from_env(namespace: &str) -> Result<Self, NodePlaneConfigError> {
        load_plane_env(namespace)?;

        let compliance = ComplianceConfig::from_env().map_err(NodePlaneConfigError::Compliance)?;
        let policy = compliance
            .load_policy()
            .map_err(NodePlaneConfigError::Compliance)?;

        let dataset_store = EvalDatasetConfig::from_env()
            .map_err(NodePlaneConfigError::Eval)?
            .dataset_store();

        let vector = match config_from_env() {
            Ok(config) if config.enabled => Some(config),
            Ok(_) => None,
            Err(err) => return Err(NodePlaneConfigError::Vector(err)),
        };

        let datamart = match WarehouseConfig::from_env() {
            Ok(cfg) => Some(cfg),
            Err(WarehouseConfigError::MissingUrl) => None,
            Err(err) => return Err(NodePlaneConfigError::Datamart(err)),
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

    pub fn datamart_config(&self) -> Option<WarehouseConfig> {
        self.datamart.clone()
    }

    pub fn vector_config(&self) -> Option<VectorStoreConfig> {
        self.vector.clone()
    }

    pub fn policy(&self) -> &Policy {
        &self.policy
    }
}

fn load_plane_env(namespace: &str) -> Result<(), NodePlaneConfigError> {
    match refractive_swan_configuration::load_env(namespace) {
        Ok(_) => Ok(()),
        Err(EnvLoadError::FileMissing { .. }) => Ok(()),
        Err(err) => Err(NodePlaneConfigError::Env(err)),
    }
}

#[derive(Debug, Error)]
pub enum NodePlaneConfigError {
    #[error("environment error: {0}")]
    Env(#[from] EnvLoadError),
    #[error("compliance configuration error: {0}")]
    Compliance(#[from] ComplianceError),
    #[error("evaluation dataset config error: {0}")]
    Eval(#[from] EvalConfigError),
    #[error("vector store configuration error: {0}")]
    Vector(#[from] VectorStoreConfigError),
    #[error("warehouse configuration error: {0}")]
    Datamart(#[from] WarehouseConfigError),
}
