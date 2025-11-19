use dfps_compliance::ComplianceConfig;
use dfps_datamart::{LoadError, load_from_pipeline_output, migrate};
use dfps_pipeline::bundle_to_mapped_sr;
use dfps_test_suite::regression;
use sqlx::SqlitePool;
use std::sync::{Mutex, OnceLock};

async fn load_baseline(pool: &SqlitePool, policy: &dfps_compliance::Policy) {
    let bundle = regression::baseline_fhir_bundle();
    let output = bundle_to_mapped_sr(&bundle).expect("pipeline maps baseline bundle");
    load_from_pipeline_output(pool, &output, policy)
        .await
        .expect("load baseline into warehouse");
}

async fn load_unknown(pool: &SqlitePool, policy: &dfps_compliance::Policy) {
    let bundle = regression::fhir_bundle_unknown_code();
    let output = bundle_to_mapped_sr(&bundle).expect("pipeline maps unknown bundle");
    load_from_pipeline_output(pool, &output, policy)
        .await
        .expect("load unknown into warehouse");
}

fn env_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

#[tokio::test]
async fn warehouse_loads_baseline_and_unknown_bundles() {
    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("connect sqlite");
    migrate(&pool).await.expect("apply migrations");
    let policy = ComplianceConfig::from_env()
        .and_then(|cfg| cfg.load_policy())
        .expect("default compliance policy");

    load_baseline(&pool, &policy).await;
    load_unknown(&pool, &policy).await;

    let patients: i64 = sqlx::query_scalar("select count(*) from dim_patient")
        .fetch_one(&pool)
        .await
        .unwrap();
    let encounters: i64 = sqlx::query_scalar("select count(*) from dim_encounter")
        .fetch_one(&pool)
        .await
        .unwrap();
    let codes: i64 = sqlx::query_scalar("select count(*) from dim_code")
        .fetch_one(&pool)
        .await
        .unwrap();
    let facts: i64 = sqlx::query_scalar("select count(*) from fact_service_request")
        .fetch_one(&pool)
        .await
        .unwrap();
    let no_match_dims: i64 =
        sqlx::query_scalar("select count(*) from dim_ncit where ncit_id = 'NO_MATCH'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert!(patients >= 1, "should load at least one patient");
    assert!(encounters >= 1, "should load at least one encounter");
    assert!(codes >= 1, "should load exploded codes");
    assert!(
        facts >= 2,
        "should load facts for baseline + unknown bundle"
    );
    assert!(
        no_match_dims >= 1,
        "NO_MATCH ncit dim should be created when unknown codes exist"
    );

    // Check FK consistency by joining facts back to dims.
    let fk_checks: (i64, i64, i64) = sqlx::query_as(
        "select
            (select count(*) from fact_service_request f join dim_patient p on f.patient_key = p.patient_key),
            (select count(*) from fact_service_request f join dim_code c on f.code_key = c.code_key),
            (select count(*) from fact_service_request f join dim_ncit n on f.ncit_key = n.ncit_key or f.ncit_key is null)"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(fk_checks.0, facts, "all facts have patient FK");
    assert_eq!(fk_checks.1, facts, "all facts have code FK");
    assert_eq!(
        fk_checks.2, facts,
        "all facts have ncit FK (including NO_MATCH)"
    );
}

#[tokio::test]
async fn bundle_respects_compliance_mode_open_source() {
    let _lock = env_guard().lock().unwrap();
    unsafe { std::env::set_var("DFPS_COMPLIANCE_MODE", "open_source") };

    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("connect sqlite");
    migrate(&pool).await.expect("apply migrations");

    // Use baseline bundle (contains CPT) to trigger license-blocked path.
    let bundle = regression::baseline_fhir_bundle();
    let output = bundle_to_mapped_sr(&bundle).expect("pipeline maps baseline bundle");
    let licensed = output
        .mapping_results
        .iter()
        .any(|m| m.license_tier.as_deref() == Some("licensed"));
    assert!(
        licensed,
        "baseline bundle should include licensed mappings to exercise open_source compliance"
    );

    let policy = ComplianceConfig::from_env()
        .and_then(|cfg| cfg.load_policy())
        .expect("open_source policy");
    let load_err = load_from_pipeline_output(&pool, &output, &policy).await;
    assert!(
        matches!(load_err, Err(LoadError::Compliance(_))),
        "export should be denied by compliance policy"
    );

    unsafe { std::env::remove_var("DFPS_COMPLIANCE_MODE") };
}

#[tokio::test]
async fn bundle_allows_licensed_codes_in_partner_mode() {
    let _lock = env_guard().lock().unwrap();
    unsafe { std::env::set_var("DFPS_COMPLIANCE_MODE", "partner") };

    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("connect sqlite");
    migrate(&pool).await.expect("apply migrations");

    let bundle = regression::baseline_fhir_bundle();
    let output = bundle_to_mapped_sr(&bundle).expect("pipeline maps baseline bundle");
    let blocked = output
        .mapping_results
        .iter()
        .any(|m| m.reason.as_deref() == Some("license_blocked"));
    assert!(
        !blocked,
        "partner mode should allow licensed tier mappings to proceed"
    );

    let policy = ComplianceConfig::from_env()
        .and_then(|cfg| cfg.load_policy())
        .expect("partner policy");
    load_from_pipeline_output(&pool, &output, &policy)
        .await
        .expect("partner mode should permit warehouse load");

    unsafe { std::env::remove_var("DFPS_COMPLIANCE_MODE") };
}
