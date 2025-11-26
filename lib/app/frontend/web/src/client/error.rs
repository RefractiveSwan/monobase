use log::{error, warn};
use refractive_swan_web_dto::{ErrorCode, ErrorKind};
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{0}")]
    Backend(BackendError),
    #[error("bundle payload missing")]
    EmptyBundle,
    #[error("invalid bundle JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("unable to read upload: {0}")]
    Upload(String),
    #[error("utf-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl ClientError {
    /// Human-friendly error fragment that can be shown in views.
    pub fn user_message(&self) -> String {
        match self {
            ClientError::Backend(err) => err.user_message(),
            ClientError::Http(inner) => {
                format!("Unable to reach backend: {}", inner)
            }
            ClientError::InvalidJson(inner) => format!("Invalid JSON: {inner}"),
            ClientError::Upload(msg) => msg.clone(),
            ClientError::Utf8(inner) => format!("UTF-8 error: {inner}"),
            ClientError::EmptyBundle => "No bundle payload supplied".to_string(),
        }
    }

    pub(super) fn log(&self) {
        match self {
            ClientError::Backend(err) => {
                warn!(
                    target: "refractive_swan_web_frontend.client",
                    "backend error: {err}"
                );
            }
            ClientError::Http(err) => {
                error!(
                    target: "refractive_swan_web_frontend.client",
                    "http client error: {err}"
                );
            }
            other => {
                warn!(
                    target: "refractive_swan_web_frontend.client",
                    "client error: {other}"
                );
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendError {
    status: StatusCode,
    code: Option<ErrorCode>,
    kind: Option<ErrorKind>,
    message: String,
    request_id: Option<String>,
}

impl BackendError {
    pub(super) fn from_http(status: StatusCode, body: String) -> Self {
        match serde_json::from_str::<BackendErrorBody>(&body) {
            Ok(payload) => Self {
                status,
                code: Some(payload.code),
                kind: Some(payload.kind),
                message: payload.message,
                request_id: Some(payload.request_id),
            },
            Err(_) => Self {
                status,
                code: None,
                kind: None,
                message: body.clone(),
                request_id: None,
            },
        }
    }

    pub(super) fn user_message(&self) -> String {
        if let Some(code) = self.code {
            if let Some(request_id) = &self.request_id {
                return format!(
                    "{} ({}) [request_id={request_id}]",
                    self.message,
                    code.as_str()
                );
            }
            return format!("{} ({})", self.message, code.as_str());
        }
        if !self.message.trim().is_empty() {
            return format!(
                "Backend responded with status {}: {}",
                self.status.as_u16(),
                self.message
            );
        }
        format!("Backend responded with status {}", self.status.as_u16())
    }
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = self.code {
            write!(f, "{} ({})", self.message, code.as_str())?;
        } else if !self.message.trim().is_empty() {
            write!(f, "{} (status {})", self.message, self.status)?;
        } else {
            write!(f, "status {}", self.status)?;
        }
        if let Some(request_id) = &self.request_id {
            write!(f, " [request_id={request_id}]")?;
        }
        if let Some(kind) = self.kind {
            write!(f, " [{kind:?}]")?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct BackendErrorBody {
    code: ErrorCode,
    kind: ErrorKind,
    message: String,
    request_id: String,
}
