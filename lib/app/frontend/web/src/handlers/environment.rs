use actix_web::{HttpResponse, Result, web};

use crate::{
    handlers::home,
    state::AppState,
    views,
    views::models::{DiagnosticsView, EnvVarView, EnvironmentView, PageContext},
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/environment").route(web::get().to(environment_page)))
        .service(web::resource("/environment/diagnostics").route(web::get().to(diagnostics)));
}

pub async fn environment_page(state: web::Data<AppState>) -> Result<HttpResponse> {
    let chrome = home::view_chrome(&state);
    let diag = load_diagnostics(&state).await;
    let env = EnvironmentView {
        frontend_listen_addr: state.config.listen_addr.clone(),
        backend_base_url: state.config.backend_base_url.clone(),
        docs_url: state.config.docs_url.clone(),
        github_url: state.config.github_url.clone(),
        feature_flags: vec![
            EnvVarView {
                name: "refractive_swan_VECTOR_ENABLED".into(),
                value: std::env::var("refractive_swan_VECTOR_ENABLED").ok(),
            },
            EnvVarView {
                name: "refractive_swan_VECTOR_BACKEND".into(),
                value: std::env::var("refractive_swan_VECTOR_BACKEND").ok(),
            },
            EnvVarView {
                name: "refractive_swan_VECTOR_NAMESPACE".into(),
                value: std::env::var("refractive_swan_VECTOR_NAMESPACE").ok(),
            },
            EnvVarView {
                name: "refractive_swan_MESH_NODE_ID".into(),
                value: std::env::var("refractive_swan_MESH_NODE_ID").ok(),
            },
        ],
        diagnostics: diag,
    };
    let ctx = PageContext {
        environment: Some(env),
        chrome,
        ..PageContext::default()
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_environment_page(&ctx)))
}

pub async fn diagnostics(state: web::Data<AppState>) -> Result<HttpResponse> {
    let diag = load_diagnostics(&state).await;
    Ok(HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(serde_json::to_string_pretty(&diag).unwrap_or_else(|_| "{}".into())))
}

async fn load_diagnostics(state: &AppState) -> DiagnosticsView {
    let health = state.client.health().await.ok();
    let metrics = state.client.metrics_summary().await.ok();
    DiagnosticsView { health, metrics }
}
