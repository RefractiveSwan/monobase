use std::sync::Arc;

use async_trait::async_trait;
use dfps_compliance::Policy;
use dfps_contracts::{AnalyticsSummaryResponse, CohortResponse, LoadSummary, PipelineOutput};
use sqlx::{Pool, Sqlite};
use thiserror::Error;
use tokio::sync::OnceCell;

use crate::{
    CohortFilters, LoadError, WarehouseConfig, cohort, connect_sqlite, load_from_pipeline_output,
    migrate, ncit_summary,
};

/// Datamart adapter errors surfaced to app crates.
#[derive(Debug, Error)]
pub enum DatamartError {
    #[error("datamart not configured")]
    Disabled,
    #[error(transparent)]
    Load(#[from] LoadError),
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
}

/// Outbound port for persisting/querying analytics data.
#[async_trait]
pub trait DatamartSink: Send + Sync {
    async fn persist(
        &self,
        output: &PipelineOutput,
        policy: &Policy,
    ) -> Result<LoadSummary, DatamartError>;

    async fn ncit_summary(&self) -> Result<AnalyticsSummaryResponse, DatamartError>;

    async fn cohort(&self, filters: &CohortFilters) -> Result<CohortResponse, DatamartError>;
}

/// Sqlite-backed implementation of [`DatamartSink`].
#[derive(Clone)]
pub struct SqliteDatamart {
    config: Option<WarehouseConfig>,
    pool: Arc<OnceCell<Pool<Sqlite>>>,
}

impl SqliteDatamart {
    /// Build from env, allowing the datamart to be disabled if config is absent.
    pub fn from_env() -> Self {
        Self {
            config: WarehouseConfig::from_env().ok(),
            pool: Arc::new(OnceCell::new()),
        }
    }

    /// Explicit config constructor (useful for tests).
    pub fn from_config(config: WarehouseConfig) -> Self {
        Self {
            config: Some(config),
            pool: Arc::new(OnceCell::new()),
        }
    }

    /// Construct from an optional config (disabled when `None`).
    pub fn from_optional_config(config: Option<WarehouseConfig>) -> Self {
        Self {
            config,
            pool: Arc::new(OnceCell::new()),
        }
    }

    async fn pool(&self) -> Result<Pool<Sqlite>, DatamartError> {
        let cfg = self.config.as_ref().ok_or(DatamartError::Disabled)?;
        let pool = self
            .pool
            .get_or_try_init(|| async {
                let pool = connect_sqlite(cfg).await?;
                migrate(&pool).await?;
                Ok::<_, sqlx::Error>(pool)
            })
            .await
            .map_err(DatamartError::Sql)?;
        Ok(pool.clone())
    }
}

#[async_trait]
impl DatamartSink for SqliteDatamart {
    async fn persist(
        &self,
        output: &PipelineOutput,
        policy: &Policy,
    ) -> Result<LoadSummary, DatamartError> {
        let pool = self.pool().await?;
        let summary = load_from_pipeline_output(&pool, output, policy).await?;
        Ok(summary)
    }

    async fn ncit_summary(&self) -> Result<AnalyticsSummaryResponse, DatamartError> {
        let pool = self.pool().await?;
        let summary = ncit_summary(&pool).await?;
        Ok(summary)
    }

    async fn cohort(&self, filters: &CohortFilters) -> Result<CohortResponse, DatamartError> {
        let pool = self.pool().await?;
        let cohort = cohort(&pool, filters).await?;
        Ok(cohort)
    }
}
