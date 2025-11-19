//! Workspace-wide observability helpers for logging and metrics snapshots.
//!
//! Hooks into the Bundle -> NCIt pipeline so CLIs can emit structured log
//! events and tests can validate mapping state distributions (OBS-01 / OBS-02).
//! See:
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-12--platform-observability--metrics-dfps_observability
//! - docs/system-design/clinical/ncit/behavior/sequence-servicerequest.md
//! - docs/runbook/040-warehouse-and-bi/bi-integration-quickstart.md

mod env;
mod logging;
mod metrics;
mod snapshot;
pub mod vector_usage;

pub use env::init_environment;
pub use logging::{log_no_match, log_pipeline_output};
pub use metrics::{PipelineMetrics, apply_vector_usage};
pub use snapshot::{MetricsRatios, MetricsSnapshot, metrics_snapshot, metrics_snapshot_json};
pub use vector_usage::{VectorCapacitySnapshot, VectorUsageSnapshot};
