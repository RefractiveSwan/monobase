use std::fs;

use axum::{
    Json,
    extract::{Json as JsonPayload, Path, State},
    response::IntoResponse,
};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use log::info;
use refractive_swan_contracts::AdminEventKind;
use serde_json::{self, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    types::UploadDatasetRequest,
    utils::{ApiError, ApiState},
};

/// Refresh dataset manifests and return the updated list.
pub async fn refresh(State(state): State<ApiState>) -> Result<axum::response::Response, ApiError> {
    {
        let mut registry = state.dataset_registry.lock().await;
        registry.mark_refreshed();
    }
    state
        .record_admin_event(
            AdminEventKind::DatasetRefresh,
            "dataset registry refresh triggered",
        )
        .await;
    super::super::handlers::eval::list_eval_datasets(State(state)).await
}

/// Upload a new dataset manifest + NDJSON payload.
pub async fn upload(
    State(state): State<ApiState>,
    JsonPayload(body): JsonPayload<UploadDatasetRequest>,
) -> Result<axum::response::Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(
        target: "refractive_swan_observability.dataset_admin",
        "request_id={request_id} event=dataset_upload name={} n_cases={}",
        body.manifest.name, body.manifest.n_cases
    );
    let data = B64
        .decode(body.ndjson_b64.as_bytes())
        .map_err(|err| ApiError::invalid_dataset(err.to_string(), request_id))?;

    let computed_sha = format!("{:x}", Sha256::digest(&data));
    if !body.manifest.sha256.is_empty() && !body.manifest.sha256.eq_ignore_ascii_case(&computed_sha)
    {
        return Err(ApiError::invalid_dataset(
            format!(
                "checksum mismatch: manifest sha={} computed sha={}",
                body.manifest.sha256, computed_sha
            ),
            request_id,
        ));
    }

    let dataset_root = state.plane.dataset_store().data_root().to_path_buf();
    fs::create_dir_all(&dataset_root)
        .map_err(|err| ApiError::ingestion(err.to_string(), request_id))?;
    let name = body.manifest.name.clone();
    let manifest_path = dataset_root.join(format!("{}.manifest.json", name));
    let data_path = dataset_root.join(format!("{}.ndjson", name));
    let mut manifest = body.manifest;
    manifest.sha256 = computed_sha.clone();
    manifest.n_cases = data
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .count();
    manifest.disabled = false;

    fs::write(&data_path, &data).map_err(|err| ApiError::ingestion(err.to_string(), request_id))?;
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|err| ApiError::ingestion(err.to_string(), request_id))?;
    fs::write(&manifest_path, manifest_json)
        .map_err(|err| ApiError::ingestion(err.to_string(), request_id))?;

    {
        let mut registry = state.dataset_registry.lock().await;
        registry.mark_refreshed();
        registry.enable(&name);
    }
    state
        .record_admin_event(
            AdminEventKind::DatasetUpload,
            format!(
                "dataset '{}' uploaded (cases={}, sha={})",
                name, manifest.n_cases, manifest.sha256
            ),
        )
        .await;

    Ok(Json(json!({
        "status": "ok",
        "sha256": computed_sha,
    }))
    .into_response())
}

/// Delete a dataset by name and mark it disabled.
pub async fn delete(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let request_id = Uuid::new_v4();
    let dataset_root = state.plane.dataset_store().data_root().to_path_buf();
    let manifest_path = dataset_root.join(format!("{}.manifest.json", name));
    let data_path = dataset_root.join(format!("{}.ndjson", name));
    let _ = fs::remove_file(manifest_path);
    let _ = fs::remove_file(data_path);
    let mut registry = state.dataset_registry.lock().await;
    registry.disable(&name);
    registry.mark_refreshed();
    info!(
        target: "refractive_swan_observability.dataset_admin",
        "request_id={request_id} event=dataset_delete name={}",
        name
    );
    state
        .record_admin_event(
            AdminEventKind::DatasetDelete,
            format!("dataset '{}' deleted", name),
        )
        .await;
    Ok(Json(json!({ "status": "ok", "disabled": name })).into_response())
}
