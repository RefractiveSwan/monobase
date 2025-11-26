use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{
    HttpRequest, HttpResponse, Result,
    error::{ErrorInternalServerError, ErrorNotFound},
    web,
};
use bytes::BytesMut;
use futures_util::TryStreamExt;
use serde::Deserialize;
use serde_json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs};

use crate::{
    handlers::home,
    state::{AppState, RegressionJobStatus},
    views::models::{
        DatasetAdminEntryView, DatasetAdminView, DatasetFixtureView, PageContext,
        RegressionJobStatusView, RegressionJobView,
    },
    views::pages::admin::render_dataset_admin_page,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/admin/datasets").route(web::get().to(datasets_page)))
        .service(web::resource("/admin/datasets/refresh").route(web::post().to(refresh_datasets)))
        .service(web::resource("/admin/datasets/upload").route(web::post().to(upload_dataset)))
        .service(
            web::resource("/admin/datasets/{name}/toggle").route(web::post().to(toggle_dataset)),
        )
        .service(
            web::resource("/admin/datasets/{name}/download/{artifact}")
                .route(web::get().to(download_dataset_artifact)),
        )
        .service(web::resource("/admin/regression/run").route(web::post().to(run_regression_job)))
        .service(web::resource("/admin/maintenance/clear").route(web::post().to(clear_caches)))
        .service(web::resource("/admin/datamart/reset").route(web::post().to(reset_datamart)));
}

pub async fn datasets_page(state: web::Data<AppState>) -> Result<HttpResponse> {
    let ctx = build_admin_context(state.get_ref(), None).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_dataset_admin_page(&ctx)))
}

pub async fn refresh_datasets(state: web::Data<AppState>) -> Result<HttpResponse> {
    let ctx = build_admin_context(state.get_ref(), Some("Dataset catalog refreshed.".into())).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_dataset_admin_page(&ctx)))
}

pub async fn toggle_dataset(
    state: web::Data<AppState>,
    path: web::Path<String>,
    form: web::Form<ToggleForm>,
) -> Result<HttpResponse> {
    let name = path.into_inner();
    if !valid_dataset_name(&name) {
        return Ok(HttpResponse::BadRequest().body("Invalid dataset name"));
    }
    let enable = form.action == "enable";
    state.set_dataset_enabled(&name, enable);
    let message = if enable {
        format!("Enabled {name}")
    } else {
        format!("Disabled {name}")
    };
    let ctx = build_admin_context(state.get_ref(), Some(message)).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_dataset_admin_page(&ctx)))
}

#[derive(Deserialize)]
pub struct ToggleForm {
    pub action: String,
}

pub async fn upload_dataset(
    state: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let mut manifest_bytes = None;
    let mut data_bytes = None;

    while let Some(mut field) = payload.try_next().await? {
        let name = field.name().to_string();
        let mut bytes = BytesMut::new();
        while let Some(chunk) = field.try_next().await? {
            bytes.extend_from_slice(&chunk);
        }
        match name.as_str() {
            "manifest" => manifest_bytes = Some(bytes.to_vec()),
            "dataset" => data_bytes = Some(bytes.to_vec()),
            _ => {}
        }
    }

    let manifest_bytes = match manifest_bytes {
        Some(bytes) => bytes,
        None => {
            return Ok(HttpResponse::BadRequest()
                .body("Upload requires a manifest JSON file under the `manifest` field"));
        }
    };
    let dataset_bytes = match data_bytes {
        Some(bytes) => bytes,
        None => {
            return Ok(HttpResponse::BadRequest()
                .body("Upload requires an NDJSON payload under the `dataset` field"));
        }
    };

    let mut manifest: refractive_swan_eval::DatasetManifest =
        match serde_json::from_slice(&manifest_bytes) {
            Ok(manifest) => manifest,
            Err(err) => {
                return Ok(HttpResponse::BadRequest().body(format!("Invalid manifest JSON: {err}")));
            }
        };
    if !valid_dataset_name(&manifest.name) {
        return Ok(HttpResponse::BadRequest().body("Manifest `name` contains invalid characters"));
    }

    let dataset_root = state.dataset_root();
    if !dataset_root.exists() {
        fs::create_dir_all(&dataset_root).map_err(ErrorInternalServerError)?;
    }
    let dataset_path = dataset_root.join(format!("{}.ndjson", manifest.name));
    fs::write(&dataset_path, &dataset_bytes).map_err(ErrorInternalServerError)?;

    let checksum = Sha256::digest(&dataset_bytes);
    manifest.sha256 = format!("{:x}", checksum);
    manifest.n_cases = dataset_bytes
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .count();
    let manifest_path = dataset_root.join(format!("{}.manifest.json", manifest.name));
    if let Ok(json) = serde_json::to_string_pretty(&manifest) {
        fs::write(&manifest_path, json).map_err(ErrorInternalServerError)?;
    }

    let ctx = build_admin_context(
        state.get_ref(),
        Some(format!("Uploaded dataset {}", manifest.name)),
    )
    .await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_dataset_admin_page(&ctx)))
}

#[derive(Deserialize)]
pub struct DatasetDownloadPath {
    pub name: String,
    pub artifact: String,
}

pub async fn download_dataset_artifact(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<DatasetDownloadPath>,
) -> Result<HttpResponse> {
    if !valid_dataset_name(&path.name) {
        return Err(ErrorNotFound("invalid dataset name"));
    }
    let dataset_root = state.dataset_root();
    let file = match path.artifact.as_str() {
        "manifest" => dataset_root.join(format!("{}.manifest.json", path.name)),
        "data" => dataset_root.join(format!("{}.ndjson", path.name)),
        _ => return Err(ErrorNotFound("unknown artifact")),
    };
    let named = NamedFile::open(file).map_err(ErrorNotFound)?;
    Ok(named.use_last_modified(true).into_response(&req))
}

pub async fn run_regression_job(state: web::Data<AppState>) -> Result<HttpResponse> {
    state.enqueue_regression_job("Smoke suite".into());
    let ctx = build_admin_context(
        state.get_ref(),
        Some("Triggered regression smoke suite.".into()),
    )
    .await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_dataset_admin_page(&ctx)))
}

pub async fn clear_caches(state: web::Data<AppState>) -> Result<HttpResponse> {
    state.clear_caches();
    let ctx = build_admin_context(
        state.get_ref(),
        Some("Cleared mapping history/log caches.".into()),
    )
    .await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_dataset_admin_page(&ctx)))
}

pub async fn reset_datamart(state: web::Data<AppState>) -> Result<HttpResponse> {
    let message = match state.reset_datamart() {
        Ok(_) => "Datamart SQLite file reset.".to_string(),
        Err(err) => format!("Datamart reset failed: {err}"),
    };
    let ctx = build_admin_context(state.get_ref(), Some(message)).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_dataset_admin_page(&ctx)))
}

async fn build_admin_context(state: &AppState, message: Option<String>) -> PageContext {
    let mut ctx = home::build_base_context(state).await;
    ctx.dataset_admin = Some(build_dataset_admin_view(state));
    ctx.regression_jobs = build_regression_views(state);
    ctx.maintenance_message = message;
    ctx
}

fn build_dataset_admin_view(state: &AppState) -> DatasetAdminView {
    let manifests = state.dataset_store.list_manifests().unwrap_or_default();
    let disabled = state.disabled_datasets();
    let disabled_set: BTreeSet<_> = disabled.into_iter().collect();
    let mut entries = Vec::new();
    for manifest in manifests {
        entries.push(DatasetAdminEntryView {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            license: manifest.license.clone(),
            source: manifest.source.clone(),
            tier: tier_label(manifest.tier).to_string(),
            notes: manifest.notes.clone(),
            n_cases: manifest.n_cases,
            sha256: manifest.sha256.clone(),
            disabled: disabled_set.contains(&manifest.name),
        });
    }
    let mut fixtures = Vec::new();
    for entry in &entries {
        fixtures.push(DatasetFixtureView {
            name: entry.name.clone(),
            manifest_href: format!("/admin/datasets/{}/download/manifest", entry.name),
            data_href: format!("/admin/datasets/{}/download/data", entry.name),
        });
    }
    DatasetAdminView {
        entries,
        fixtures,
        disabled_total: disabled_set.len(),
        root: state.dataset_root().display().to_string(),
        datamart_url: state.datamart_url(),
        datamart_reset_supported: state
            .datamart_url()
            .map(|url| url.starts_with("sqlite://"))
            .unwrap_or(false),
    }
}

fn build_regression_views(state: &AppState) -> Vec<RegressionJobView> {
    state
        .regression_jobs_snapshot()
        .into_iter()
        .map(|job| RegressionJobView {
            id: job.id.to_string(),
            label: job.label,
            status: map_regression_status(job.status),
            submitted_at: job.submitted_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            completed_at: job
                .completed_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            log: job.log.clone(),
            error: job.error.clone(),
        })
        .collect()
}

fn map_regression_status(status: RegressionJobStatus) -> RegressionJobStatusView {
    match status {
        RegressionJobStatus::Pending => RegressionJobStatusView::Pending,
        RegressionJobStatus::Running => RegressionJobStatusView::Running,
        RegressionJobStatus::Passed => RegressionJobStatusView::Passed,
        RegressionJobStatus::Failed => RegressionJobStatusView::Failed,
    }
}

fn tier_label(tier: refractive_swan_eval::DatasetTier) -> &'static str {
    match tier {
        refractive_swan_eval::DatasetTier::Gold => "gold",
        refractive_swan_eval::DatasetTier::Silver => "silver",
        refractive_swan_eval::DatasetTier::Bronze => "bronze",
        refractive_swan_eval::DatasetTier::Uncategorized => "uncategorized",
    }
}

fn valid_dataset_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}
