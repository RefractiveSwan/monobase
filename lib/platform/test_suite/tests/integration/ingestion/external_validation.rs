//! External validator integration tests (FHIR-CONF-015).

use axum::{Router, http::StatusCode, response::IntoResponse, routing::post};
use refractive_swan_core::fhir::Bundle;
use refractive_swan_ingestion::{
    IngestionError, bundle_to_staging_with_validation,
    validation::{
        ExternalValidationContext, ValidationMode, ValidationReport,
        external::{
            ExternalValidationError, ExternalValidationOutcome, ExternalValidator, OperationOutcome,
        },
    },
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
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

#[tokio::test]
async fn external_issues_merge_into_report() {
    let (_addr, shutdown, handle) = spawn_validator(true).await;
    let bundle: Bundle = refractive_swan_test_suite::regression::baseline_fhir_bundle();
    let report: ValidationReport = tokio::task::spawn_blocking(move || {
        let validator = BlockingValidator::new(_addr.to_string());
        let ctx = ExternalValidationContext {
            validator: Some(&validator),
            profile_url: None,
        };
        refractive_swan_ingestion::validation::validate_bundle_with_external_profile(
            &bundle,
            ValidationMode::ExternalStrict,
            ctx,
        )
    })
    .await
    .expect("blocking eval");

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
    let (_addr, shutdown, handle) = spawn_validator(true).await;
    let bundle: Bundle = refractive_swan_test_suite::regression::baseline_fhir_bundle();
    let outcome = tokio::task::spawn_blocking(move || {
        let validator = BlockingValidator::new(_addr.to_string());
        let ctx = ExternalValidationContext {
            validator: Some(&validator),
            profile_url: None,
        };
        bundle_to_staging_with_validation(&bundle, ValidationMode::ExternalStrict, ctx)
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
    let (_addr, shutdown, handle) = spawn_validator(false).await;
    let bundle: Bundle = refractive_swan_test_suite::regression::baseline_fhir_bundle();
    let report = tokio::task::spawn_blocking(move || {
        let validator = BlockingValidator::new(_addr.to_string());
        let ctx = ExternalValidationContext {
            validator: Some(&validator),
            profile_url: None,
        };
        refractive_swan_ingestion::validation::validate_bundle_with_external_profile(
            &bundle,
            ValidationMode::ExternalPreferred,
            ctx,
        )
    })
    .await
    .expect("join blocking");
    assert!(!report.has_errors());

    let _ = shutdown.send(());
    handle.await.expect("validator join");
}

#[derive(Clone)]
struct BlockingValidator {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl BlockingValidator {
    fn new(base_url: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("validator client");
        Self { client, base_url }
    }
}

impl ExternalValidator for BlockingValidator {
    fn validate_bundle(
        &self,
        bundle: &Bundle,
        profile_url: Option<&str>,
    ) -> Result<ExternalValidationOutcome, ExternalValidationError> {
        let mut url = self.base_url.clone();
        if !url.ends_with("/$validate") {
            if url.ends_with('/') {
                url.push_str("$validate");
            } else {
                url.push_str("/$validate");
            }
        }
        let mut request = self.client.post(url).json(bundle);
        if let Some(profile) = profile_url {
            request = request.query(&[("profile", profile)]);
        }
        let response = request
            .send()
            .map_err(|err| ExternalValidationError::Failed(err.to_string()))?;
        let outcome: OperationOutcome = response
            .json()
            .map_err(|err| ExternalValidationError::Parse(err.to_string()))?;
        Ok(Some(outcome))
    }
}
