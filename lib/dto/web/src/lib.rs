//! Shared DTOs for the DFPS web surfaces (frontend + API).
//! Re-exports the canonical contracts used by HTTP handlers and clients so
//! both sides stay in sync.

pub use dfps_contracts::{
    AnalyticsSummaryResponse, AnalyticsSummaryRow, CohortResponse, CohortRow, DatasetManifest,
    EvalRunResponse, EvalSummary, PipelineMetrics, PipelineOutput,
};

pub use dfps_contracts::errors::{ErrorCode, ErrorKind};
