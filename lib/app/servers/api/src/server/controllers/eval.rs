use axum::{
    Json,
    extract::{Json as JsonPayload, Query, State},
    response::IntoResponse,
};
use log::info;
use uuid::Uuid;

use crate::{
    dto::EvalRunResponse,
    types::{EvalQuery, EvalRunRequest},
    utils::{ApiError, ApiState},
};

/// Return eval summary for a single dataset and top_k.
pub async fn eval_summary(
    State(state): State<ApiState>,
    Query(query): Query<EvalQuery>,
) -> Result<axum::response::Response, ApiError> {
    let request_id = Uuid::new_v4();
    let dataset = query.dataset;
    info!(target: "refractive_swan_api", "request_id={request_id} eval_summary dataset={dataset}");
    let cases = state
        .plane
        .dataset_store()
        .load_dataset(&dataset)
        .map_err(|err| ApiError::invalid_dataset(err.to_string(), request_id))?;
    let summary = run_eval_internal(&cases, query.top_k);
    Ok(Json(summary).into_response())
}

/// List dataset manifests with disabled/refresh metadata.
pub async fn list_datasets(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    let manifests = state
        .plane
        .dataset_store()
        .list_manifests()
        .map_err(|err| ApiError::invalid_dataset(err.to_string(), Uuid::new_v4()))?;
    let entries = {
        let registry = state.dataset_registry.lock().await;
        manifests
            .into_iter()
            .map(
                |manifest| refractive_swan_contracts::eval::DatasetListEntry {
                    disabled: manifest.disabled || registry.is_disabled(&manifest.name),
                    last_refresh_iso: registry.last_refresh_iso.clone(),
                    manifest,
                },
            )
            .collect::<Vec<_>>()
    };
    Ok(Json(entries).into_response())
}

/// Execute an eval run and cache latest summary.
pub async fn run_eval(
    State(state): State<ApiState>,
    JsonPayload(body): JsonPayload<EvalRunRequest>,
) -> Result<axum::response::Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} eval_run dataset={} top_k={}",
        body.dataset,
        body.top_k
    );
    let outcome = state
        .plane
        .dataset_store()
        .load_dataset_with_manifest(&body.dataset)
        .map_err(|err| ApiError::invalid_dataset(err.to_string(), request_id))?;
    let summary = run_eval_internal(&outcome.cases, body.top_k);
    let response = EvalRunResponse {
        dataset: body.dataset.clone(),
        manifest: Some(outcome.manifest),
        summary,
    };
    {
        let mut latest = state.latest_eval.lock().await;
        *latest = Some(response.clone());
    }
    Ok(Json(response).into_response())
}

/// Return the most recent eval run (if any).
pub async fn latest(State(state): State<ApiState>) -> Result<axum::response::Response, ApiError> {
    let latest = state.latest_eval.lock().await;
    if let Some(run) = &*latest {
        Ok(Json(run).into_response())
    } else {
        Err(ApiError::invalid_dataset(
            "no eval has run yet".to_string(),
            Uuid::new_v4(),
        ))
    }
}

fn run_eval_internal(
    cases: &[refractive_swan_eval::EvalCase],
    top_k: usize,
) -> refractive_swan_eval::EvalSummary {
    let summary = refractive_swan_eval::run_eval_with_mapper(cases, |rows| {
        refractive_swan_mapping::map_staging_codes(rows).0
    });
    if top_k > 1 {
        // Placeholder until engine exposes true top-k.
        return summary;
    }
    summary
}
