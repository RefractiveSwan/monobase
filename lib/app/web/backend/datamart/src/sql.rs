use dfps_compliance::{assert_export_allowed, load_policy_from_env};
use dfps_configuration::load_env;
use dfps_pipeline::PipelineOutput;
use dfps_terminology::codesystem::LicenseTier;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite, SqlitePool, Transaction};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseConfig {
    pub url: String,
    pub schema: Option<String>,
    pub max_connections: u32,
}

impl WarehouseConfig {
    pub fn from_env() -> Result<Self, String> {
        load_env("domain.datamart")
            .map_err(|err| format!("warehouse env load error: {err}"))
            .ok();
        let url = std::env::var("DFPS_WAREHOUSE_URL")
            .map_err(|_| "DFPS_WAREHOUSE_URL missing".to_string())?;
        let schema = std::env::var("DFPS_WAREHOUSE_SCHEMA").ok();
        let max_connections = std::env::var("DFPS_WAREHOUSE_MAX_CONNECTIONS")
            .ok()
            .and_then(|raw| raw.parse::<u32>().ok())
            .unwrap_or(5);
        Ok(Self {
            url,
            schema,
            max_connections,
        })
    }
}

pub async fn migrate(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    for stmt in ddl_statements() {
        sqlx::query(stmt).execute(pool).await?;
    }
    Ok(())
}

/// Summary of rows inserted or updated during a load.
#[derive(Debug, Default, Clone, Serialize)]
pub struct LoadSummary {
    pub patients: u64,
    pub encounters: u64,
    pub codes: u64,
    pub ncit: u64,
    pub facts: u64,
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error("export blocked by compliance policy: {0}")]
    Compliance(String),
    #[error("failed to load compliance policy: {0}")]
    CompliancePolicy(dfps_compliance::ComplianceError),
}

/// Load a PipelineOutput into the warehouse, upserting dims and inserting facts.
pub async fn load_from_pipeline_output(
    pool: &Pool<Sqlite>,
    output: &PipelineOutput,
) -> Result<LoadSummary, LoadError> {
    let policy = load_policy_from_env().map_err(LoadError::CompliancePolicy)?;
    enforce_export_policy(output, &policy)?;

    let (dims, facts) = crate::from_pipeline_output(output);
    let mut tx = pool.begin().await?;
    let mut summary = LoadSummary::default();

    summary.patients = upsert_patients(&mut tx, &dims.patients).await?;
    summary.encounters = upsert_encounters(&mut tx, &dims.encounters).await?;
    summary.codes = upsert_codes(&mut tx, &dims.codes).await?;
    summary.ncit = upsert_ncit(&mut tx, &dims.ncit).await?;
    summary.facts = insert_facts(&mut tx, &facts).await?;

    tx.commit().await?;
    Ok(summary)
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
        if let Some(label) = mapping.license_tier.as_deref() {
            if let Some(tier) = parse_license_tier(label) {
                tiers.push(tier);
            }
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

#[derive(Debug, FromRow)]
pub struct FactServiceRequestRow {
    pub sr_id: String,
    pub patient_key: i64,
    pub encounter_key: Option<i64>,
    pub code_key: i64,
    pub ncit_key: Option<i64>,
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
            "INSERT OR REPLACE INTO fact_service_request (sr_id, patient_key, encounter_key, code_key, ncit_key, status, intent, description, ordered_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(row.sr_id)
        .bind(row.patient_key)
        .bind(row.encounter_key)
        .bind(row.code_key)
        .bind(row.ncit_key)
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
