use axum::{Router, http::StatusCode, response::IntoResponse, routing::post};
use dfps_core::fhir::Bundle;
use dfps_ingestion::{
    IngestionError, bundle_to_staging_with_validation,
    validation::{ValidationMode, ValidationReport},
};
use serde_json::json;
use once_cell::sync::Lazy;
use std::{net::SocketAddr, sync::{Arc, Mutex}};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};

async fn mock_validate_handler(with_issue: Arc<bool>) -> impl IntoResponse {
    if *with_issue {
        let outcome = json!({
            "resourceType": "OperationOutcome",
            "issues": [{
                "severity": "error",
                "code": "invalid",
                "diagnostics": "External validator: missing subject"
            }]
        });
        (StatusCode::OK, axum::Json(outcome))
    } else {
        let outcome = json!({ "resourceType": "OperationOutcome", "issues": [] });
        (StatusCode::OK, axum::Json(outcome))
    }
}

async fn spawn_validator(with_issue: bool) -> (SocketAddr, oneshot::Sender<()>, JoinHandle<()>) {
    let state = Arc::new(with_issue);
    let app = Router::new().route(
        "/fhir/$validate",
        post({
            let state = state.clone();
            move || mock_validate_handler(state.clone())
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind validator");
    let addr = listener.local_addr().expect("validator addr");
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let handle = tokio::spawn(async move {
        let shutdown = async {
            let _ = shutdown_rx.await;
        };
        if let Err(err) = axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown)
            .await
        {
            panic!("mock validator error: {err}");
        }
    });

    (addr, shutdown_tx, handle)
}

fn set_env_for(with_issue: bool) {
    let _ = with_issue;
    unsafe {
        std::env::set_var("DFPS_FHIR_VALIDATOR_BASE_URL", "mock://validator");
        std::env::set_var(
            "DFPS_FHIR_VALIDATOR_MOCK",
            if with_issue { "error" } else { "ok" },
        );
        std::env::set_var("DFPS_FHIR_VALIDATOR_TIMEOUT_SECS", "5");
    }
}

static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[tokio::test]
async fn external_issues_merge_into_report() {
    let _guard = ENV_LOCK.lock().unwrap();
    let (_addr, shutdown, handle) = spawn_validator(true).await;
    set_env_for(true);

    let bundle: Bundle = dfps_test_suite::regression::baseline_fhir_bundle();
    let report: ValidationReport =
        dfps_ingestion::validation::validate_bundle_with_external_profile(
            &bundle,
            ValidationMode::ExternalStrict,
            None,
        );

    assert!(report.has_errors(), "external error should surface");
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.id.starts_with("VAL_EXTERNAL"))
    );

    let _ = shutdown.send(());
    handle.await.expect("validator join");
}

#[tokio::test]
async fn external_strict_blocks_ingestion_on_error() {
    let _guard = ENV_LOCK.lock().unwrap();
    let (_addr, shutdown, handle) = spawn_validator(true).await;
    set_env_for(true);

    let bundle: Bundle = dfps_test_suite::regression::baseline_fhir_bundle();
    let outcome = tokio::task::spawn_blocking(move || {
        bundle_to_staging_with_validation(&bundle, ValidationMode::ExternalStrict)
    })
    .await
    .expect("join blocking");
    match outcome {
        Err(IngestionError::ValidationFailed(issues)) => {
            assert!(
                issues
                    .iter()
                    .any(|issue| issue.id.starts_with("VAL_EXTERNAL")),
                "external validation issues should be present"
            );
        }
        other => panic!("expected validation failure, got {:?}", other),
    }

    let _ = shutdown.send(());
    handle.await.expect("validator join");
}

#[tokio::test]
async fn external_preferred_allows_pass_through_when_clean() {
    let _guard = ENV_LOCK.lock().unwrap();
    let (_addr, shutdown, handle) = spawn_validator(false).await;
    set_env_for(false);

    let bundle: Bundle = dfps_test_suite::regression::baseline_fhir_bundle();
    let report = tokio::task::spawn_blocking(move || {
        dfps_ingestion::validation::validate_bundle_with_external_profile(
            &bundle,
            ValidationMode::ExternalPreferred,
            None,
        )
    })
    .await
    .expect("join blocking");
    assert!(!report.has_errors());

    let _ = shutdown.send(());
    handle.await.expect("validator join");
}
