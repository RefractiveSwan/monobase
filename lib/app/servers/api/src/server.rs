use std::net::{IpAddr, SocketAddr};

use axum::Router;
use log::{info, warn};
use tokio::net::TcpListener;
use tower::make::Shared;

pub use crate::utils::{
    ApiConfig, ApiError, ApiServerConfig, ApiState, ErrorResponse, ServerError,
};

pub mod controllers;
pub mod handlers;
mod routes;

pub use handlers::DatasetNodeRegistry;

/// Convenience helper that applies state immediately for consumers expecting a state-less Router.
pub fn router_with_state(state: ApiState) -> Router {
    routes::build_router().with_state(state)
}

/// Start the HTTP server using the provided configuration.
///
/// Builds the router, wires shared state, and blocks until Ctrl+C (or shutdown).
pub async fn run(config: ApiConfig) -> Result<(), ServerError> {
    let addr = socket_addr(&config.server)?;
    info!(target: "refractive_swan_api", "starting web backend on {addr}");
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|source| ServerError::Bind { addr, source })?;

    let router = router_with_state(ApiState::from_plane_config(config.plane));
    let make_service = Shared::new(router);

    axum::serve(listener, make_service)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServerError::Serve)?;

    info!(target: "refractive_swan_api", "server stopped");
    Ok(())
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => info!(target: "refractive_swan_api", "received shutdown signal"),
        Err(err) => warn!(target: "refractive_swan_api", "failed waiting for ctrl_c: {err}"),
    }
}

fn socket_addr(server: &ApiServerConfig) -> Result<SocketAddr, ServerError> {
    let ip: IpAddr = server
        .host
        .parse()
        .map_err(|source| ServerError::InvalidHost {
            host: server.host.clone(),
            source,
        })?;
    Ok(SocketAddr::new(ip, server.port))
}
