use actix_web::{HttpResponse, Result, web};

use crate::{
    client::CohortFilters,
    state::AppState,
    view_model::{AnalyticsSummaryView, CohortView},
    views,
};

use super::home;

/// Registers analytics dashboard route.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/analytics").route(web::get().to(analytics_dashboard)));
}

/// Renders `/analytics` by calling warehouse-backed summary + cohort endpoints.
pub async fn analytics_dashboard(
    state: web::Data<AppState>,
    query: Option<web::Query<CohortFilters>>,
) -> Result<HttpResponse> {
    let mut filters = query.map(|q| q.into_inner()).unwrap_or_default();
    let mut ctx = home::build_base_context(&state.client, &state.dataset_store).await;
    match state.client.analytics_summary().await {
        Ok(summary) => ctx.analytics_summary = Some(AnalyticsSummaryView::from_response(&summary)),
        Err(err) => {
            ctx.analytics_error = Some(format!("Analytics summary failed: {}", err.user_message()));
        }
    }
    match state.client.analytics_cohort(&filters).await {
        Ok(cohort) => {
            ctx.cohort = Some(CohortView::from_response(&cohort));
            ctx.cohort_filters = filters.clone();
        }
        Err(err) => {
            ctx.cohort_error = Some(format!("Cohort query failed: {}", err.user_message()));
        }
    }
    if ctx.cohort.is_none() {
        filters.date_from.get_or_insert_with(String::new);
        ctx.cohort_filters = filters;
    }
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_page(&ctx)))
}

#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use dfps_observability::PipelineMetrics;
    use std::time::Duration;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::{
        client::{
            AnalyticsSummaryResponse, AnalyticsSummaryRow, BackendClient, CohortResponse,
            CohortRow, HealthResponse,
        },
        config::AppConfig,
        routes,
        state::AppState,
    };

    #[actix_web::test]
    async fn analytics_route_shows_summary_and_cohort() {
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
            .respond_with(ResponseTemplate::new(200).set_body_json(PipelineMetrics {
                analytics_requests: 2,
                cohort_queries: 1,
                avg_cohort_size: Some(1.0),
                ..PipelineMetrics::default()
            }))
            .mount(&backend)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/eval/datasets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
            .mount(&backend)
            .await;
        Mock::given(method("GET"))
            .and(path("/analytics/ncit-summary"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(AnalyticsSummaryResponse {
                    rows: vec![AnalyticsSummaryRow {
                        ncit_id: "C1234".into(),
                        preferred_name: Some("FDG Uptake".into()),
                        mapping_state: Some("auto_mapped".into()),
                        time_bucket: Some("2024-05-01".into()),
                        count: 3,
                    }],
                }),
            )
            .mount(&backend)
            .await;
        Mock::given(method("GET"))
            .and(path("/analytics/cohort"))
            .respond_with(ResponseTemplate::new(200).set_body_json(CohortResponse {
                total: 1,
                rows: vec![CohortRow {
                    sr_id: "SR-1".into(),
                    patient_id: Some("P1".into()),
                    encounter_id: Some("ENC-1".into()),
                    ncit_id: Some("C1234".into()),
                    status: "active".into(),
                    intent: "order".into(),
                    description: "FDG".into(),
                    ordered_at: Some("2024-05-01T12:00:00Z".into()),
                    mapping_state: Some("auto_mapped".into()),
                }],
            }))
            .mount(&backend)
            .await;

        let config = AppConfig {
            listen_addr: "127.0.0.1:0".into(),
            backend_base_url: backend.uri(),
            client_timeout: Duration::from_secs(5),
            docs_url: None,
        };
        let client = BackendClient::from_config(&config).expect("client");
        let dataset_store = dfps_eval::FileDatasetStore::default();
        let state = web::Data::new(AppState::new(config.clone(), client, dataset_store));
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .configure(routes::configure),
        )
        .await;

        let request = test::TestRequest::get().uri("/analytics").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let body = test::read_body(response).await;
        let html = String::from_utf8(body.to_vec()).expect("html");
        assert!(html.contains("Analytics overview"));
        assert!(html.contains("FDG Uptake"));
        assert!(html.contains("C1234"));
        assert!(html.contains("Matched 1 orders"));
    }
}
