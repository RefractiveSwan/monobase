use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

/// Errors exposed by the server wrapper.
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("invalid bind host '{host}': {source}")]
    InvalidHost {
        host: String,
        #[source]
        source: std::net::AddrParseError,
    },
    #[error("failed to bind server at {addr}: {source}")]
    Bind {
        addr: std::net::SocketAddr,
        #[source]
        source: std::io::Error,
    },
    #[error("server error: {0}")]
    Serve(#[source] std::io::Error),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: refractive_swan_contracts::ErrorCode,
    pub kind: refractive_swan_contracts::ErrorKind,
    pub message: String,
    pub request_id: Uuid,
}

#[derive(Debug, Clone)]
pub enum ApiError {
    InvalidJson {
        message: String,
        request_id: Uuid,
    },
    Ingestion {
        message: String,
        request_id: Uuid,
    },
    InvalidDataset {
        message: String,
        request_id: Uuid,
    },
    Compliance {
        message: String,
        request_id: Uuid,
    },
    #[allow(dead_code)]
    Internal {
        message: String,
        request_id: Uuid,
    },
}

impl ApiError {
    pub fn invalid_json(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        log::warn!(
            target: "refractive_swan_api",
            "request_id={request_id} invalid json: {message}"
        );
        Self::InvalidJson {
            message,
            request_id,
        }
    }

    pub fn ingestion(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        log::warn!(
            target: "refractive_swan_api",
            "request_id={request_id} invalid fhir payload: {message}"
        );
        Self::Ingestion {
            message,
            request_id,
        }
    }

    pub fn invalid_dataset(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        log::warn!(
            target: "refractive_swan_api",
            "request_id={request_id} invalid dataset: {message}"
        );
        Self::InvalidDataset {
            message,
            request_id,
        }
    }

    pub fn compliance(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        log::warn!(
            target: "refractive_swan_compliance",
            "request_id={request_id} export blocked: {message}"
        );
        Self::Compliance {
            message,
            request_id,
        }
    }

    #[allow(dead_code)]
    pub fn internal(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        log::error!(
            target: "refractive_swan_api",
            "request_id={request_id} internal error: {message}"
        );
        Self::Internal {
            message,
            request_id,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, kind, message, request_id) = match self {
            ApiError::InvalidJson {
                message,
                request_id,
            } => (
                StatusCode::BAD_REQUEST,
                refractive_swan_contracts::ErrorCode::InvalidJson,
                refractive_swan_contracts::ErrorKind::AppHttpServer,
                message,
                request_id,
            ),
            ApiError::Ingestion {
                message,
                request_id,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                refractive_swan_contracts::ErrorCode::InvalidFhir,
                refractive_swan_contracts::ErrorKind::DomainIngestion,
                message,
                request_id,
            ),
            ApiError::InvalidDataset {
                message,
                request_id,
            } => (
                StatusCode::BAD_REQUEST,
                refractive_swan_contracts::ErrorCode::InvalidDataset,
                refractive_swan_contracts::ErrorKind::DomainMapping,
                message,
                request_id,
            ),
            ApiError::Compliance {
                message,
                request_id,
            } => (
                StatusCode::FORBIDDEN,
                refractive_swan_contracts::ErrorCode::ComplianceBlocked,
                refractive_swan_contracts::ErrorKind::DomainCompliance,
                message,
                request_id,
            ),
            ApiError::Internal {
                message,
                request_id,
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                refractive_swan_contracts::ErrorCode::InternalError,
                refractive_swan_contracts::ErrorKind::AppHttpServer,
                message,
                request_id,
            ),
        };
        (
            status,
            Json(ErrorResponse {
                code,
                kind,
                message,
                request_id,
            }),
        )
            .into_response()
    }
}
