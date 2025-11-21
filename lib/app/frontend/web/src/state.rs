use crate::{client::BackendClient, config::AppConfig};
use refractive_swan_observability::PipelineMetrics;
use log::info;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub client: BackendClient,
    pub dataset_store: Arc<dyn refractive_swan_eval::DatasetStore + Send + Sync>,
    pub analytics_metrics: Arc<Mutex<PipelineMetrics>>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        client: BackendClient,
        dataset_store: Arc<dyn refractive_swan_eval::DatasetStore + Send + Sync>,
    ) -> Self {
        Self {
            config,
            client,
            dataset_store,
            analytics_metrics: Arc::new(Mutex::new(PipelineMetrics::default())),
        }
    }

    pub fn record_analytics(&self, cohort_total: Option<usize>) {
        if let Ok(mut metrics) = self.analytics_metrics.lock() {
            metrics.analytics_requests += 1;
            if let Some(total) = cohort_total {
                metrics.cohort_queries += 1;
                metrics.cohort_results_total += total;
                if metrics.cohort_queries > 0 {
                    metrics.avg_cohort_size =
                        Some(metrics.cohort_results_total as f32 / metrics.cohort_queries as f32);
                }
            }
            info!(
                target: "refractive_swan_web_frontend.analytics",
                "analytics_requests={} cohort_queries={} last_total={:?}",
                metrics.analytics_requests,
                metrics.cohort_queries,
                cohort_total
            );
        }
    }
}
