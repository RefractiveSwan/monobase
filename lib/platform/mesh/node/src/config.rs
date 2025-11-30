use refractive_swan_cache_store::{CacheBackend, CacheConfig, CacheConfigError};
use refractive_swan_compliance::{ComplianceConfig, ComplianceError, Policy};
use refractive_swan_configuration::{EnvLoadError, EnvValueError, string_var, u64_var};
use refractive_swan_datamart::sql::{WarehouseConfig, WarehouseConfigError};
use refractive_swan_eval::{
    FileDatasetStore,
    config::{EvalConfigError, EvalDatasetConfig},
};
use refractive_swan_mesh_dto::MeshNodeId;
use refractive_swan_vector_store::{VectorStoreConfig, VectorStoreConfigError, config_from_env};
use thiserror::Error;

const DEFAULT_MAX_DATASET_SIZE: u64 = 50_000;

/// Configuration inputs required to build a [`NodeDataPlane`].
#[derive(Debug, Clone)]
pub struct NodePlaneConfig {
    pub(crate) node_id: MeshNodeId,
    pub(crate) policy: Policy,
    pub(crate) dataset_store: FileDatasetStore,
    pub(crate) datamart: Option<WarehouseConfig>,
    pub(crate) vector: Option<VectorStoreConfig>,
    pub(crate) cache: Option<CacheConfig>,
    pub(crate) max_dataset_size: u64,
    pub(crate) tags: Vec<String>,
}

impl NodePlaneConfig {
    /// Load configuration from the provided env namespace.
    pub fn from_env(namespace: &str) -> Result<Self, NodePlaneConfigError> {
        load_plane_env(namespace)?;

        let node_id = string_var("refractive_swan_MESH_NODE_ID")
            .map_err(NodePlaneConfigError::EnvValue)?
            .and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(MeshNodeId::from_string(trimmed.to_string()))
                }
            })
            .unwrap_or_else(MeshNodeId::new_random);
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
        let cache_cfg = CacheConfig::from_env().map_err(NodePlaneConfigError::Cache)?;
        let cache = if cache_cfg.backend == CacheBackend::Disabled {
            None
        } else {
            Some(cache_cfg)
        };

        let max_dataset_size = u64_var("refractive_swan_MESH_MAX_DATASET_SIZE")
            .map_err(NodePlaneConfigError::EnvValue)?
            .unwrap_or(DEFAULT_MAX_DATASET_SIZE);
        let tags = string_var("refractive_swan_MESH_NODE_TAGS")
            .map_err(NodePlaneConfigError::EnvValue)?
            .and_then(parse_tags)
            .unwrap_or_else(|| vec!["dev".into()]);

        Ok(Self {
            node_id,
            policy,
            dataset_store,
            datamart,
            vector,
            cache,
            max_dataset_size,
            tags,
        })
    }

    /// Load configuration and override the node identifier (useful for tests or multi-node setups).
    pub fn from_env_with_node_id(
        namespace: &str,
        node_id: MeshNodeId,
    ) -> Result<Self, NodePlaneConfigError> {
        let mut config = Self::from_env(namespace)?;
        config.node_id = node_id;
        Ok(config)
    }

    pub fn node_id(&self) -> &MeshNodeId {
        &self.node_id
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

    pub fn cache_config(&self) -> Option<CacheConfig> {
        self.cache.clone()
    }

    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    pub fn max_dataset_size(&self) -> u64 {
        self.max_dataset_size
    }

    pub fn tags(&self) -> Vec<String> {
        self.tags.clone()
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
    #[error("environment value error: {0}")]
    EnvValue(#[from] EnvValueError),
    #[error("compliance configuration error: {0}")]
    Compliance(#[from] ComplianceError),
    #[error("evaluation dataset config error: {0}")]
    Eval(#[from] EvalConfigError),
    #[error("vector store configuration error: {0}")]
    Vector(#[from] VectorStoreConfigError),
    #[error("warehouse configuration error: {0}")]
    Datamart(#[from] WarehouseConfigError),
    #[error("cache configuration error: {0}")]
    Cache(#[from] CacheConfigError),
}

fn parse_tags(raw: String) -> Option<Vec<String>> {
    let tags: Vec<String> = raw
        .split(',')
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
        .map(|tag| tag.to_string())
        .collect();
    if tags.is_empty() { None } else { Some(tags) }
}
