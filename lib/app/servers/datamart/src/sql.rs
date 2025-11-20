use dfps_compliance::{Policy, assert_export_allowed};
use dfps_configuration::{EnvValueError, load_env};
use dfps_contracts::{
    AnalyticsSummaryResponse, AnalyticsSummaryRow, CohortResponse, CohortRow, LoadSummary,
    PipelineOutput,
};
use dfps_terminology::codesystem::LicenseTier;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Row, Sqlite, SqlitePool, Transaction};
use thiserror::Error;

use crate::{DimCode, DimEncounter, DimNCIT, DimPatient, FactServiceRequest};

pub const CREATE_DIM_PATIENT: &str = r#"
CREATE TABLE IF NOT EXISTS dim_patient (
  patient_key INTEGER PRIMARY KEY,
  patient_id TEXT NOT NULL
);"#;

pub const CREATE_DIM_ENCOUNTER: &str = r#"
CREATE TABLE IF NOT EXISTS dim_encounter (
  encounter_key INTEGER PRIMARY KEY,
  encounter_id TEXT NOT NULL,
  patient_key INTEGER NOT NULL REFERENCES dim_patient(patient_key)
);"#;

pub const CREATE_DIM_CODE: &str = r#"
CREATE TABLE IF NOT EXISTS dim_code (
  code_key INTEGER PRIMARY KEY,
  code_element_id TEXT NOT NULL,
  system TEXT,
  code TEXT,
  display TEXT
);"#;

pub const CREATE_DIM_NCIT: &str = r#"
CREATE TABLE IF NOT EXISTS dim_ncit (
  ncit_key INTEGER PRIMARY KEY,
  ncit_id TEXT NOT NULL,
  preferred_name TEXT,
  semantic_group TEXT
);"#;

pub const CREATE_FACT_SERVICE_REQUEST: &str = r#"
CREATE TABLE IF NOT EXISTS fact_service_request (
  sr_id TEXT PRIMARY KEY,
  patient_key INTEGER NOT NULL REFERENCES dim_patient(patient_key),
  encounter_key INTEGER REFERENCES dim_encounter(encounter_key),
  code_key INTEGER NOT NULL REFERENCES dim_code(code_key),
  ncit_key INTEGER REFERENCES dim_ncit(ncit_key),
  mapping_state TEXT NOT NULL,
  status TEXT,
  intent TEXT,
  description TEXT,
  ordered_at TEXT
);"#;

pub fn ddl_statements() -> &'static [&'static str] {
    &[
        CREATE_DIM_PATIENT,
        CREATE_DIM_ENCOUNTER,
        CREATE_DIM_CODE,
        CREATE_DIM_NCIT,
        CREATE_FACT_SERVICE_REQUEST,
    ]
}

const ORDER_DAY_EXPR: &str = "CASE WHEN fsr.ordered_at IS NULL OR LENGTH(fsr.ordered_at) < 10 THEN NULL ELSE substr(fsr.ordered_at, 1, 10) END";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseConfig {
    pub url: String,
    pub schema: Option<String>,
    pub max_connections: u32,
}

impl WarehouseConfig {
    pub fn from_env() -> Result<Self, WarehouseConfigError> {
        let _ = load_env("domain.datamart");
        let url = dfps_configuration::string_var("DFPS_WAREHOUSE_URL")
            .map_err(WarehouseConfigError::Env)?
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or(WarehouseConfigError::MissingUrl)?;
        let schema = dfps_configuration::string_var("DFPS_WAREHOUSE_SCHEMA")
            .map_err(WarehouseConfigError::Env)?
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let max_connections = dfps_configuration::u32_var("DFPS_WAREHOUSE_MAX_CONNECTIONS")
            .map_err(WarehouseConfigError::Env)?
            .unwrap_or(5);
        Ok(Self {
            url,
            schema,
            max_connections,
        })
    }
}

#[derive(Debug, Error)]
pub enum WarehouseConfigError {
    #[error("DFPS_WAREHOUSE_URL must be set")]
    MissingUrl,
    #[error("invalid warehouse env value: {0}")]
    Env(#[from] EnvValueError),
}

#[derive(Debug, Default, Clone)]
pub struct CohortFilters {
    pub ncit_id: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

pub async fn migrate(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    for stmt in ddl_statements() {
        sqlx::query(stmt).execute(pool).await?;
    }
    ensure_fact_mapping_state_column(pool).await?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error("export blocked by compliance policy: {0}")]
    Compliance(String),
}

/// Load a PipelineOutput into the warehouse, upserting dims and inserting facts.
pub async fn load_from_pipeline_output(
    pool: &Pool<Sqlite>,
    output: &PipelineOutput,
    policy: &Policy,
) -> Result<LoadSummary, LoadError> {
    enforce_export_policy(output, policy)?;

    let (dims, facts) = crate::from_pipeline_output(output);
    let mut tx = pool.begin().await?;
    let summary = LoadSummary {
        patients: upsert_patients(&mut tx, &dims.patients).await?,
        encounters: upsert_encounters(&mut tx, &dims.encounters).await?,
        codes: upsert_codes(&mut tx, &dims.codes).await?,
        ncit: upsert_ncit(&mut tx, &dims.ncit).await?,
        facts: insert_facts(&mut tx, &facts).await?,
    };

    tx.commit().await?;
    Ok(summary)
}

/// Load a sequence of pipeline outputs without buffering them all in memory.
pub async fn load_streaming_iter<I>(
    pool: &Pool<Sqlite>,
    outputs: &mut I,
    policy: &Policy,
) -> Result<LoadSummary, LoadError>
where
    I: Iterator<Item = PipelineOutput>,
{
    let mut agg = LoadSummary::default();
    while let Some(output) = outputs.next() {
        let chunk = load_from_pipeline_output(pool, &output, policy).await?;
        agg.accumulate(&chunk);
    }
    Ok(agg)
}

pub async fn connect_sqlite(cfg: &WarehouseConfig) -> Result<Pool<Sqlite>, sqlx::Error> {
    SqlitePool::connect_lazy(&cfg.url)
}

#[derive(Debug, FromRow)]
pub struct DimPatientRow {
    pub patient_key: i64,
    pub patient_id: String,
}

impl From<&DimPatient> for DimPatientRow {
    fn from(dim: &DimPatient) -> Self {
        Self {
            patient_key: dim.key.0 as i64,
            patient_id: dim.patient_id.clone(),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct DimEncounterRow {
    pub encounter_key: i64,
    pub encounter_id: String,
    pub patient_key: i64,
}

impl From<&DimEncounter> for DimEncounterRow {
    fn from(dim: &DimEncounter) -> Self {
        Self {
            encounter_key: dim.key.0 as i64,
            encounter_id: dim.encounter_id.clone(),
            patient_key: dim.patient_key.0 as i64,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct DimCodeRow {
    pub code_key: i64,
    pub code_element_id: String,
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

impl From<&DimCode> for DimCodeRow {
    fn from(dim: &DimCode) -> Self {
        Self {
            code_key: dim.key.0 as i64,
            code_element_id: dim.code_element_id.clone(),
            system: dim.system.clone(),
            code: dim.code.clone(),
            display: dim.display.clone(),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct DimNCITRow {
    pub ncit_key: i64,
    pub ncit_id: String,
    pub preferred_name: String,
    pub semantic_group: Option<String>,
}

impl From<&DimNCIT> for DimNCITRow {
    fn from(dim: &DimNCIT) -> Self {
        Self {
            ncit_key: dim.key.0 as i64,
            ncit_id: dim.ncit_id.clone(),
            preferred_name: dim.preferred_name.clone(),
            semantic_group: Some(dim.semantic_group.clone()),
        }
    }
}

fn enforce_export_policy(
    output: &PipelineOutput,
    policy: &dfps_compliance::Policy,
) -> Result<(), LoadError> {
    let mut tiers = Vec::new();
    for mapping in &output.mapping_results {
        if let Some(label) = mapping.license_tier.as_deref()
            && let Some(tier) = parse_license_tier(label)
        {
            tiers.push(tier);
        }
    }
    assert_export_allowed(&tiers, policy).map_err(|err| LoadError::Compliance(err.to_string()))
}

fn parse_license_tier(value: &str) -> Option<LicenseTier> {
    match value.trim() {
        "licensed" => Some(LicenseTier::Licensed),
        "open" => Some(LicenseTier::Open),
        "internal_only" => Some(LicenseTier::InternalOnly),
        _ => None,
    }
}

pub async fn ncit_summary(pool: &Pool<Sqlite>) -> Result<AnalyticsSummaryResponse, sqlx::Error> {
    let sql = format!(
        r#"
SELECT
    COALESCE(dn.ncit_id, 'NO_MATCH') AS ncit_id,
    dn.preferred_name,
    fsr.mapping_state,
    {order_day_expr} AS order_day,
    COUNT(*) AS total_count
FROM fact_service_request fsr
LEFT JOIN dim_ncit dn ON fsr.ncit_key = dn.ncit_key
GROUP BY ncit_id, dn.preferred_name, fsr.mapping_state, order_day
ORDER BY total_count DESC, ncit_id ASC
"#,
        order_day_expr = ORDER_DAY_EXPR
    );

    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    let mut summary_rows = Vec::new();
    for row in rows {
        let ncit_id: String = row.try_get("ncit_id")?;
        let preferred_name: Option<String> = row.try_get("preferred_name")?;
        let mapping_state: Option<String> = row.try_get("mapping_state")?;
        let time_bucket: Option<String> = row.try_get("order_day")?;
        let count: i64 = row.try_get("total_count")?;
        summary_rows.push(AnalyticsSummaryRow {
            ncit_id,
            preferred_name,
            mapping_state,
            time_bucket,
            count: count as usize,
        });
    }
    Ok(AnalyticsSummaryResponse { rows: summary_rows })
}

pub async fn cohort(
    pool: &Pool<Sqlite>,
    filters: &CohortFilters,
) -> Result<CohortResponse, sqlx::Error> {
    let sql = format!(
        r#"
SELECT
    fsr.sr_id,
    dp.patient_id,
    de.encounter_id,
    dn.ncit_id,
    fsr.status,
    fsr.intent,
    fsr.description,
    fsr.ordered_at,
    fsr.mapping_state
FROM fact_service_request fsr
JOIN dim_patient dp ON dp.patient_key = fsr.patient_key
LEFT JOIN dim_encounter de ON de.encounter_key = fsr.encounter_key
LEFT JOIN dim_ncit dn ON fsr.ncit_key = dn.ncit_key
WHERE
    (?1 IS NULL OR dn.ncit_id = ?1)
    AND (?2 IS NULL OR fsr.status = ?2)
    AND (?3 IS NULL OR {order_day_expr} >= ?3)
    AND (?4 IS NULL OR {order_day_expr} <= ?4)
ORDER BY fsr.ordered_at ASC, fsr.sr_id ASC
"#,
        order_day_expr = ORDER_DAY_EXPR
    );

    let rows = sqlx::query(&sql)
        .bind(filters.ncit_id.as_deref())
        .bind(filters.status.as_deref())
        .bind(filters.date_from.as_deref())
        .bind(filters.date_to.as_deref())
        .fetch_all(pool)
        .await?;

    let mut cohort_rows = Vec::new();
    for row in rows {
        cohort_rows.push(CohortRow {
            sr_id: row.try_get("sr_id")?,
            patient_id: Some(row.try_get::<String, _>("patient_id")?),
            encounter_id: row.try_get("encounter_id")?,
            ncit_id: row.try_get("ncit_id")?,
            status: row.try_get("status")?,
            intent: row.try_get("intent")?,
            description: row.try_get("description")?,
            ordered_at: row.try_get("ordered_at")?,
            mapping_state: row.try_get("mapping_state")?,
        });
    }

    Ok(CohortResponse {
        total: cohort_rows.len(),
        rows: cohort_rows,
    })
}

#[derive(Debug, FromRow)]
pub struct FactServiceRequestRow {
    pub sr_id: String,
    pub patient_key: i64,
    pub encounter_key: Option<i64>,
    pub code_key: i64,
    pub ncit_key: Option<i64>,
    pub mapping_state: String,
    pub status: String,
    pub intent: String,
    pub description: String,
    pub ordered_at: Option<String>,
}

impl From<&FactServiceRequest> for FactServiceRequestRow {
    fn from(fact: &FactServiceRequest) -> Self {
        Self {
            sr_id: fact.sr_id.clone(),
            patient_key: fact.patient_key.0 as i64,
            encounter_key: fact.encounter_key.map(|k| k.0 as i64),
            code_key: fact.code_key.0 as i64,
            ncit_key: fact.ncit_key.map(|k| k.0 as i64),
            mapping_state: fact.mapping_state.clone(),
            status: fact.status.clone(),
            intent: fact.intent.clone(),
            description: fact.description.clone(),
            ordered_at: fact.ordered_at.clone(),
        }
    }
}

async fn upsert_patients(
    tx: &mut Transaction<'_, Sqlite>,
    dims: &[DimPatient],
) -> Result<u64, sqlx::Error> {
    let mut inserted = 0;
    for dim in dims {
        let row: DimPatientRow = dim.into();
        let res = sqlx::query(
            "INSERT OR IGNORE INTO dim_patient (patient_key, patient_id) VALUES (?, ?)",
        )
        .bind(row.patient_key)
        .bind(row.patient_id)
        .execute(&mut **tx)
        .await?;
        inserted += res.rows_affected();
    }
    Ok(inserted)
}

async fn upsert_encounters(
    tx: &mut Transaction<'_, Sqlite>,
    dims: &[DimEncounter],
) -> Result<u64, sqlx::Error> {
    let mut inserted = 0;
    for dim in dims {
        let row: DimEncounterRow = dim.into();
        let res = sqlx::query(
            "INSERT OR IGNORE INTO dim_encounter (encounter_key, encounter_id, patient_key) VALUES (?, ?, ?)",
        )
        .bind(row.encounter_key)
        .bind(row.encounter_id)
        .bind(row.patient_key)
        .execute(&mut **tx)
        .await?;
        inserted += res.rows_affected();
    }
    Ok(inserted)
}

async fn upsert_codes(
    tx: &mut Transaction<'_, Sqlite>,
    dims: &[DimCode],
) -> Result<u64, sqlx::Error> {
    let mut inserted = 0;
    for dim in dims {
        let row: DimCodeRow = dim.into();
        let res = sqlx::query(
            "INSERT OR IGNORE INTO dim_code (code_key, code_element_id, system, code, display) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(row.code_key)
        .bind(row.code_element_id)
        .bind(row.system)
        .bind(row.code)
        .bind(row.display)
        .execute(&mut **tx)
        .await?;
        inserted += res.rows_affected();
    }
    inserted += 0;
    Ok(inserted)
}

async fn upsert_ncit(
    tx: &mut Transaction<'_, Sqlite>,
    dims: &[DimNCIT],
) -> Result<u64, sqlx::Error> {
    let mut inserted = 0;
    for dim in dims {
        let row: DimNCITRow = dim.into();
        let res = sqlx::query(
            "INSERT OR IGNORE INTO dim_ncit (ncit_key, ncit_id, preferred_name, semantic_group) VALUES (?, ?, ?, ?)",
        )
        .bind(row.ncit_key)
        .bind(row.ncit_id)
        .bind(row.preferred_name)
        .bind(row.semantic_group)
        .execute(&mut **tx)
        .await?;
        inserted += res.rows_affected();
    }
    Ok(inserted)
}

async fn insert_facts(
    tx: &mut Transaction<'_, Sqlite>,
    facts: &[FactServiceRequest],
) -> Result<u64, sqlx::Error> {
    let mut inserted = 0;
    for fact in facts {
        let row: FactServiceRequestRow = fact.into();
        let res = sqlx::query(
            "INSERT OR REPLACE INTO fact_service_request (sr_id, patient_key, encounter_key, code_key, ncit_key, mapping_state, status, intent, description, ordered_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(row.sr_id)
        .bind(row.patient_key)
        .bind(row.encounter_key)
        .bind(row.code_key)
        .bind(row.ncit_key)
        .bind(row.mapping_state)
        .bind(row.status)
        .bind(row.intent)
        .bind(row.description)
        .bind(row.ordered_at)
        .execute(&mut **tx)
        .await?;
        inserted += res.rows_affected();
    }
    Ok(inserted)
}

async fn ensure_fact_mapping_state_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let columns = sqlx::query("PRAGMA table_info('fact_service_request')")
        .fetch_all(pool)
        .await?;
    let has_column = columns.iter().any(|row| {
        row.try_get::<String, _>("name")
            .map(|name| name == "mapping_state")
            .unwrap_or(false)
    });
    if !has_column {
        sqlx::query(
            "ALTER TABLE fact_service_request ADD COLUMN mapping_state TEXT NOT NULL DEFAULT 'unknown'",
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dfps_compliance::{ComplianceMode, Policy};
    use dfps_contracts::{
        DimNCITConcept, MappingResult, MappingSourceVersion, MappingState, PipelineOutput,
        StgSrCodeExploded,
    };
    use dfps_core::{
        clinical::order::{ServiceRequestIntent, ServiceRequestStatus},
        mapping::{MappingStrategy, MappingThresholds},
        staging::StgServiceRequestFlat,
    };
    use sqlx::sqlite::SqlitePoolOptions;
    use std::env;
    use std::sync::Mutex;

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    fn sample_output() -> PipelineOutput {
        PipelineOutput {
            flats: vec![StgServiceRequestFlat {
                sr_id: "SR-1".into(),
                patient_id: "PAT-1".into(),
                encounter_id: Some("ENC-1".into()),
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
                score: 0.98,
                strategy: MappingStrategy::Lexical,
                state: MappingState::AutoMapped,
                thresholds: MappingThresholds::default(),
                source_version: MappingSourceVersion::new("ncit", "umls"),
                reason: None,
                license_tier: None,
                source_kind: None,
            }],
            dim_concepts: vec![DimNCITConcept {
                ncit_id: "C1234".into(),
                preferred_name: "FDG Uptake".into(),
                semantic_group: "Procedure".into(),
            }],
            vector_usage: None,
        }
    }

    fn sample_no_match_output() -> PipelineOutput {
        PipelineOutput {
            flats: vec![StgServiceRequestFlat {
                sr_id: "SR-NM-1".into(),
                patient_id: "PAT-NM-1".into(),
                encounter_id: None,
                status: "active".into(),
                status_enum: ServiceRequestStatus::Active,
                intent: "order".into(),
                intent_enum: ServiceRequestIntent::Order,
                description: "Unknown".into(),
                ordered_at: None,
            }],
            exploded_codes: vec![StgSrCodeExploded {
                sr_id: "SR-NM-1".into(),
                system: Some("http://example.org".into()),
                code: Some("UNK-1".into()),
                display: Some("unknown".into()),
            }],
            mapping_results: vec![MappingResult {
                code_element_id: "SR-NM-1::http://example.org::UNK-1".into(),
                cui: None,
                ncit_id: None,
                score: 0.0,
                strategy: MappingStrategy::Lexical,
                state: MappingState::NoMatch,
                thresholds: MappingThresholds::default(),
                source_version: MappingSourceVersion::new("ncit", "regression"),
                reason: Some("no_match".into()),
                license_tier: None,
                source_kind: None,
            }],
            dim_concepts: Vec::new(),
            vector_usage: None,
        }
    }

    fn reset_env() {
        for key in [
            "DFPS_WAREHOUSE_URL",
            "DFPS_WAREHOUSE_SCHEMA",
            "DFPS_WAREHOUSE_MAX_CONNECTIONS",
        ] {
            unsafe {
                env::remove_var(key);
            }
        }
    }

    async fn seed_pool() -> Pool<Sqlite> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect sqlite");
        super::migrate(&pool).await.expect("migrate memory db");
        let policy = Policy::default_for_mode(ComplianceMode::Internal);
        super::load_from_pipeline_output(&pool, &sample_output(), &policy)
            .await
            .expect("load pipeline output");
        pool
    }

    #[test]
    fn warehouse_config_requires_url() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        let err = WarehouseConfig::from_env().unwrap_err();
        matches!(err, WarehouseConfigError::MissingUrl);
    }

    #[test]
    fn warehouse_config_parses_env_values() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        unsafe {
            env::set_var("DFPS_WAREHOUSE_URL", "sqlite://datamart.db");
            env::set_var("DFPS_WAREHOUSE_SCHEMA", "analytics");
            env::set_var("DFPS_WAREHOUSE_MAX_CONNECTIONS", "9");
        }
        let cfg = WarehouseConfig::from_env().expect("config");
        assert_eq!(cfg.url, "sqlite://datamart.db");
        assert_eq!(cfg.schema.as_deref(), Some("analytics"));
        assert_eq!(cfg.max_connections, 9);
        reset_env();
    }

    #[tokio::test]
    async fn streaming_loader_accumulates_rows() {
        let cfg = WarehouseConfig {
            url: "sqlite::memory:".into(),
            schema: None,
            max_connections: 1,
        };
        let pool = connect_sqlite(&cfg).await.expect("pool");
        migrate(&pool).await.expect("migrate");
        let policy = Policy::default_for_mode(ComplianceMode::Partner);
        let mut iterator = vec![sample_output(), sample_output()].into_iter();
        let summary = load_streaming_iter(&pool, &mut iterator, &policy)
            .await
            .expect("streaming summary");
        assert_eq!(summary.facts, 2);
        assert_eq!(summary.patients, 1);
    }

    #[tokio::test]
    async fn load_handles_no_match_and_duplicates() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect sqlite");
        super::migrate(&pool).await.expect("migrate");
        let policy = Policy::default_for_mode(ComplianceMode::Internal);
        super::load_from_pipeline_output(&pool, &sample_no_match_output(), &policy)
            .await
            .expect("initial load");
        super::load_from_pipeline_output(&pool, &sample_no_match_output(), &policy)
            .await
            .expect("duplicate load");
        let fact_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fact_service_request")
            .fetch_one(&pool)
            .await
            .expect("count facts");
        assert_eq!(fact_count, 1);
        let dims: Vec<String> = sqlx::query_scalar("SELECT ncit_id FROM dim_ncit")
            .fetch_all(&pool)
            .await
            .expect("fetch dim_ncit");
        assert!(
            dims.iter().any(|id| id == "NO_MATCH"),
            "expected NO_MATCH dim in {:?}",
            dims
        );
    }

    #[tokio::test]
    async fn load_rejects_compliance_export() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect sqlite");
        super::migrate(&pool).await.expect("migrate");
        let policy = Policy::default_for_mode(ComplianceMode::OpenSource);
        let mut restricted = sample_output();
        restricted.mapping_results[0].license_tier = Some("licensed".into());
        let err = super::load_from_pipeline_output(&pool, &restricted, &policy)
            .await
            .expect_err("compliance block");
        assert!(matches!(err, LoadError::Compliance(_)));
    }

    #[tokio::test]
    async fn ncit_summary_reports_rows() {
        let pool = seed_pool().await;
        let response = super::ncit_summary(&pool).await.expect("summary query");
        assert!(!response.rows.is_empty());
        let row = &response.rows[0];
        assert_eq!(row.ncit_id, "C1234");
        assert_eq!(row.mapping_state.as_deref(), Some("auto_mapped"));
        assert_eq!(row.count, 1);
    }

    #[tokio::test]
    async fn cohort_filters_by_ncit_id() {
        let pool = seed_pool().await;
        let response = super::cohort(
            &pool,
            &CohortFilters {
                ncit_id: Some("C1234".into()),
                ..Default::default()
            },
        )
        .await
        .expect("cohort query");
        assert_eq!(response.total, 1);
        let row = &response.rows[0];
        assert_eq!(row.sr_id, "SR-1");
        assert_eq!(row.ncit_id.as_deref(), Some("C1234"));
        assert_eq!(row.mapping_state.as_deref(), Some("auto_mapped"));

        let empty = super::cohort(
            &pool,
            &CohortFilters {
                ncit_id: Some("UNKNOWN".into()),
                ..Default::default()
            },
        )
        .await
        .expect("empty filters");
        assert_eq!(empty.total, 0);
    }
}
