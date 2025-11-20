pub mod client;
pub mod config;
pub mod handlers;
pub mod routes;
pub mod state;
pub mod view_model;
pub mod views;

use actix_web::{App, HttpServer, web};
use client::BackendClient;
use config::AppConfig;
use state::AppState;
use std::{env, path::PathBuf, sync::Arc};

pub async fn run() -> std::io::Result<()> {
    if let Err(err) = dfps_configuration::load_env("app.web.frontend") {
        return Err(std::io::Error::other(format!(
            "dfps_web_frontend env error: {err}"
        )));
    }
    let config = AppConfig::from_env()
        .map_err(|err| std::io::Error::other(format!("frontend config error: {err}")))?;
    let client = BackendClient::from_config(&config)
        .map_err(|err| std::io::Error::other(format!("failed to create backend client: {err}")))?;
    let listen_addr = config.listen_addr.clone();
    let dataset_store = dataset_store_from_env();
    let state = AppState::new(config, client, dataset_store);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure)
    })
    .bind(&listen_addr)?
    .run()
    .await
}

fn dataset_store_from_env() -> Arc<dyn dfps_eval::DatasetStore + Send + Sync> {
    let root = env::var("DFPS_EVAL_DATA_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dfps_eval::default_data_root());
    Arc::new(dfps_eval::FileDatasetStore::new(root))
}
