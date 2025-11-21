use actix_web::{HttpResponse, Result, web};

use crate::views::layout::ViewChrome;
use crate::{state::AppState, views};

/// Registers developer-facing component preview routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/ui/components").route(web::get().to(components_preview)));
}

/// Renders the Storybook-like gallery so designers can verify tokens quickly.
pub async fn components_preview(state: web::Data<AppState>) -> Result<HttpResponse> {
    let chrome = ViewChrome::from(&state.config);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_components_preview_page(&chrome)))
}

#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use refractive_swan_observability::PipelineMetrics;
    use std::{sync::Arc, time::Duration};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::{
        client::{BackendClient, HealthResponse},
        config::AppConfig,
        routes,
        state::AppState,
    };

    #[actix_web::test]
    async fn components_preview_route_renders_gallery() {
        let backend = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(HealthResponse {
                status: "ok".into(),
            }))
            .mount(&backend)
            .await;
        Mock::given(method("GET"))
            .and(path("/metrics/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(PipelineMetrics::default()))
            .mount(&backend)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/eval/datasets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
            .mount(&backend)
            .await;

        let config = AppConfig {
            listen_addr: "127.0.0.1:0".into(),
            backend_base_url: backend.uri(),
            client_timeout: Duration::from_secs(5),
            docs_url: Some("http://localhost:3000/docs".into()),
            github_url: Some("https://github.com/example/repo".into()),
        };
        let client = BackendClient::from_config(&config).expect("client");
        let dataset_store = Arc::new(refractive_swan_eval::FileDatasetStore::default());
        let state = web::Data::new(AppState::new(config.clone(), client, dataset_store));
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .configure(routes::configure),
        )
        .await;

        let request = test::TestRequest::get().uri("/ui/components").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let body = test::read_body(response).await;
        let html = String::from_utf8(body.to_vec()).expect("html response");
        assert!(html.contains("UI Components Gallery"));
        assert!(html.contains("Mapping metrics"));
        assert!(html.contains("State chips"));
    }
}
