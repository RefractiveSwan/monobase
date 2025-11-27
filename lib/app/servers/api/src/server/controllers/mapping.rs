use std::collections::HashSet;

use axum::{
    Json,
    body::Bytes,
    extract::{Query, State},
    response::IntoResponse,
};
use log::{info, warn};
use refractive_swan_compliance::assert_export_allowed;
use refractive_swan_contracts::{
    DimNCITConcept, MappingResult, MappingState, PipelineMetrics, PipelineOutput,
    StgServiceRequestFlat, StgSrCodeExploded, ValidationReport, VectorUsageSnapshot,
};
use refractive_swan_core::fhir::Bundle;
use refractive_swan_datamart::DatamartError;
use refractive_swan_observability::{log_no_match, log_pipeline_output_with_summary};
use refractive_swan_pipeline::{PipelineError, PipelineRunConfig};
use refractive_swan_terminology::codesystem::LicenseTier;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    types::MapQuery,
    utils::{ApiError, ApiState},
};

/// Main mapping entrypoint: ingest Bundles, map, persist, and respond.
pub async fn map_bundles(
    State(state): State<ApiState>,
    Query(params): Query<MapQuery>,
    body: Bytes,
) -> Result<axum::response::Response, ApiError> {
    let request_id = Uuid::new_v4();
    let bundles = parse_bundles(&body, request_id)?;
    if bundles.is_empty() {
        return Err(ApiError::invalid_json(
            "request body did not contain any Bundles",
            request_id,
        ));
    }
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} map_bundles start bundles={}",
        bundles.len()
    );

    let mut response = AggregatedPipelineOutput::default();
    let policy = state.plane.policy();
    let mut request_metrics = PipelineMetrics {
        compliance_mode: Some(policy.mode.as_str().to_string()),
        mesh_node_id: Some(state.plane.node_id().to_string()),
        ..PipelineMetrics::default()
    };
    let pipeline = state.plane.pipeline();
    let datamart = state.plane.datamart();
    let vector_override = matches!(params.vector.as_deref(), Some("disabled"));
    let vector_context = state.plane.vector_context();

    for bundle in bundles {
        let exec = {
            let config = PipelineRunConfig::default();
            pipeline
                .map_bundle_with_validation(
                    &bundle,
                    &config,
                    if vector_override {
                        None
                    } else {
                        vector_context.as_ref()
                    },
                )
                .map_err(|err| match err {
                    PipelineError::Ingestion(source) => {
                        ApiError::ingestion(source.to_string(), request_id)
                    }
                })?
        };
        enforce_export_policy(&exec.output, policy, request_id)?;

        log_pipeline_output_with_summary(
            &exec.output.flats,
            &exec.output.exploded_codes,
            &exec.output.mapping_results,
            &exec.metrics,
            &mut request_metrics,
        );
        if let Err(err) = datamart.persist(&exec.output, policy).await {
            match err {
                DatamartError::Disabled => {
                    warn!(
                        target: "refractive_swan_api",
                        "request_id={request_id} datamart persist skipped (disabled)"
                    );
                }
                other => {
                    warn!(
                        target: "refractive_swan_api",
                        "request_id={request_id} datamart persist failed: {other}"
                    );
                }
            }
        }

        for mapping in &exec.output.mapping_results {
            if matches!(mapping.state, MappingState::NoMatch) {
                log_no_match(mapping);
            }
        }
        response.record_output(exec.output, exec.validation);
    }

    {
        let metrics_handle = state.plane.metrics();
        let mut global = metrics_handle.lock().await;
        global.merge(&request_metrics);
    }
    let total_flats = response.flats.len();
    let total_mappings = response.mapping_results.len();
    let total_dim_concepts = response.dim_concepts.len();
    let payload = response.into_body();

    info!(
        target: "refractive_swan_api",
        "request_id={request_id} map_bundles complete bundles={} flats={} mappings={} dim_concepts={} automap={} needs_review={} no_match={} license_blocked={} compliance_mode={}",
        request_metrics.bundle_count,
        total_flats,
        total_mappings,
        total_dim_concepts,
        request_metrics.auto_mapped,
        request_metrics.needs_review,
        request_metrics.no_match,
        request_metrics.license_blocked,
        state.plane.policy().mode.as_str()
    );
    if request_metrics.license_blocked > 0 {
        warn!(
            target: "refractive_swan_compliance",
            "request_id={request_id} compliance_blocked reason=license_blocked mode={} count={}",
            state.plane.policy().mode.as_str(),
            request_metrics.license_blocked
        );
    }

    Ok(Json(payload).into_response())
}

#[derive(Default)]
struct AggregatedPipelineOutput {
    flats: Vec<StgServiceRequestFlat>,
    exploded_codes: Vec<StgSrCodeExploded>,
    mapping_results: Vec<MappingResult>,
    dim_concepts: Vec<DimNCITConcept>,
    seen_dim_ids: HashSet<String>,
    vector_usage: Option<VectorUsageSnapshot>,
    validation_reports: Vec<ValidationReport>,
}

impl AggregatedPipelineOutput {
    fn record_output(&mut self, output: PipelineOutput, validation: ValidationReport) {
        self.flats.extend(output.flats);
        self.exploded_codes.extend(output.exploded_codes);
        self.mapping_results.extend(output.mapping_results);
        for concept in output.dim_concepts {
            if self.seen_dim_ids.insert(concept.ncit_id.clone()) {
                self.dim_concepts.push(concept);
            }
        }
        if self.vector_usage.is_none() {
            self.vector_usage = output.vector_usage;
        }
        self.validation_reports.push(validation);
    }

    fn into_body(self) -> MapBundlesBody {
        MapBundlesBody {
            flats: self.flats,
            exploded_codes: self.exploded_codes,
            mapping_results: self.mapping_results,
            dim_concepts: self.dim_concepts,
            vector_usage: self.vector_usage,
            validation_reports: self.validation_reports,
        }
    }
}

#[derive(serde::Serialize)]
struct MapBundlesBody {
    flats: Vec<StgServiceRequestFlat>,
    exploded_codes: Vec<StgSrCodeExploded>,
    mapping_results: Vec<MappingResult>,
    dim_concepts: Vec<DimNCITConcept>,
    vector_usage: Option<VectorUsageSnapshot>,
    validation_reports: Vec<ValidationReport>,
}

/// Apply compliance export guard to a pipeline output.
fn enforce_export_policy(
    output: &PipelineOutput,
    policy: &refractive_swan_compliance::Policy,
    request_id: Uuid,
) -> Result<(), ApiError> {
    let tiers = license_tiers_from_output(output);
    assert_export_allowed(&tiers, policy).map_err(|err| {
        ApiError::compliance(
            format!(
                "export blocked by compliance mode {}: {}",
                policy.mode.as_str(),
                err
            ),
            request_id,
        )
    })
}

/// Collect license tiers from mapping results for policy checks.
fn license_tiers_from_output(output: &PipelineOutput) -> Vec<LicenseTier> {
    let mut tiers = HashSet::new();
    for mapping in &output.mapping_results {
        if let Some(label) = mapping.license_tier.as_deref()
            && let Some(tier) = parse_license_tier(label)
        {
            tiers.insert(tier);
        }
    }
    tiers.into_iter().collect()
}

/// Parse a license tier label into the enum used by policy checks.
fn parse_license_tier(value: &str) -> Option<LicenseTier> {
    match value.trim() {
        "licensed" => Some(LicenseTier::Licensed),
        "open" => Some(LicenseTier::Open),
        "internal_only" => Some(LicenseTier::InternalOnly),
        _ => None,
    }
}

/// Parse JSON or NDJSON body into Bundles.
fn parse_bundles(body: &[u8], request_id: Uuid) -> Result<Vec<Bundle>, ApiError> {
    if body.iter().all(|byte| byte.is_ascii_whitespace()) {
        return Err(ApiError::invalid_json("request body is empty", request_id));
    }

    match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(map)) => {
            let bundle: Bundle = serde_json::from_value(Value::Object(map))
                .map_err(|err| ApiError::invalid_json(err.to_string(), request_id))?;
            Ok(vec![bundle])
        }
        Ok(Value::Array(items)) => {
            let mut bundles = Vec::with_capacity(items.len());
            for item in items {
                let bundle: Bundle = serde_json::from_value(item)
                    .map_err(|err| ApiError::invalid_json(err.to_string(), request_id))?;
                bundles.push(bundle);
            }
            Ok(bundles)
        }
        Ok(_) => Err(ApiError::invalid_json(
            "expected a Bundle object or array of Bundles",
            request_id,
        )),
        Err(_) => parse_ndjson(body, request_id),
    }
}

/// Parse NDJSON payload line by line into Bundles.
fn parse_ndjson(body: &[u8], request_id: Uuid) -> Result<Vec<Bundle>, ApiError> {
    let text = std::str::from_utf8(body)
        .map_err(|err| ApiError::invalid_json(err.to_string(), request_id))?;
    let mut bundles = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let bundle: Bundle = serde_json::from_str(trimmed).map_err(|err| {
            ApiError::invalid_json(format!("ndjson line {}: {}", idx + 1, err), request_id)
        })?;
        bundles.push(bundle);
    }

    if bundles.is_empty() {
        Err(ApiError::invalid_json(
            "request body did not contain Bundle entries",
            request_id,
        ))
    } else {
        Ok(bundles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bundle_json(id: &str) -> String {
        serde_json::json!({
            "resourceType": "Bundle",
            "type": "collection",
            "entry": [{
                "resource": {
                    "resourceType": "ServiceRequest",
                    "id": id,
                    "status": "active",
                    "intent": "order",
                    "subject": { "reference": "Patient/p1" }
                }
            }]
        })
        .to_string()
    }

    #[test]
    fn parses_single_bundle_object() {
        let body = sample_bundle_json("sr-1");
        let bundles = parse_bundles(body.as_bytes(), Uuid::nil()).unwrap();
        assert_eq!(bundles.len(), 1);
        assert_eq!(bundles[0].resource_type, "Bundle");
    }

    #[test]
    fn parses_array_of_bundles() {
        let body = format!(
            "[{},{}]",
            sample_bundle_json("sr-1"),
            sample_bundle_json("sr-2")
        );
        let bundles = parse_bundles(body.as_bytes(), Uuid::nil()).unwrap();
        assert_eq!(bundles.len(), 2);
    }

    #[test]
    fn parses_ndjson_payload() {
        let body = format!(
            "{}\n{}\n",
            sample_bundle_json("sr-1"),
            sample_bundle_json("sr-2")
        );
        let bundles = parse_bundles(body.as_bytes(), Uuid::nil()).unwrap();
        assert_eq!(bundles.len(), 2);
    }

    #[test]
    fn errors_on_whitespace_body() {
        let body = "   ";
        let err = parse_bundles(body.as_bytes(), Uuid::nil()).unwrap_err();
        assert!(matches!(err, ApiError::InvalidJson { .. }));
    }

    #[test]
    fn errors_on_invalid_json() {
        let body = "{ not valid json }";
        let err = parse_bundles(body.as_bytes(), Uuid::nil()).unwrap_err();
        assert!(matches!(err, ApiError::InvalidJson { .. }));
    }
}
