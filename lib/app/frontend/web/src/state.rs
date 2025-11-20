use crate::{client::BackendClient, config::AppConfig};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub client: BackendClient,
    pub dataset_store: Arc<dyn dfps_eval::DatasetStore + Send + Sync>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        client: BackendClient,
        dataset_store: Arc<dyn dfps_eval::DatasetStore + Send + Sync>,
    ) -> Self {
        Self {
            config,
            client,
            dataset_store,
        }
    }
}
