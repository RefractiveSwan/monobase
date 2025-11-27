//! Shared DTOs for the refractive_swan web surfaces (frontend + API).
//! Re-exports the canonical contracts used by HTTP handlers and clients so
//! both sides stay in sync.

pub use refractive_swan_contracts::{
    AdminEvent, AdminEventKind, AnalyticsSummaryResponse, AnalyticsSummaryRow, CohortResponse,
    CohortRow, DatasetListEntry, DatasetManifest, EvalRunResponse, EvalSummary, PipelineMetrics,
    PipelineOutput,
};

pub use refractive_swan_contracts::errors::{ErrorCode, ErrorKind};
