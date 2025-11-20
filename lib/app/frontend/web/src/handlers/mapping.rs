use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Result, web};
use bytes::BytesMut;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::{
    handlers::home,
    state::AppState,
    view_model::{AlertKind, AlertMessage, MappingResultsView, PageContext},
    views,
};

const MAX_UPLOAD_BYTES: usize = 512 * 1024; // Mirrors dfps_cli bundle cap.

/// Registers mapping-specific HTMX endpoints.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/map/paste").route(web::post().to(map_from_paste)))
        .service(web::resource("/map/upload").route(web::post().to(map_from_upload)));
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
    let mut ctx = home::build_base_context(&state.client, state.dataset_store.as_ref()).await;
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

/// Handles HTMX upload flow, mirroring dfps_cli ingestion size/error semantics.
pub async fn map_from_upload(
    state: web::Data<AppState>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let hx = is_htmx(&req);
    let mut ctx = home::build_base_context(&state.client, state.dataset_store.as_ref()).await;
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

/// Posts JSON payloads to dfps_api /api/map-bundles and wires the HTMX fragment response.
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
                text: format!("Backend error: {}", err.user_message()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test, web};
    use dfps_contracts::pipeline::{
        DimNCITConcept, MappingResult, MappingSourceVersion, MappingState, MappingStrategy,
        MappingThresholds, StgServiceRequestFlat, StgSrCodeExploded,
    };
    use dfps_observability::PipelineMetrics;
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
            vector_usage: None,
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
        let dataset_store = Arc::new(dfps_eval::FileDatasetStore::default());
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
