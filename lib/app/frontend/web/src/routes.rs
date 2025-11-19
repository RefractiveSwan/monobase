use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Result, http::header, web};
use bytes::BytesMut;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::{
    client::{BackendClient, ClientError, CohortFilters},
    state::AppState,
    view_model::{
        AlertKind, AlertMessage, AnalyticsSummaryView, CohortView, DEFAULT_EVAL_DATASET,
        EvalContext, HealthOverview, MappingResultsView, PageContext,
    },
    views,
};
use dfps_eval::report;

const MAX_UPLOAD_BYTES: usize = 512 * 1024; // 512KiB bundles are enough for MVP use.

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(index)))
        .service(web::resource("/docs").route(web::get().to(docs_redirect)))
        .service(web::resource("/analytics").route(web::get().to(analytics_dashboard)))
        .service(web::resource("/map/paste").route(web::post().to(map_from_paste)))
        .service(web::resource("/map/upload").route(web::post().to(map_from_upload)))
        .service(web::resource("/eval/report").route(web::get().to(eval_report)))
        .service(web::resource("/eval").route(web::get().to(eval_page)))
        .service(web::resource("/eval/run").route(web::post().to(eval_run)));
}

async fn index(state: web::Data<AppState>) -> Result<HttpResponse> {
    let ctx = build_base_context(&state.client, &state.dataset_store).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_page(&ctx)))
}

async fn analytics_dashboard(
    state: web::Data<AppState>,
    query: Option<web::Query<CohortFilters>>,
) -> Result<HttpResponse> {
    let mut filters = query.map(|q| q.into_inner()).unwrap_or_default();
    let mut ctx = build_base_context(&state.client, &state.dataset_store).await;
    match state.client.analytics_summary().await {
        Ok(summary) => ctx.analytics_summary = Some(AnalyticsSummaryView::from_response(&summary)),
        Err(err) => {
            ctx.analytics_error = Some(format!(
                "Analytics summary failed: {}",
                summarize_client_error(err)
            ));
        }
    }
    match state.client.analytics_cohort(&filters).await {
        Ok(cohort) => {
            ctx.cohort = Some(CohortView::from_response(&cohort));
            ctx.cohort_filters = filters.clone();
        }
        Err(err) => {
            ctx.cohort_error = Some(format!(
                "Cohort query failed: {}",
                summarize_client_error(err)
            ));
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

#[derive(Deserialize, Serialize)]
struct BundleForm {
    bundle_text: String,
}

async fn map_from_paste(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<BundleForm>,
) -> Result<HttpResponse> {
    let hx = is_htmx(&req);
    let mut ctx = build_base_context(&state.client, &state.dataset_store).await;
    let trimmed = form.bundle_text.trim();
    if trimmed.is_empty() {
        ctx.alert = Some(AlertMessage {
            kind: AlertKind::Error,
            text: "Paste a Bundle payload before submitting.".to_string(),
        });
        return Ok(respond(ctx, hx));
    }

    match serde_json::from_str::<serde_json::Value>(trimmed) {
        Ok(value) => handle_mapping(value, state, ctx, hx).await,
        Err(err) => {
            ctx.alert = Some(AlertMessage {
                kind: AlertKind::Error,
                text: format!("Invalid JSON: {err}"),
            });
            Ok(respond(ctx, hx))
        }
    }
}

async fn map_from_upload(
    state: web::Data<AppState>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let hx = is_htmx(&req);
    let mut ctx = build_base_context(&state.client, &state.dataset_store).await;
    match read_bundle_file(&mut payload).await {
        Ok(Some(text)) => match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(value) => handle_mapping(value, state, ctx, hx).await,
            Err(err) => {
                ctx.alert = Some(AlertMessage {
                    kind: AlertKind::Error,
                    text: format!("Invalid JSON: {err}"),
                });
                Ok(respond(ctx, hx))
            }
        },
        Ok(None) => {
            ctx.alert = Some(AlertMessage {
                kind: AlertKind::Error,
                text: "Choose a JSON file before submitting.".to_string(),
            });
            Ok(respond(ctx, hx))
        }
        Err(msg) => {
            ctx.alert = Some(AlertMessage {
                kind: AlertKind::Error,
                text: msg,
            });
            Ok(respond(ctx, hx))
        }
    }
}

async fn handle_mapping(
    payload: serde_json::Value,
    state: web::Data<AppState>,
    mut ctx: PageContext,
    hx: bool,
) -> Result<HttpResponse> {
    match state.client.map_bundles(payload).await {
        Ok(response) => {
            ctx.results = Some(MappingResultsView::from_response(&response));
            let mapped = ctx.results.as_ref().map(|res| res.rows.len()).unwrap_or(0);
            ctx.alert = Some(if mapped == 0 {
                AlertMessage {
                    kind: AlertKind::Info,
                    text: "Backend responded but did not emit MappingResult rows. Confirm your bundle produced `stg_sr_code_exploded` entries."
                        .to_string(),
                }
            } else {
                AlertMessage {
                    kind: AlertKind::Info,
                    text: format!("Mapped {mapped} code(s)"),
                }
            });
            Ok(respond(ctx, hx))
        }
        Err(err) => {
            ctx.alert = Some(AlertMessage {
                kind: AlertKind::Error,
                text: format!("Backend error: {}", summarize_client_error(err)),
            });
            Ok(respond(ctx, hx))
        }
    }
}

async fn build_base_context(
    client: &BackendClient,
    store: &dfps_eval::FileDatasetStore,
) -> PageContext {
    let mut ctx = PageContext::default();
    ctx.datasets = client.eval_datasets().await.unwrap_or_default();
    if let Some(first) = ctx.datasets.first() {
        ctx.selected_eval_dataset = first.name.clone();
    }
    ctx.metrics = client.metrics_summary().await.ok();
    match client.health().await {
        Ok(resp) => {
            let status = resp.status;
            let ok = status == "ok";
            ctx.health = Some(HealthOverview {
                status: status.clone(),
                ok,
            });
            if !ok {
                ctx.health_error = Some(format!(
                    "Health endpoint returned status '{status}'. See backend logs for details."
                ));
            }
        }
        Err(err) => {
            ctx.health_error = Some(format!("Health endpoint unreachable: {err}"));
        }
    }
    ctx.metrics = client.metrics_summary().await.ok();
    let selected_dataset = ctx
        .datasets
        .first()
        .map(|m| m.name.as_str())
        .unwrap_or(DEFAULT_EVAL_DATASET);
    match build_eval_report_fragment(client, store, selected_dataset).await {
        Ok(html) => {
            ctx.eval_report_html = Some(html);
            ctx.selected_eval_dataset = selected_dataset.to_string();
        }
        Err(err) => {
            ctx.eval_panel_error = Some(err);
        }
    }
    ctx
}

async fn eval_page(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut ctx = PageContext::default();
    ctx.datasets = state.client.eval_datasets().await.unwrap_or_default();
    let selected = ctx
        .datasets
        .first()
        .map(|m| m.name.clone())
        .unwrap_or_else(|| DEFAULT_EVAL_DATASET.to_string());
    ctx.selected_eval_dataset = selected.clone();
    if let Ok(run) = state.client.eval_run(&selected, 1).await {
        ctx.eval = Some(EvalContext {
            dataset: selected.clone(),
            summary: run.summary,
        });
    }
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_eval_page(&ctx)))
}

#[derive(Deserialize)]
struct EvalReportQuery {
    dataset: Option<String>,
}

#[derive(Deserialize)]
struct EvalRunForm {
    dataset: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
}

fn default_top_k() -> usize {
    1
}

async fn eval_report(
    state: web::Data<AppState>,
    query: web::Query<EvalReportQuery>,
) -> Result<HttpResponse> {
    let dataset = query
        .dataset
        .as_deref()
        .unwrap_or(DEFAULT_EVAL_DATASET)
        .to_string();
    match build_eval_report_fragment(&state.client, &state.dataset_store, &dataset).await {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(err) => Ok(HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Eval report error: {err}"))),
    }
}

async fn eval_run(
    state: web::Data<AppState>,
    form: web::Form<EvalRunForm>,
) -> Result<HttpResponse> {
    let dataset = form.dataset.clone();
    match state.client.eval_run(&dataset, form.top_k).await {
        Ok(run) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(views::render_eval_fragment(&run))),
        Err(err) => Ok(HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Eval run error: {err}"))),
    }
}

async fn build_eval_report_fragment(
    client: &BackendClient,
    store: &dfps_eval::FileDatasetStore,
    dataset: &str,
) -> Result<String, String> {
    let summary = client
        .eval_summary(dataset)
        .await
        .map_err(|err| format!("Backend eval error: {err}"))?;
    let baseline = match report::load_baseline_snapshot_from(store.root(), dataset) {
        Ok(snapshot) => Some(snapshot),
        Err(err) => {
            eprintln!("warning: baseline load failed for {dataset}: {err}");
            None
        }
    };
    let html = report::render_html(
        &summary,
        baseline.as_ref().map(|snapshot| &snapshot.summary),
    );
    Ok(html)
}

fn respond(ctx: PageContext, hx: bool) -> HttpResponse {
    if hx {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(views::render_results_fragment(&ctx))
    } else {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(views::render_page(&ctx))
    }
}

fn is_htmx(req: &HttpRequest) -> bool {
    req.headers().contains_key("HX-Request")
}

async fn read_bundle_file(payload: &mut Multipart) -> Result<Option<String>, String> {
    while let Some(field) = payload
        .try_next()
        .await
        .map_err(|err| format!("Failed to read upload: {err}"))?
    {
        if field.name() != "bundle_file" {
            continue;
        }
        let mut field = field;
        let mut bytes = BytesMut::new();
        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(|err| format!("Failed to read upload chunk: {err}"))?
        {
            if bytes.len() + chunk.len() > MAX_UPLOAD_BYTES {
                return Err("Uploaded file is too large (max 512KiB)".to_string());
            }
            bytes.extend_from_slice(&chunk);
        }
        if bytes.is_empty() {
            return Err("Uploaded file is empty".to_string());
        }
        return String::from_utf8(bytes.to_vec())
            .map(Some)
            .map_err(|_| "File must be UTF-8 encoded JSON".to_string());
    }

    Ok(None)
}

fn summarize_client_error(err: ClientError) -> String {
    match err {
        ClientError::Backend { status, body } => format!("status {status} - {body}"),
        ClientError::Http(inner) => format!("HTTP error: {inner}"),
        ClientError::InvalidJson(inner) => format!("Invalid JSON: {inner}"),
        ClientError::Upload(msg) => msg,
        ClientError::Utf8(inner) => format!("UTF-8 error: {inner}"),
        ClientError::EmptyBundle => "No bundle payload supplied".to_string(),
    }
}

async fn docs_redirect(state: web::Data<AppState>) -> Result<HttpResponse> {
    if let Some(url) = &state.config.docs_url {
        Ok(HttpResponse::Found()
            .append_header((header::LOCATION, url.as_str()))
            .finish())
    } else {
        Ok(HttpResponse::NotFound().body("Docs URL not configured for this environment."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test, web};
    use dfps_core::{
        mapping::{
            DimNCITConcept, MappingResult, MappingSourceVersion, MappingState, MappingStrategy,
            MappingThresholds,
        },
        staging::{StgServiceRequestFlat, StgSrCodeExploded},
    };
    use dfps_observability::PipelineMetrics;
    use serde_json::json;
    use std::time::Duration;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::{
        client::{
            AnalyticsSummaryResponse, AnalyticsSummaryRow, CohortResponse, CohortRow,
            HealthResponse, MapBundlesResponse,
        },
        config::AppConfig,
    };

    fn sample_backend_response() -> MapBundlesResponse {
        MapBundlesResponse {
            flats: vec![StgServiceRequestFlat {
                sr_id: "SR-1".into(),
                patient_id: "P1".into(),
                encounter_id: None,
                status: "active".into(),
                intent: "order".into(),
                description: "PET-CT".into(),
                ordered_at: Some("2024-05-01T12:00:00Z".into()),
            }],
            exploded_codes: vec![StgSrCodeExploded {
                sr_id: "SR-1".into(),
                system: Some("http://loinc.org".into()),
                code: Some("24606-6".into()),
                display: Some("FDG uptake".into()),
            }],
            mapping_results: vec![MappingResult {
                code_element_id: "SR-1::http://loinc.org::24606-6".into(),
                cui: Some("C0001".into()),
                ncit_id: Some("C1234".into()),
                score: 0.99,
                strategy: MappingStrategy::Lexical,
                state: MappingState::AutoMapped,
                thresholds: MappingThresholds::default(),
                source_version: MappingSourceVersion::new("ncit-2024", "umls-2024"),
                reason: None,
                license_tier: None,
                source_kind: None,
            }],
            dim_concepts: vec![DimNCITConcept {
                ncit_id: "C1234".into(),
                preferred_name: "FDG Uptake".into(),
                semantic_group: "Test".into(),
            }],
        }
    }

    #[actix_web::test]
    async fn submitting_bundle_renders_mapping_rows() {
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
                bundle_count: 1,
                flats_count: 1,
                exploded_count: 1,
                mapping_count: 1,
                auto_mapped: 1,
                needs_review: 0,
                no_match: 0,
                license_blocked: 0,
                vector_queries: 0,
                vector_hits: 0,
                vector_fallbacks: 0,
                vector_latency_ms_p95: None,
                vector_capacity_geom_rm: None,
                vector_capacity_geom_dm: None,
                vector_capacity_geom_rm_sqrt_dm: None,
                vector_capacity_cap_alpha_sim: None,
                analytics_requests: 0,
                cohort_queries: 0,
                cohort_results_total: 0,
                avg_cohort_size: None,
                ..PipelineMetrics::default()
            }))
            .mount(&backend)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/eval/datasets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
            .mount(&backend)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/map-bundles"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_backend_response()))
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
        let app = test::init_service(App::new().app_data(state.clone()).configure(configure)).await;

        let payload = json!({ "resourceType": "Bundle", "type": "collection" }).to_string();
        let request = test::TestRequest::post()
            .uri("/map/paste")
            .set_form(&BundleForm {
                bundle_text: payload,
            })
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let body = test::read_body(response).await;
        let html = String::from_utf8(body.to_vec()).expect("html");
        assert!(html.contains("MappingResult rows"));
        assert!(html.contains("C1234"));
        assert!(html.contains("AutoMapped"));
    }

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
        let app = test::init_service(App::new().app_data(state.clone()).configure(configure)).await;

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
