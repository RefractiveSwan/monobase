use std::sync::Arc;

use async_trait::async_trait;
use refractive_swan_compliance::Policy;
use refractive_swan_contracts::{
    AnalyticsSummaryResponse, CohortResponse, LoadSummary, PipelineOutput,
};
use refractive_swan_datamart_port::{CohortFilters, DatamartError, DatamartSink};
use sqlx::{Pool, Sqlite};
use tokio::sync::OnceCell;

use crate::{
    WarehouseConfig, cohort, connect_sqlite, load_from_pipeline_output, migrate, ncit_summary,
};

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
            .map_err(|err| DatamartError::Backend(err.to_string()))?;
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
        let summary = load_from_pipeline_output(&pool, output, policy)
            .await
            .map_err(|err| DatamartError::Load(err.to_string()))?;
        Ok(summary)
    }

    async fn ncit_summary(&self) -> Result<AnalyticsSummaryResponse, DatamartError> {
        let pool = self.pool().await?;
        let summary = ncit_summary(&pool)
            .await
            .map_err(|err| DatamartError::Backend(err.to_string()))?;
        Ok(summary)
    }

    async fn cohort(&self, filters: &CohortFilters) -> Result<CohortResponse, DatamartError> {
        let pool = self.pool().await?;
        let cohort = cohort(&pool, filters)
            .await
            .map_err(|err| DatamartError::Backend(err.to_string()))?;
        Ok(cohort)
    }
}
