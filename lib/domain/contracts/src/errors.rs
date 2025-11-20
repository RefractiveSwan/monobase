//! Shared error taxonomy/code definitions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// High-level category for user-facing errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    DomainIngestion,
    DomainMapping,
    DomainCompliance,
    DomainVector,
    DomainWarehouse,
    PlatformConfig,
    PlatformStore,
    PlatformTerminology,
    AppHttpClient,
    AppHttpServer,
    AppCliUsage,
    AppCliRuntime,
}

/// Stable error code string shared across CLI/API responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidJson,
    InvalidFhir,
    ComplianceBlocked,
    InvalidDataset,
    VectorBackendUnavailable,
    WarehouseExportBlocked,
    InternalError,
}

impl ErrorCode {
    /// Return the canonical string for the code.
    pub const fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InvalidJson => "invalid_json",
            ErrorCode::InvalidFhir => "invalid_fhir",
            ErrorCode::ComplianceBlocked => "compliance_blocked",
            ErrorCode::InvalidDataset => "invalid_dataset",
            ErrorCode::VectorBackendUnavailable => "vector_backend_unavailable",
            ErrorCode::WarehouseExportBlocked => "warehouse_export_blocked",
            ErrorCode::InternalError => "internal_error",
        }
    }
}
