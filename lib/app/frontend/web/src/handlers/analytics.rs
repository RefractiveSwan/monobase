use actix_web::{HttpResponse, Result, http::header, web};
use serde::Deserialize;

use crate::{
    client::CohortFilters,
    state::AppState,
    views,
    views::models::{AnalyticsSummaryFilter, AnalyticsSummaryView, CohortView},
};

use super::home;

/// Registers analytics dashboard route.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/analytics").route(web::get().to(analytics_dashboard)))
        .service(web::resource("/analytics/cohort/export").route(web::get().to(cohort_export)))
        .service(
            web::resource("/analytics/summary/fragment")
                .route(web::get().to(analytics_summary_fragment)),
        );
}

/// Renders `/analytics` by calling warehouse-backed summary + cohort endpoints.
pub async fn analytics_dashboard(
    state: web::Data<AppState>,
    query: Option<web::Query<CohortFilters>>,
) -> Result<HttpResponse> {
    let mut filters = query.map(|q| q.into_inner()).unwrap_or_default();
    let mut ctx = home::build_base_context(&state).await;
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
    state.record_analytics(ctx.cohort.as_ref().map(|view| view.total));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_analytics_page(&ctx)))
}

#[derive(Deserialize)]
pub struct AnalyticsSummaryQuery {
    pub state_filter: Option<String>,
    pub range_days: Option<u32>,
}

pub async fn analytics_summary_fragment(
    state: web::Data<AppState>,
    query: web::Query<AnalyticsSummaryQuery>,
) -> Result<HttpResponse> {
    match state.client.analytics_summary().await {
        Ok(summary) => {
            let filter = AnalyticsSummaryFilter {
                state: query.state_filter.as_ref().and_then(|s| {
                    if s.trim().is_empty() {
                        None
                    } else {
                        Some(s.clone())
                    }
                }),
                range_days: query.range_days.filter(|v| *v > 0),
            };
            let view = AnalyticsSummaryView::from_response_with_filter(&summary, &filter);
            Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(crate::views::pages::analytics::render_summary_fragment(
                    &view,
                )))
        }
        Err(err) => Ok(HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Analytics summary failed: {}", err.user_message()))),
    }
}

pub async fn cohort_export(
    state: web::Data<AppState>,
    query: Option<web::Query<CohortFilters>>,
) -> Result<HttpResponse> {
    let filters = query.map(|q| q.into_inner()).unwrap_or_default();
    match state.client.analytics_cohort(&filters).await {
        Ok(cohort) => {
            let csv = build_cohort_csv(&cohort);
            Ok(HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, "text/csv"))
                .insert_header((
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"cohort_export.csv\"",
                ))
                .body(csv))
        }
        Err(err) => Ok(HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Cohort export failed: {}", err.user_message()))),
    }
}

fn build_cohort_csv(cohort: &crate::client::CohortResponse) -> String {
    let mut rows = String::from(
        "sr_id,patient_id,encounter_id,ncit_id,description,status,intent,ordered_at,mapping_state\n",
    );
    for row in &cohort.rows {
        rows.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            row.sr_id,
            row.patient_id.clone().unwrap_or_default(),
            row.encounter_id.clone().unwrap_or_default(),
            row.ncit_id.clone().unwrap_or_default(),
            row.description.replace('"', "'"),
            row.status,
            row.intent,
            row.ordered_at.clone().unwrap_or_default(),
            row.mapping_state.clone().unwrap_or_default(),
        ));
    }
    rows
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
            github_url: None,
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

        let request = test::TestRequest::get().uri("/analytics").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let body = test::read_body(response).await;
        let html = String::from_utf8(body.to_vec()).expect("html");
        assert!(html.contains("Analytics Summary"));
        assert!(html.contains("FDG Uptake"));
        assert!(html.contains("C1234"));
        assert!(html.contains("Found 1 matching records"));
    }

    #[actix_web::test]
    async fn analytics_route_handles_backend_errors() {
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
        Mock::given(method("GET"))
            .and(path("/analytics/ncit-summary"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&backend)
            .await;
        Mock::given(method("GET"))
            .and(path("/analytics/cohort"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&backend)
            .await;

        let config = AppConfig {
            listen_addr: "127.0.0.1:0".into(),
            backend_base_url: backend.uri(),
            client_timeout: Duration::from_secs(5),
            docs_url: None,
            github_url: None,
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

        let request = test::TestRequest::get().uri("/analytics").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let body = test::read_body(response).await;
        let html = String::from_utf8(body.to_vec()).expect("html");
        assert!(html.contains("Analytics Summary"));
        assert!(html.contains("Analytics summary failed"));
        assert!(html.contains("Cohort query failed"));
    }
}
