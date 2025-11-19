use crate::{client::BackendClient, config::AppConfig};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub client: BackendClient,
    pub dataset_store: dfps_eval::FileDatasetStore,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        client: BackendClient,
        dataset_store: dfps_eval::FileDatasetStore,
    ) -> Self {
        Self {
            config,
            client,
            dataset_store,
        }
    }
}
