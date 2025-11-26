use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Result, http::header, web};
use bytes::BytesMut;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    client::MapBundlesResponse,
    handlers::home,
    state::{AppState, LogEntry, MappingHistoryEntry},
    views,
    views::models::{
        AlertKind, AlertMessage, MappingResultsView, PageContext, summary_from_reports,
    },
    views::pages::workbench::render_bundle_textarea_fragment,
};
use std::time::Instant;
use uuid::Uuid;

const MAX_UPLOAD_BYTES: usize = 512 * 1024; // Mirrors refractive_swan_cli bundle cap.

/// Registers mapping-specific HTMX endpoints.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/map/paste").route(web::post().to(map_from_paste)))
        .service(web::resource("/map/upload").route(web::post().to(map_from_upload)))
        .service(web::resource("/map/history/{id}").route(web::get().to(history_entry)))
        .service(web::resource("/map/template").route(web::get().to(bundle_template)))
        .service(web::resource("/map/download/latest").route(web::get().to(download_latest)));
}

#[derive(Deserialize, Serialize)]
pub struct BundleForm {
    pub bundle_text: String,
}

/// Handles the "Paste JSON" HTMX form and keeps responses aligned with CLI expectations.
pub async fn map_from_paste(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<BundleForm>,
) -> Result<HttpResponse> {
    let hx = is_htmx(&req);
    let mut ctx = home::build_base_context(&state).await;
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

/// Handles HTMX upload flow, mirroring refractive_swan_cli ingestion size/error semantics.
pub async fn map_from_upload(
    state: web::Data<AppState>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let hx = is_htmx(&req);
    let mut ctx = home::build_base_context(&state).await;
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

/// Posts JSON payloads to refractive_swan_api /api/map-bundles and wires the HTMX fragment response.
async fn handle_mapping(
    payload: serde_json::Value,
    state: web::Data<AppState>,
    mut ctx: PageContext,
    hx: bool,
) -> Result<HttpResponse> {
    let started = Instant::now();
    let vector_mode = state.vector_mode();
    match state.client.map_bundles(payload, vector_mode).await {
        Ok(response) => {
            let duration = started.elapsed().as_millis();
            let records = response.clone();
            ctx.results = Some(MappingResultsView::from_response(&response));
            let mapped = ctx.results.as_ref().map(|res| res.rows.len()).unwrap_or(0);
            ctx.validation_summary = summary_from_reports(&response.validation_reports);
            if let Some(results) = &ctx.results {
                for row in &results.no_matches {
                    let reason = row.reason.as_deref().unwrap_or("unknown");
                    state.record_log(LogEntry::no_match(&row.sr_id, &row.code, reason));
                }
            }
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
            state.record_history(MappingHistoryEntry::success(duration, mapped, records));
            let history_entries = state.history_snapshot();
            ctx.mapping_history = home::hydrate_mapping_history(&history_entries);
            Ok(respond(ctx, hx))
        }
        Err(err) => {
            let duration = started.elapsed().as_millis();
            let msg = err.user_message();
            state.record_history(MappingHistoryEntry::failure(duration, msg.clone()));
            state.record_log(LogEntry::error(format!("Mapping error: {}", msg)));
            let history_entries = state.history_snapshot();
            ctx.mapping_history = home::hydrate_mapping_history(&history_entries);
            ctx.alert = Some(AlertMessage {
                kind: AlertKind::Error,
                text: format!("Backend error: {}", msg),
            });
            Ok(respond(ctx, hx))
        }
    }
}

fn respond(ctx: PageContext, hx: bool) -> HttpResponse {
    if hx {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(views::render_results_fragment(&ctx))
    } else {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(views::render_workbench_page(&ctx))
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

pub async fn history_entry(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid history id"))?;
    if let Some(entry) = state.history_entry(&id) {
        if let Some(response) = entry.response {
            let mut ctx = PageContext::default();
            ctx.results = Some(MappingResultsView::from_response(&response));
            if !entry.validation_reports.is_empty() {
                ctx.validation_summary = summary_from_reports(&entry.validation_reports);
            }
            return Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(views::render_results_fragment(&ctx)));
        }
    }
    Ok(HttpResponse::NotFound()
        .content_type("text/plain; charset=utf-8")
        .body("Mapping history not found"))
}

#[derive(Deserialize)]
pub struct TemplateQuery {
    pub name: Option<String>,
}

pub async fn bundle_template(query: web::Query<TemplateQuery>) -> Result<HttpResponse> {
    let name = query.name.as_deref().unwrap_or("blank");
    let template = crate::templates::load_template(name);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_bundle_textarea_fragment(template).into_string()))
}

pub async fn download_latest(state: web::Data<AppState>) -> Result<HttpResponse> {
    if let Some(entry) = state
        .history_snapshot()
        .into_iter()
        .find(|entry| entry.response.is_some())
    {
        if let Some(response) = entry.response {
            let bytes = build_ndjson_bytes(&response)
                .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;
            return Ok(HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, "application/x-ndjson"))
                .insert_header((
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"mapping_results.ndjson\"",
                ))
                .body(bytes));
        }
    }
    Ok(HttpResponse::NotFound()
        .content_type("text/plain; charset=utf-8")
        .body("No mapping results cached yet."))
}

fn build_ndjson_bytes(response: &MapBundlesResponse) -> Result<Vec<u8>, serde_json::Error> {
    let mut buffer = Vec::new();
    for flat in &response.flats {
        serde_json::to_writer(&mut buffer, &json!({"kind":"staging_flat","value":flat}))?;
        buffer.push(b'\n');
    }
    for code in &response.exploded_codes {
        serde_json::to_writer(
            &mut buffer,
            &json!({"kind":"stg_sr_code_exploded","value":code}),
        )?;
        buffer.push(b'\n');
    }
    for result in &response.mapping_results {
        serde_json::to_writer(
            &mut buffer,
            &json!({"kind":"mapping_result","value":result}),
        )?;
        buffer.push(b'\n');
    }
    for concept in &response.dim_concepts {
        serde_json::to_writer(&mut buffer, &json!({"kind":"dim_concept","value":concept}))?;
        buffer.push(b'\n');
    }
    for (bundle_index, report) in response.validation_reports.iter().enumerate() {
        for issue in &report.issues {
            serde_json::to_writer(
                &mut buffer,
                &json!({"kind":"validation_issue","bundle_index": bundle_index, "value": issue}),
            )?;
            buffer.push(b'\n');
        }
    }
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test, web};
    use refractive_swan_contracts::pipeline::{
        DimNCITConcept, MappingResult, MappingSourceVersion, MappingState, MappingStrategy,
        MappingThresholds, PipelineOutput, StgServiceRequestFlat, StgSrCodeExploded,
    };
    use refractive_swan_core::order::{ServiceRequestIntent, ServiceRequestStatus};
    use refractive_swan_observability::PipelineMetrics;
    use serde_json::json;
    use std::{sync::Arc, time::Duration};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::{
        client::{BackendClient, HealthResponse, MapBundlesResponse},
        config::AppConfig,
        routes,
        state::AppState,
    };

    fn sample_backend_response() -> MapBundlesResponse {
        let output = PipelineOutput {
            flats: vec![StgServiceRequestFlat {
                sr_id: "SR-1".into(),
                patient_id: "P1".into(),
                encounter_id: None,
                status: "active".into(),
                status_enum: ServiceRequestStatus::Active,
                intent: "order".into(),
                intent_enum: ServiceRequestIntent::Order,
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
            vector_usage: None,
        };
        MapBundlesResponse::new(output, Vec::new())
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
        assert!(html.contains("Mapping Results"));
        assert!(html.contains("C1234"));
        assert!(html.contains("AutoMapped"));
    }
}
