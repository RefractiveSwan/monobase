use std::{fmt, process};

use dfps_pipeline::PipelineError;

#[derive(Debug, Clone, Copy)]
pub enum ExitCode {
    Success = 0,
    Config = 10,
    InvalidInput = 11,
    Io = 12,
    Compliance = 13,
    External = 14,
    Internal = 15,
}

#[derive(Debug, Clone)]
pub struct CliError {
    pub code: ExitCode,
    pub message: String,
}

impl CliError {
    pub fn new(code: ExitCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new(ExitCode::Config, message)
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(ExitCode::InvalidInput, message)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ExitCode::Io, message)
    }

    pub fn compliance(message: impl Into<String>) -> Self {
        Self::new(ExitCode::Compliance, message)
    }

    pub fn external(message: impl Into<String>) -> Self {
        Self::new(ExitCode::External, message)
    }

    pub fn report(self, bin: &str) -> ! {
        eprintln!("[{bin}] {}", self);
        process::exit(self.code as i32);
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CliError {}

pub type CliResult<T> = Result<T, CliError>;

pub fn run_bin(bin: &str, run: impl FnOnce() -> CliResult<()>) {
    if let Err(err) = run() {
        err.report(bin);
    }
}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        CliError::io(value.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(value: serde_json::Error) -> Self {
        CliError::invalid(value.to_string())
    }
}

impl From<sqlx::Error> for CliError {
    fn from(value: sqlx::Error) -> Self {
        CliError::external(value.to_string())
    }
}

impl From<PipelineError> for CliError {
    fn from(value: PipelineError) -> Self {
        CliError::invalid(value.to_string())
    }
}
